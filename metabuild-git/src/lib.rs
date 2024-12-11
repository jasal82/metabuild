use auth_git2::GitAuthenticator;
use git2::{Config, FetchOptions, PushOptions, RemoteCallbacks};

pub fn make_git_config() -> Result<Config, git2::Error> {
    Config::new()
}

pub fn make_git_authenticator() -> GitAuthenticator {
    GitAuthenticator::default()
}

pub fn make_fetch_options<'a>(auth: &'a GitAuthenticator, config: &'a Config) -> FetchOptions<'a> {
    let mut fetch_options = FetchOptions::new();
    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.credentials(auth.credentials(&config));
    fetch_options.remote_callbacks(remote_callbacks);
    fetch_options
}

pub fn make_push_options<'a>(auth: &'a GitAuthenticator, config: &'a Config) -> PushOptions<'a> {
    let mut push_options = PushOptions::new();
    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.credentials(auth.credentials(&config));
    push_options.remote_callbacks(remote_callbacks);
    push_options
}