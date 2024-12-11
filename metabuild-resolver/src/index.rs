use crate::repository::{BareRepository, RefType};
use anyhow::Error;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Entry {
    Git {
        url: String,
    },
    Artifactory {
        server: String,
        repo: String,
        path: String,
    }
}

pub struct Index {
    repo: BareRepository,
    reftype: RefType,
    data: IndexMap<String, Entry>,
}

impl Index {
    pub fn new(url: &str, branch: &str, storage_path: &Path) -> Result<Self, Error> {
        let repo = BareRepository::new(url, Some(storage_path))?;
        let reftype = RefType::Branch(branch.to_string());
        let index_contents = repo.get_file(&reftype, Path::new("index.json"))?;
        let data = serde_json::from_str(&String::from_utf8_lossy(&index_contents))?;
        Ok(Self { repo, reftype, data })
    }

    pub fn get_entries(&self) -> Result<Vec<&str>, Error> {
        Ok(self.data.keys().map(|k| k.as_str()).collect())
    }

    pub fn get_entry(&self, name: &str) -> Result<&Entry, Error> {
        self.data
            .get(name)
            .ok_or(anyhow::anyhow!("No index entry found for '{name}'"))
    }

    pub fn add_entry(&mut self, name: &str, entry: Entry) -> Result<(), Error> {
        self.data.insert(name.to_string(), entry);
        self.save_index()
    }

    pub fn remove_entry(&mut self, name: &str) -> Result<(), Error> {
        self.data.shift_remove(name);
        self.save_index()
    }

    pub fn revert(&mut self) -> Result<(), Error> {
        self.repo.revert(&self.reftype)
    }

    pub fn push(&self) -> Result<(), Error> {
        self.repo.push(&self.reftype)
    }

    fn save_index(&self) -> Result<(), Error> {
        let index_contents = serde_json::to_string_pretty(&self.data)?;
        self.repo.update_file_and_commit(&self.reftype, "index.json", index_contents.as_bytes(), "Update index").map(|_| ())
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
