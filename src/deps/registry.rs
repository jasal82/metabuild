use crypto_hash::{Algorithm, Hasher};
use git2::Repository;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::git;

pub struct Registry {
    url: String,
    repo: Repository,
}

impl Registry {
    pub fn new(url: &str, cache_base: &Path) -> Self {
        Self {
            url: url.to_string(),
            repo: Self::open_or_clone(url, cache_base).unwrap(),
        }
    }

    fn get_cache_path(cache_base: &Path, url: &str) -> Result<PathBuf, anyhow::Error> {
        let mut path = cache_base.to_path_buf();

        let hash = {
            let mut hasher = Hasher::new(Algorithm::SHA256);

            hasher.write(url.as_bytes()).unwrap();

            hasher
                .finish()
                .into_iter()
                .fold(String::new(), |s: String, i| {
                    s + format!("{:02x}", i).as_str()
                })
        };

        path.push(hash);
        Ok(path)
    }

    fn open_or_clone(url: &str, cache_base: &Path) -> Result<Repository, anyhow::Error> {
        let path = Self::get_cache_path(cache_base, url)?;
        let repo = if path.exists() {
            Repository::open(path)?
        } else {
            std::fs::create_dir_all(&path)?;
            git::clone(url, &path, None, None)
                .map_err(|e| anyhow::anyhow!("Failed to clone repository '{}': {}", url, e))?
        };
        Ok(repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_path() {
        let path = Registry::get_cache_path(
            PathBuf::from(".mb/cache").as_path(),
            "https://github.com/jasal82/metabuild",
        )
        .unwrap();
        let mut it = path.iter();
        assert_eq!(it.next(), Some(std::ffi::OsStr::new(".mb")));
        assert_eq!(it.next(), Some(std::ffi::OsStr::new("cache")));
        assert_eq!(
            it.next(),
            Some(std::ffi::OsStr::new(
                "be3f0241bdb54e48668999e56314d8c286e436f6f523c2584795706511d9179b"
            ))
        );
    }

    #[test]
    fn test_open_or_clone() {
        let temp_dir = tempfile::tempdir().unwrap();
        let _repo =
            Registry::open_or_clone("https://github.com/jasal82/metabuild", temp_dir.path())
                .unwrap();
        assert!(Path::new(temp_dir.path())
            .join("be3f0241bdb54e48668999e56314d8c286e436f6f523c2584795706511d9179b")
            .join("Cargo.toml")
            .exists());
    }
}
