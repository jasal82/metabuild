use std::collections::HashMap;
#[cfg(unix)]
use {std::fs::Permissions, std::os::unix::fs::PermissionsExt};

fn get_target() -> &'static str {
    env!("TARGET")
}

pub fn pinned_version() -> Option<semver::Version> {
    let mut path = std::env::current_dir().unwrap();
    loop {
        let version_file = path.join(".mb-version");
        if version_file.exists() {
            let content = std::fs::read_to_string(&version_file).unwrap();
            match semver::Version::parse(&content.trim()) {
                Ok(version) => {
                    crate::logging::info(format!(
                        "Version pinned to {} by .mb-version file in {:?}",
                        version, path
                    ));
                    return Some(version);
                }
                Err(_e) => {
                    crate::logging::error(format!(
                        "Invalid version '{}' in .mb-version file {:?}",
                        content, path
                    ));
                    std::process::exit(1);
                }
            }
        }
        if !path.pop() {
            break;
        }
    }
    None
}

pub fn running_on_buildserver() -> bool {
    std::env::var("GITLAB_CI").is_ok()
}

pub fn download_and_run(version: &semver::Version) {
    crate::logging::info(format!("Downloading metabuild version {}", version));
    let mut path = std::env::current_dir().unwrap();
    path.push(".mb");
    path.push("bin");
    std::fs::create_dir_all(&path).unwrap();
    path.push("mb");
    if !path.exists() {
        let target = get_target();
        let mut suffix = "";
        if cfg!(windows) {
            suffix = ".exe";
            path.set_extension("exe");
        }
        let url = format!(
            r#"https://github.com/jasal82/metabuild/releases/download/v{version}/mb-v{version}-{target}{suffix}"#
        );
        let _ = crate::net::download_file(&url, &path, &HashMap::new());
        #[cfg(unix)]
        std::fs::set_permissions(&path, Permissions::from_mode(0o755)).unwrap();
    }

    crate::logging::info(format!("Running pinned metabuild version {}", version));
    let mut command = std::process::Command::new(&path);
    command.args(std::env::args().skip(1));
    let status = command.status().unwrap();
    std::process::exit(status.code().unwrap());
}
