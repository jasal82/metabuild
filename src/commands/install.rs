use crate::git;
use crate::net;
use std::collections::HashMap;
use std::path::Path;

pub fn install_script_modules(
    config: &toml::Table,
    username: Option<&str>,
    password: Option<&str>,
) {
    println!("Installing script-modules...");
    for (mod_pkg_name, v) in config["script-modules"].as_table().unwrap() {
        println!("  [*] {mod_pkg_name}");
        let entry = v.as_table().unwrap();
        let url = entry["repo"].as_str().unwrap();
        let refname = entry["ref"].as_str().unwrap();
        let mod_pkg_dir = Path::new(".mb").join("modules").join(mod_pkg_name);
        if mod_pkg_dir.exists() {
            std::fs::remove_dir_all(mod_pkg_dir.as_path()).unwrap_or_else(|_| {
                panic!(
                    "Failed to clear mod package dir for module {}",
                    mod_pkg_name
                )
            });
        }
        std::fs::create_dir_all(mod_pkg_dir.as_path()).unwrap_or_else(|_| {
            panic!(
                "Failed to create mod package dir for module {}",
                mod_pkg_name
            )
        });
        let repo = git::clone(url, mod_pkg_dir.as_path(), username, password).unwrap();
        git::checkout_ref(&repo, refname).unwrap_or_else(|_| panic!("Failed to change ref"));
    }
}

#[cfg(windows)]
fn install_os_executables(config: &toml::Table) {
    if config["executables"].is_table()
        && config["executables"]
            .as_table()
            .unwrap()
            .contains_key("windows")
    {
        for (name, url) in config["executables"]["windows"].as_table().unwrap() {
            println!("  [*] {name}");
            let bin_dir = Path::new(".mb").join("bin");
            std::fs::create_dir_all(bin_dir.as_path())
                .unwrap_or_else(|_| panic!("Failed to create bin dir"));
            let file_name = format!("{name}.exe");
            net::download_file(
                url.as_str().unwrap(),
                &bin_dir.join(file_name),
                &HashMap::new(),
            )
            .unwrap_or_else(|_| panic!("Failed to download executable {}", name));
        }
    }
}

#[cfg(not(windows))]
fn install_os_executables(config: &toml::Table) {
    if config["executables"].is_table()
        && config["executables"]
            .as_table()
            .unwrap()
            .contains_key("linux")
    {
        for (name, url) in config["executables"]["linux"].as_table().unwrap() {
            println!("  [*] {name}");
            let bin_dir = Path::new(".mb").join("bin");
            std::fs::create_dir_all(bin_dir.as_path())
                .unwrap_or_else(|_| panic!("Failed to create bin dir"));
            net::download_file(url.as_str().unwrap(), &bin_dir.join(name), &HashMap::new())
                .unwrap_or_else(|_| panic!("Failed to download executable {}", name));
        }
    }
}

pub fn install_executables(config: &toml::Table) {
    if config.contains_key("executables") {
        println!("Installing executables...");
        install_os_executables(config);
    }
}
