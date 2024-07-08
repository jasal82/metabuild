use anyhow::Error;
use git2::{AutotagOption, Repository};
use git2::build::RepoBuilder;
use log::debug;
use std::path::Path;
use tempfile::TempDir;
use metabuild_git::*;

pub struct BareRepository {
    repo: Repository,
    #[allow(unused)] // this field is used to control the lifetime of the tempdir
    temp_dir: Option<TempDir>,
}

pub enum RefType {
    Tag(String),
    Branch(String),
}

impl BareRepository {
    pub fn new(url: &str, cache_path: Option<&Path>) -> Result<Self, Error> {
        let mut temp_dir = None;
        let path = cache_path.unwrap_or_else(|| {
            temp_dir = Some(tempfile::tempdir().expect("Failed to create temp dir"));
            temp_dir.as_ref().unwrap().path()
        });

        debug!("Preparing local working copy for repository url {url}");

        let repo = match Repository::open_bare(&path) {
            Ok(repo) => {
                debug!("Opening existing bare repository at {:?}", path);
                let mut remote = repo.find_remote("origin").unwrap();
                if remote.url().unwrap() == url {
                    let git_config = make_git_config()?;
                    let auth = make_git_authenticator();
                    let mut fetch_options = make_fetch_options(&auth, &git_config);
                    fetch_options.download_tags(AutotagOption::All);
                    remote.fetch(&["refs/heads/*:refs/heads/*"], Some(&mut fetch_options), None)?;
                    drop(remote);
                    repo
                } else {
                    // Remote URL has changed, need to delete and re-clone
                    debug!("Remote url has changed, deleting local working copy");
                    drop(remote);
                    drop(repo);
                    std::fs::remove_dir_all(&path)?;
                    debug!("Cloning new bare repository into {:?}", path);
                    BareRepository::clone(url, path)?
                }
            }
            Err(_) => {
                debug!("Cloning new bare repository into {:?}", path);
                BareRepository::clone(url, path)?
            }
        };

        Ok(Self { repo, temp_dir })
    }

    fn clone(url: &str, path: &Path) -> Result<Repository, Error> {
        std::fs::create_dir_all(&path)?;
        let git_config = make_git_config()?;
        let auth = make_git_authenticator();
        let fetch_options = make_fetch_options(&auth, &git_config);
        let mut repo_builder = RepoBuilder::new();
        repo_builder.fetch_options(fetch_options);
        repo_builder.bare(true).clone(url, &path).map_err(anyhow::Error::from)
    }

    pub fn get_tags(&self) -> Result<Vec<String>, Error> {
        let mut tags = Vec::new();
        self.repo
            .tag_names(None)?
            .iter()
            .filter_map(|s| s)
            .for_each(|s| tags.push(s.to_string()));
        Ok(tags)
    }

    pub fn get_file(&self, ref_type: &RefType, path: &Path) -> Result<Vec<u8>, Error> {
        let reference = match ref_type {
            RefType::Tag(tag) => format!("refs/tags/{}", tag),
            RefType::Branch(branch) => format!("refs/heads/{}", branch),
        };

        let tree_id = self.repo.revparse_single(&reference)?.peel_to_tree()?.id();
        let tree = self.repo.find_tree(tree_id)?;
        let path = Path::new(path);
        let entry = tree.get_path(path)?;
        let blob = self.repo.find_blob(entry.id())?;
        Ok(blob.content().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::{BareRepository, RefType};
    use anyhow::Error;
    use std::path::Path;

    #[test]
    fn test_get_tags() -> Result<(), Error> {
        let repo = BareRepository::new("https://github.com/jasal82/module2.git", None)?;
        let tags = repo.get_tags()?;
        assert!(tags.contains(&String::from("2.0.1")));
        Ok(())
    }

    #[test]
    fn test_get_file() -> Result<(), Error> {
        let repo = BareRepository::new("https://github.com/jasal82/cconfig.git", None)?;
        let contents = repo.get_file(
            &RefType::Branch(String::from("master")),
            Path::new("CMakeLists.txt"),
        )?;
        let contents_string = String::from_utf8_lossy(&contents);
        assert!(contents_string.contains("project(CCONFIG)"));
        Ok(())
    }
}
