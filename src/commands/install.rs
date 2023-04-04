use crate::git;
use crate::net;
use std::collections::HashMap;
use std::path::Path;

pub fn install_script_modules(
    config: &toml::Table,
    username: &Option<String>,
    password: &Option<String>,
) {
    println!("Installing script-modules...");
    for (mod_pkg_name, v) in config["script-modules"].as_table().unwrap() {
        let entry = v.as_table().unwrap();
        let url = entry["repo"].as_str().unwrap();
        let refname = entry["ref"].as_str().unwrap();
        let mod_pkg_dir = Path::new(".mb/modules").join(mod_pkg_name);
        std::fs::remove_dir_all(&mod_pkg_dir).unwrap_or_else(|_| {
            panic!("Failed to clear mod package dir for module {mod_pkg_name}")
        });
        std::fs::create_dir_all(&mod_pkg_dir).unwrap_or_else(|_| {
            panic!("Failed to create mod package dir for module {mod_pkg_name}")
        });
        let repo = git::clone(url, &mod_pkg_dir, username, password).unwrap();
        git::checkout_ref(&repo, refname).unwrap_or_else(|_| panic!("Failed to change ref"));
    }
}

pub fn install_executables(config: &toml::Table) {
    println!("Installing executables...");
    for (name, url) in config["executables"]["windows"].as_table().unwrap() {
        let bin_dir = Path::new(".mb/bin");
        std::fs::create_dir_all(bin_dir).unwrap_or_else(|_| panic!("Failed to create bin dir"));
        #[cfg(windows)]
        let file_name = format!("{name}.exe");
        #[cfg(not(windows))]
        let file_name = name;
        net::download_file(
            url.as_str().unwrap(),
            &bin_dir.join(file_name),
            &HashMap::new(),
        )
        .unwrap_or_else(|_| panic!("Failed to download executable {name}"));
    }
}
