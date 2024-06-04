use anyhow::Error;
use indexmap::IndexMap;
use resolvo::{
    Candidates, Dependencies, DependencyProvider, KnownDependencies, NameId, Pool, SolvableId,
    SolverCache, VersionSetId,
};
use serde_json;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use toml::Value;

use crate::index::Index;
use crate::module::{Module, Version, VersionReq};
use crate::repository::{BareRepository, RefType};

pub struct Inventory {
    index: Index,
    pool: Rc<Pool<VersionReq>>,
    index_cache: IndexMap<String, IndexMap<Version, IndexMap<String, VersionReq>>>,
    cache_path: PathBuf,
}

impl Inventory {
    pub fn new(index_url: &str, cache_path: &Path) -> Result<Self, Error> {
        let index_cache_path = cache_path.join("index");
        std::fs::create_dir_all(&index_cache_path)?;
        let index = Index::new(
            index_url,
            &RefType::Branch(String::from("main")),
            &index_cache_path
        )?;

        Ok(Inventory {
            index,
            pool: Rc::new(Pool::new()),
            cache_path: cache_path.to_path_buf(),
            index_cache: IndexMap::new()
        })
    }

    pub fn pool(&self) -> Rc<Pool<VersionReq>> {
        self.pool.clone()
    }
    pub fn index(&self) -> &Index { &self.index }

    pub fn update_cache(&mut self) -> Result<(), Error> {
        let modules_cache_path = self.cache_path.join("modules");
        let cache_file = self.cache_path.join("cache.json");
        std::fs::create_dir_all(&modules_cache_path)?;
        let cache_contents = std::fs::read_to_string(&cache_file).unwrap_or_default();
        if cache_contents.len() > 0 {
            self.index_cache = serde_json::from_str(&cache_contents)?;
        }
        for module in self.index.get_modules()? {
            let url = self.index.get_url(&module)?;
            let repo = BareRepository::new(url, Some(modules_cache_path.join(&module).as_path()))?;
            let versions = repo.get_tags()?;
            for ref version in versions {
                let download_manifest = match self.index_cache.get(module) {
                    Some(inner_map) => !inner_map.contains_key(&Version::from_str(&version)?),
                    None => true,
                };
                if download_manifest {
                    let req_ref = RefType::Tag(version.to_string());
                    let manifest_contents =
                        String::from_utf8(repo.get_file(&req_ref, Path::new("manifest.toml"))?)?;
                    let manifest: Value = toml::from_str(manifest_contents.as_str())?;
                    let module_entry = self
                        .index_cache
                        .entry(module.to_string())
                        .or_insert_with(IndexMap::new);
                    let version_entry = module_entry
                        .entry(Version::from_str(&version)?)
                        .or_insert_with(IndexMap::new);
                    if let Some(dependencies) = manifest.get("dependencies") {
                        for (dep_name, dep_version) in dependencies.as_table().unwrap() {
                            version_entry.insert(
                                dep_name.to_string(),
                                VersionReq::from_str(dep_version.as_str().unwrap())?,
                            );
                        }
                    }
                }
            }
        }
        let updated_cache_contents = serde_json::to_string_pretty(&self.index_cache)?;
        std::fs::write(&cache_file, updated_cache_contents)?;
        Ok(())
    }

    pub fn add_module(&mut self, module: Module) {
        let entry = self
            .index_cache
            .entry(module.name.to_string())
            .or_insert_with(|| IndexMap::new());
        entry.insert(module.version, module.dependencies);
    }

    pub fn map_dependency(&self, name: &str, range: &VersionReq) -> VersionSetId {
        let dep_name = self.pool.intern_package_name(name);
        self.pool.intern_version_set(dep_name, range.clone())
    }

    pub fn map_solvable(&self, solvable: &SolvableId) -> (&str, &semver::Version) {
        let s = self.pool.resolve_solvable(*solvable);
        (self.pool.resolve_package_name(s.name_id()), &s.inner().0)
    }
}

impl DependencyProvider<VersionReq> for &Inventory {
    fn pool(&self) -> Rc<Pool<VersionReq>> {
        self.pool.clone()
    }

    async fn sort_candidates(
        &self,
        _solver: &SolverCache<VersionReq, String, Self>,
        solvables: &mut [SolvableId],
    ) {
        solvables.sort_by(|a, b| {
            let a = self.pool.resolve_solvable(*a).inner();
            let b = self.pool.resolve_solvable(*b).inner();
            b.cmp(&a)
        });
    }

    async fn get_candidates(&self, name: NameId) -> Option<Candidates> {
        let package_name = self.pool.resolve_package_name(name);
        let Some(package) = self.index_cache.get(package_name) else {
            return None;
        };

        let mut candidates = Candidates {
            candidates: Vec::with_capacity(package.len()),
            ..Candidates::default()
        };
        for version in package.keys() {
            let solvable = self.pool.intern_solvable(name, version.clone());
            candidates.candidates.push(solvable);
        }

        Some(candidates)
    }

    async fn get_dependencies(&self, solvable: SolvableId) -> Dependencies {
        let candidate = self.pool.resolve_solvable(solvable);
        let package_name = self.pool.resolve_package_name(candidate.name_id());
        let version = candidate.inner();
        let Some(deps) = self
            .index_cache
            .get(package_name)
            .and_then(|v| v.get(version))
        else {
            return Dependencies::Known(Default::default());
        };

        let mut result = KnownDependencies {
            requirements: Vec::with_capacity(deps.len()),
            constrains: vec![],
        };

        for req in deps.iter() {
            let dep_name = self.pool.intern_package_name(req.0);
            let dep_spec = self.pool.intern_version_set(dep_name, req.1.clone().into());
            result.requirements.push(dep_spec);
        }

        Dependencies::Known(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_cache() -> Result<(), anyhow::Error> {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut inventory = Inventory::new("https://github.com/jasal82/index.git", temp_dir.path());
        inventory.update_cache()
    }
}
