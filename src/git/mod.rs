use anyhow::Error;
use git2::{Repository, build::RepoBuilder};
use std::path::Path;
use metabuild_git::*;

pub fn clone(
    url: &str,
    dst: &Path,
) -> Result<Repository, Error> {
    let git_config = make_git_config()?;
    let auth = make_git_authenticator();
    let fetch_options = make_fetch_options(&auth, &git_config);
    let mut repo_builder = RepoBuilder::new();
    repo_builder.fetch_options(fetch_options);
    Ok(repo_builder.clone(url, dst)?)
}

pub fn checkout(
    repo: &Repository,
    ref_name: &str,
) -> Result<(), Error> {
    let (object, reference) = repo.revparse_ext(ref_name)?;
    repo.checkout_tree(&object, None)?;
    match reference {
        Some(gref) => repo.set_head(gref.name().unwrap())?,
        None => repo.set_head_detached(object.id())?,
    }
    Ok(())
}
