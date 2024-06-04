use crate::repository::{BareRepository, RefType};
use anyhow::Error;
use indexmap::IndexMap;
use std::path::Path;
use yaml_rust::YamlLoader;

pub struct Index {
    data: IndexMap<String, String>,
}

impl Index {
    pub fn new(url: &str, ref_type: &RefType, path: &Path) -> Result<Self, Error> {
        let mut map: IndexMap<String, String> = IndexMap::new();
        let repo = BareRepository::new(url, Some(path))?;
        let index_contents = repo.get_file(&ref_type, Path::new("index.yaml"))?;
        let index = YamlLoader::load_from_str(&String::from_utf8_lossy(&index_contents))?;
        let doc = &index[0];
        for (k, v) in doc
            .as_hash()
            .ok_or(anyhow::anyhow!("Failed to load index"))?
        {
            let name = k.as_str().unwrap();
            let url = v.as_str().unwrap();
            map.insert(name.to_string(), url.to_string());
        }
        Ok(Self { data: map })
    }

    pub fn get_modules(&self) -> Result<Vec<&str>, Error> {
        Ok(self.data.keys().map(|k| k.as_str()).collect())
    }

    pub fn get_url(&self, module_name: &str) -> Result<&String, Error> {
        self.data
            .get(module_name)
            .ok_or(anyhow::anyhow!("Module not found ({module_name})"))
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
        println!("modules: {:?}", index.get_modules().unwrap());
        assert_eq!(
            index.get_url("module1").unwrap(),
            "https://github.com/jasal82/module1.git"
        );
        assert_eq!(
            index.get_url("module2").unwrap(),
            "https://github.com/jasal82/module2.git"
        );
    }
}
