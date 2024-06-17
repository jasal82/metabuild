use anyhow::Error;
use indexmap::IndexMap;
use resolvo::{
    Candidates, Dependencies, DependencyProvider, KnownDependencies, NameId, Pool, SolvableId,
    SolverCache, VersionSetId,
};
use serde::Deserialize;
use serde_json;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use toml::Value;
use ureq;

use crate::index::{Index, LocationInfo};
use crate::package::{Package, Version, VersionReq};
use crate::repository::{BareRepository, RefType};

// TODO remove this trait and one of the retriever impls; there's only one necessary that can switch based on the LocationInfo variant
trait MetadataRetriever {
    fn fetch_versions(&self) -> Result<Vec<String>, Error>;
    fn fetch_package_manifest(&self, version: &str) -> Result<Value, Error>;
}

struct GitMetadataRetriever {
    repo: BareRepository,
}
impl GitMetadataRetriever {
    fn new(name: &str, url: &str, cache_path: &Path) -> Result<Self, Error> {
        let repo = BareRepository::new(url, Some(cache_path.join(name).as_path()))?;
        Ok(Self { repo })
    }
}

impl MetadataRetriever for GitMetadataRetriever {
    fn fetch_versions(&self) -> Result<Vec<String>, Error> {
        self.repo.get_tags()
    }

    fn fetch_package_manifest(&self, version: &str) -> Result<Value, Error> {
        let req_ref = RefType::Tag(version.to_string());
        let manifest_contents =
            String::from_utf8(self.repo.get_file(&req_ref, Path::new("manifest.toml"))?)?;
        toml::from_str(manifest_contents.as_str()).map_err(|e| anyhow::anyhow!("Failed to parse manifest: {e}"))
    }
}

#[derive(Deserialize)]
struct AqlResult {
    results: Vec<FileInfo>,
}

#[derive(Deserialize)]
struct FileInfo {
    path: String,
    #[allow(unused)]
    name: String,
}

struct ArtifactoryMetadataRetriever {
    server: String,
    repo: String,
    path: String,
    token: Option<String>,
}

impl ArtifactoryMetadataRetriever {
    fn new(server: &str, repo: &str, path: &str, token: Option<String>) -> Self {
        Self { server: server.to_string(), repo: repo.to_string(), path: path.to_string(), token }
    }
}

impl MetadataRetriever for ArtifactoryMetadataRetriever {
    fn fetch_versions(&self) -> Result<Vec<String>, Error> {
        let query = format!(
            r#"items.find({{
                "repo": "{}",
                "path": {{ "$match": "{}/*" }},
                "name": "manifest.toml"
            }})"#,
            self.repo,
            self.path
        );

        let url = format!("{}/api/search/aql", self.server);

        let mut request = ureq::post(&url);
        if let Some(token) = self.token.as_ref() {
            request = request.set("Authorization", format!("Bearer {token}").as_str());
        }

        let response: AqlResult = request
            .send_string(&query)?
            .into_json()?;

        let mut versions = Vec::new();

        for file in response.results {
            if let Some(version_str) = file.path.split('/').last() {
                versions.push(version_str.to_string());
            }
        }

        Ok(versions)
    }

    fn fetch_package_manifest(&self, version: &str) -> Result<Value, Error> {
        let url = format!(
            "{}/{}/{}/{}/manifest.toml",
            self.server,
            self.repo,
            self.path,
            version
        );

        let mut request = ureq::get(&url);
        if let Some(token) = self.token.as_ref() {
            request = request.set("Authorization", format!("Bearer {token}").as_str());
        }

        let response = request.call()?;

        if response.status() != 200 {
            return Err(anyhow::anyhow!("Server returned code {}: {}", response.status(), response.into_string()?));
        }

        let manifest_contents = response.into_string()?;
        toml::from_str(manifest_contents.as_str()).map_err(|e| anyhow::anyhow!("Failed to parse manifest: {e}"))
    }
}

pub struct Inventory {
    index: Index,
    pool: Rc<Pool<VersionReq>>,
    index_cache: IndexMap<String, IndexMap<Version, Package>>,
    cache_path: PathBuf,
    git_cache_path: PathBuf,
    artifactory_token: Option<String>
}

impl Inventory {
    pub fn new(index_url: &str, cache_path: &Path, artifactory_token: &Option<String>) -> Result<Self, Error> {
        let index_cache_path = cache_path.join("index");
        std::fs::create_dir_all(&index_cache_path)?;
        let index = Index::new(
            index_url,
            &RefType::Branch(String::from("main")),
            &index_cache_path
        )?;

        let git_cache_path = cache_path.join("git");
        std::fs::create_dir_all(&git_cache_path)?;

        Ok(Inventory {
            index,
            pool: Rc::new(Pool::new()),
            index_cache: IndexMap::new(),
            cache_path: cache_path.to_path_buf(),
            git_cache_path,
            artifactory_token: artifactory_token.clone()
        })
    }

    fn make_metadata_retriever(&self, name: &str, location_info: &LocationInfo) -> Result<Box<dyn MetadataRetriever>, Error> {
        match location_info {
            LocationInfo::Git(url) => {
                Ok(Box::new(GitMetadataRetriever::new(name, url, self.git_cache_path.as_path())?))
            },
            LocationInfo::Artifactory { server, repo, path } => {
                Ok(Box::new(ArtifactoryMetadataRetriever::new(server, repo, path, self.artifactory_token.clone())))
            }
        }
    }

    pub fn pool(&self) -> Rc<Pool<VersionReq>> {
        self.pool.clone()
    }
    pub fn index(&self) -> &Index { &self.index }

    pub fn get_package(&self, name: &str, version: &semver::Version) -> Result<&Package, Error> {
        let package_entry = self.index_cache.get(name).expect("Dependency '{name}' not found");
        let version_entry= package_entry.get(&Version { 0: version.clone() }).expect("Dependency '{name}/{version}' not found");
        Ok(version_entry)
    }

    pub fn update_cache(&mut self) -> Result<(), Error> {
        let cache_file = self.cache_path.join("cache.json");
        let cache_contents = std::fs::read_to_string(&cache_file).unwrap_or_default();
        if cache_contents.len() > 0 {
            self.index_cache = serde_json::from_str(&cache_contents)?;
        }

        for module in self.index.get_packages()? {
            let location_info = self.index.get_package_location(&module)?;
            println!("Fetching module {module}");
            let metadata_retriever = self.make_metadata_retriever(module, location_info)?;
            let versions = metadata_retriever.fetch_versions()?;
            for ref version in versions {
                println!("Trying version {version}");
                let download_manifest = match self.index_cache.get(module) {
                    Some(inner_map) => !inner_map.contains_key(&Version::from_str(&version)?),
                    None => true,
                };
                if download_manifest {
                    let manifest = metadata_retriever.fetch_package_manifest(version)?;
                    let module_entry = self
                        .index_cache
                        .entry(module.to_string())
                        .or_insert_with(IndexMap::new);
                    let version_entry = module_entry
                        .entry(Version::from_str(&version)?)
                        .or_insert_with(|| Package::new(module, version));
                    if let Some(dependencies) = manifest.get("dependencies") {
                        for (dep_name, dep_version) in dependencies.as_table().unwrap() {
                            version_entry.add_dependency(
                                dep_name,
                                dep_version.as_str().unwrap()
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

    pub fn add_package(&mut self, package: Package) {
        let entry = self
            .index_cache
            .entry(package.name.to_string())
            .or_insert_with(|| IndexMap::new());
        entry.insert(package.version.clone(), package);
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
            .and_then(|p| Some(&p.dependencies))
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
