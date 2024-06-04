use crate::commands::config::ConfigData;
use crate::git;
use crate::net;
use anyhow::Error;
use metabuild_resolver::{inventory::Inventory, solve};
use std::collections::HashMap;
use std::path::Path;

fn parse_dependency_table(dependencies: &toml::Table) -> Result<HashMap<String, semver::VersionReq>, Error> {
    let mut map = HashMap::new();
    for (k, v) in dependencies {
        map.insert(k.to_string(), semver::VersionReq::parse(v.as_str().unwrap())?);
    }
    Ok(map)
}

fn install_module(module_base_path: &Path, name: &str, url: &str, version: &semver::Version) -> Result<(), Error> {
    let module_path = module_base_path.join(name);
    let repo = git::clone(url, module_path.as_path())?;
    git::checkout(&repo, version.to_string().as_str())
}

pub fn install_script_modules(
    config: &ConfigData,
    manifest: &toml::Table,
) -> Result<(), Error> {
    let module_base_path = Path::new(".mb").join("modules");
    if module_base_path.exists() {
        std::fs::remove_dir_all(&module_base_path)?;
    }
    std::fs::create_dir_all(&module_base_path)?;

    let index_url = manifest.get("registries")
        .and_then(toml::Value::as_table)
        .and_then(|registries| registries.get("default"))
        .and_then(toml::Value::as_str)
        .or(config.index_url.as_ref().map(String::as_str))
        .expect("No index URL specified in project or global config");

    if index_url.starts_with("http") {
        return Err(anyhow::anyhow!("HTTP(S) index URLs are currently not supported"));
    }

    let dependency_table = manifest.get("dependencies")
        .and_then(toml::Value::as_table);

    if let Some(dependency_table) = dependency_table {
        println!("Using index {index_url}");
        println!("Updating list of available packages");
        let mut inventory = Inventory::new(index_url, Path::new(".mb").join("cache").as_path())?;
        inventory.update_cache()?;
        let dependencies = parse_dependency_table(dependency_table)?;
        println!("Installing dependencies");
        match solve(&inventory, dependencies) {
            Ok(result) => {
                for (mod_name, mod_version) in result {
                    println!("  [*] {mod_name}/{mod_version}");
                    let mod_url = inventory.index().get_url(&mod_name)?;
                    install_module(module_base_path.as_path(), mod_name.as_str(), mod_url, &mod_version)?;
                }
                Ok(())
            },
            Err(metabuild_resolver::SolverError::Unsolvable(reason)) => {
                println!("{}", reason);
                Err(anyhow::anyhow!("Cannot resolve script module dependencies"))
            },
            Err(metabuild_resolver::SolverError::Cancelled) => {
                Err(anyhow::anyhow!("Solver was cancelled"))
            }
        }
    } else {
        println!("No dependencies specified");
        Ok(())
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
        println!("Installing executables");
        install_os_executables(config);
    }
}
