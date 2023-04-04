use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::Path;

pub fn clone(
    url: &str,
    dst: &Path,
    username: &Option<String>,
    password: &Option<String>,
) -> Result<Repository, Box<dyn std::error::Error>> {
    let mut repo_builder = RepoBuilder::new();
    if username.is_some() && password.is_some() {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(username.as_ref().unwrap(), password.as_ref().unwrap())
        });
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        repo_builder.fetch_options(fetch_options);
    }
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
