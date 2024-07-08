use crate::repository::{BareRepository, RefType};
use anyhow::Error;
use indexmap::IndexMap;
use log::debug;
use std::path::Path;
use yaml_rust::{YamlLoader, Yaml};

pub enum LocationInfo {
    Git(String),
    Artifactory {
        server: String,
        repo: String,
        path: String
    },
}

pub struct Index {
    data: IndexMap<String, LocationInfo>,
}

impl Index {
    pub fn new(url: &str, ref_type: &RefType, path: &Path) -> Result<Self, Error> {
        let repo = BareRepository::new(url, Some(path))?;

        let mut data: IndexMap<String, LocationInfo> = IndexMap::new();
        let index_contents = repo.get_file(&ref_type, Path::new("index.yaml"))?;
        let index = YamlLoader::load_from_str(&String::from_utf8_lossy(&index_contents))?;
        let doc = &index[0];
        for (k, v) in doc
            .as_hash()
            .ok_or(anyhow::anyhow!("Failed to load module index"))?
        {
            debug!("Processing index entry for {}", k.as_str().unwrap());
            let name = k.as_str().expect("Invalid key");
            let reference = if let Some(m) = v.as_hash() {
                let server = m[&Yaml::from_str("server")].as_str().expect("Expected 'server' key").to_string();
                let repo = m[&Yaml::from_str("repo")].as_str().expect("Expected 'repo' key").to_string();
                let path = m[&Yaml::from_str("path")].as_str().expect("Expected 'path' key").to_string();
                debug!("Artifactory source {},{},{}", server, repo, path);
                LocationInfo::Artifactory {
                    server,
                    repo,
                    path
                }
            } else {
                let url = v.as_str().expect("Expected a string value").to_string();
                debug!("Git source {}", url);
                LocationInfo::Git(url)
            };
            data.insert(name.to_string(), reference);
        }

        Ok(Self { data })
    }

    pub fn get_packages(&self) -> Result<Vec<&str>, Error> {
        Ok(self.data.keys().map(|k| k.as_str()).collect())
    }

    pub fn get_package_location(&self, package_name: &str) -> Result<&LocationInfo, Error> {
        self.data
            .get(package_name)
            .ok_or(anyhow::anyhow!("Package not found ({package_name})"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_index() {
        let temp_dir = tempdir().unwrap();
        let index = Index::new(
            "https://github.com/jasal82/index.git",
            &RefType::Branch(String::from("main")),
            temp_dir.path(),
        )
        .unwrap();
        println!("modules: {:?}", index.get_packages().unwrap());
        assert_eq!(
            index.get_module_url("module1").unwrap(),
            "https://github.com/jasal82/module1.git"
        );
        assert_eq!(
            index.get_module_url("module2").unwrap(),
            "https://github.com/jasal82/module2.git"
        );
    }
}
