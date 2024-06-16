use crate::commands::config::ConfigData;
use crate::git;
use anyhow::Error;
use flate2::read::GzDecoder;
use metabuild_resolver::{inventory::Inventory, index::LocationInfo, solve};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;
use tar::Archive;
use tempfile::TempDir;
use url::Url;

struct GitInstaller;
impl GitInstaller {
    fn install(&self, url: &str, name: &str, version: &str, target_path: &Path) -> Result<(), Error> {
        let target_path = target_path.join(name);
        let repo = git::clone(url, target_path.as_path())?;
        git::checkout(&repo, version.to_string().as_str())
    }
}

struct ArtifactoryInstaller;
impl ArtifactoryInstaller {
    fn download_file(url: &str, target_path: &Path) -> Result<(), Error> {
        let agent = ureq::AgentBuilder::new().build();
        let response = match agent.get(url).call() {
            Ok(response) => response,
            Err(ureq::Error::Status(code, response)) => {
                return Err(anyhow::anyhow!("Server returned code {}: {}", code, response.status_text()));
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Unknown error"));
            }
        };

        let mut output_file = File::create(target_path)?;
        match std::io::copy(&mut response.into_reader(), &mut output_file) {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(anyhow::anyhow!("Could not write to file: {e}"));
            }
        }
    }

    fn install(&self, server: &str, repo: &str, path: &str, name: &str, version: &str, target_path: &Path) -> Result<(), Error> {
        std::fs::create_dir_all(target_path.join(name))?;

        // Fetch manifest first
        let url = Url::parse(format!("{server}/{repo}/{path}/{version}/manifest.toml").as_str())?;
        let output_path = target_path.join(name).join("manifest.toml");
        Self::download_file(url.as_str(), output_path.as_path())?;

        // Then fetch the package tarball
        let url = Url::parse(format!("{server}/{repo}/{path}/{version}/package.tar.gz").as_str())?;
        let t = TempDir::new()?;
        let output_path = t.path().join("package.tar.gz");
        Self::download_file(url.as_str(), output_path.as_path())?;

        // Unpack the tarball
        let archive_file = File::open(&output_path)?;
        let target_path = target_path.join(name);
        let decoder = GzDecoder::new(archive_file);
        let mut archive = Archive::new(decoder);
        archive.unpack(target_path).map_err(|e| anyhow::anyhow!("Failed to unpack archive: {e}"))
    }
}

fn clear_or_create_directory(path: &Path) -> io::Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }
    std::fs::create_dir_all(&path)
}

fn get_index_url(config: &ConfigData, manifest: &toml::Table) -> Result<String, Error> {
    let url = manifest.get("registries")
        .and_then(toml::Value::as_table)
        .and_then(|registries| registries.get("default"))
        .and_then(toml::Value::as_str)
        .or(config.index_url.as_ref().map(String::as_str))
        .expect("No index URL specified in project or global config")
        .to_string();

    if url.starts_with("http") {
        return Err(anyhow::anyhow!("HTTP(S) index URLs are currently not supported"));
    }

    Ok(url)
}

fn parse_dependencies(manifest: &toml::Table) -> Result<HashMap<String, semver::VersionReq>, Error> {
    let dependency_table = manifest.get("dependencies")
        .and_then(toml::Value::as_table);

    let mut map = HashMap::new();
    if let Some(dependency_table) = dependency_table {
        for (k, v) in dependency_table {
            map.insert(k.to_string(), semver::VersionReq::parse(v.as_str().unwrap())?);
        }
    }

    Ok(map)
}

pub fn install_dependencies(
    config: &ConfigData,
    manifest: &toml::Table,
) -> Result<(), Error> {
    // Clear module installation path if it exists or create it
    let dependencies_path = Path::new(".mb").join("deps");
    clear_or_create_directory(dependencies_path.as_path())?;

    let index_url = get_index_url(config, manifest)?;
    println!("Using index {index_url}");

    let dependencies = parse_dependencies(manifest)?;

    println!("Updating cache...");
    let mut inventory = Inventory::new(&index_url, Path::new(".mb").join("cache").as_path(), &config.artifactory_token)?;
    inventory.update_cache()?;

    println!("Resolving dependencies...");
    match solve(&inventory, dependencies) {
        Ok(result) => {
            println!("Installing dependencies...");
            let git_installer = GitInstaller {};
            let artifactory_installer = ArtifactoryInstaller {};
            for (dep_name, dep_version) in result {
                let dep_location = inventory.index().get_package_location(&dep_name)?;
                let source = match dep_location {
                    LocationInfo::Git(_) => "Git",
                    LocationInfo::Artifactory {..} => "Artifactory",
                };

                println!("  {dep_name}/{dep_version} (from {source})");

                match dep_location {
                    LocationInfo::Git(url) => {
                        git_installer.install(url, dep_name.as_str(), dep_version.to_string().as_str(), dependencies_path.as_path())?;
                    },
                    LocationInfo::Artifactory { server, repo, path } => {
                        artifactory_installer.install(server, repo, path, dep_name.as_str(), dep_version.to_string().as_str(), dependencies_path.as_path())?;
                    }
                }
            }

            Ok(())
        },
        Err(metabuild_resolver::SolverError::Unsolvable(reason)) => {
            println!("{}", reason);
            Err(anyhow::anyhow!("Could not resolve dependencies"))
        },
        Err(metabuild_resolver::SolverError::Cancelled) => {
            Err(anyhow::anyhow!("Resolving was cancelled"))
        }
    }
}
