use git2::{build::RepoBuilder, Config, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::Path;

pub fn clone(
    url: &str,
    dst: &Path,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<Repository, Box<dyn std::error::Error>> {
    let config = Config::open_default()?;
    let mut repo_builder = RepoBuilder::new();
    let mut callbacks = RemoteCallbacks::new();
    if username.is_some() && password.is_some() {
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(username.as_ref().unwrap(), password.as_ref().unwrap())
        });
    } else {
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::credential_helper(&config, url, username.or(username_from_url))
        });
    }
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    repo_builder.fetch_options(fetch_options);
    Ok(repo_builder.clone(url, dst)?)
}

pub fn checkout_ref(repo: &Repository, refname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (object, reference) = repo.revparse_ext(refname)?;
    repo.checkout_tree(&object, None)?;
    match reference {
        Some(gref) => repo.set_head(gref.name().expect("Invalid reference name")),
        None => repo.set_head_detached(object.id()),
    }?;
    Ok(())
}
