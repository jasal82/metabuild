use anyhow::Error;
use clap::Parser;
use colored::Colorize;
use color_eyre::eyre::Result;
use commands::config::ConfigData;
use log::{error, info, debug};
use md5;
use metabuild_resolver::index::Index;
use std::panic;
use std::path::{Path, PathBuf};

mod cli;
mod commands;
mod git;
mod net;
mod pinning;
mod scripting;

use cli::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn parse_manifest(file: &Path) -> toml::Table {
    debug!("Reading manifest file {file:?}");
    let content = std::fs::read_to_string(file)
        .expect(format!("Could not read manifest file '{}'", file.display()).as_str());
    content
        .parse::<toml::Table>()
        .expect(format!("Could not parse manifest file '{}'", file.display()).as_str())
}

fn get_index_url(config: &ConfigData, manifest: &toml::Table) -> Result<String, Error> {
    let url = manifest.get("registries")
        .and_then(toml::Value::as_table)
        .and_then(|registries| registries.get("default"))
        .and_then(toml::Value::as_str)
        .or(config.index.as_ref().map(String::as_str))
        .expect("No index URL specified in project or global config")
        .to_string();

    if url.starts_with("http") {
        return Err(anyhow::anyhow!("HTTP(S) index URLs are currently not supported. Use SSH instead."));
    }

    Ok(url)
}

fn open_index(index_url: &str, index_path: &Path) -> Result<Index, Error> {
    let index = Index::new(
        index_url,
        "main",
        &index_path
    )?;
    
    Ok(index)
}

fn to_scope(local: bool) -> commands::config::ConfigScope {
    if local {
        commands::config::ConfigScope::Local
    } else {
        commands::config::ConfigScope::Global
    }
}

pub fn main() -> Result<(), anyhow::Error> {
    color_eyre::install().map_err(|_| anyhow::anyhow!("Failed to install color_eyre"))?;
    pretty_env_logger::init();
    panic::set_hook(Box::new(|panic_info| {
        error!("Panicked: {panic_info}");
    }));

    // Check if metabuild version is pinned
    if let Some(pinned_version) = pinning::pinned_version() {
        if let Ok(current_version) = semver::Version::parse(VERSION) {
            if current_version != pinned_version {
                if pinning::running_on_buildserver() {
                    info!("Running on buildserver, but pinned version is set. Aborting.");
                    info!("Please use the appropriate Docker image for this build.");
                    std::process::exit(1);
                } else {
                    pinning::download_and_run(&pinned_version)?;
                }
            }
        }
    }
    
    let cli = Cli::parse();
    let mut config_figment = commands::config::Config::new();
    let config = &config_figment.merged;

    let local_path = Path::new(".mb");
    
    let handle_index_command = |index: &Option<String>, f: &dyn Fn(&mut Index) -> Result<(), Error>| {
        if let Some(index_url) = index.as_ref().or(config.index.as_ref()) {
            let index_hash = format!("{:x}", md5::compute(&index_url));
            let index_path = local_path.join("index").join(index_hash);
            std::fs::create_dir_all(&index_path)?;
            let mut index = open_index(&index_url, &index_path)?;
            f(&mut index)
        } else {
            eprintln!("{}: {}", "Error".red().bold(), "No index configured and no --index parameter specified");
            std::process::exit(1);
        }
    };

    match &cli.command {
        Commands::Index { command } => match command {
            IndexCommands::AddGit { name, url, index } => {
                handle_index_command(index, &|index| commands::index::add_git(index, name, url))
            }
            IndexCommands::AddArtifactory { name, server, repo, path, index } => {
                handle_index_command(index, &|index| commands::index::add_artifactory(index, name, server, repo, path))
            }
            IndexCommands::Remove { name, index } => {
                handle_index_command(index, &|index| commands::index::remove(index, name))
            }
            IndexCommands::Push { index } => {
                handle_index_command(index, &|index| commands::index::push(index))
            }
            IndexCommands::Revert { index } => {
                handle_index_command(index, &|index| commands::index::revert(index))
            }
            IndexCommands::List { index } => {
                handle_index_command(index, &|index| commands::index::list(index))
            }
        },
        Commands::Install {
            file,
        } => {
            let manifest = parse_manifest(file.as_ref().unwrap_or(&PathBuf::from("manifest.toml")));
            let index_url: String = get_index_url(&config, &manifest)?;
            let index_hash = format!("{:x}", md5::compute(&index_url));
            let index_path = local_path.join("index").join(index_hash);
            let index = open_index(&index_url, &index_path)?;
            commands::install::install_dependencies(&index, &config, &manifest, local_path)
        }
        Commands::Run { file, args: _ } => {
            if let Err(e) = scripting::run_file(file.as_ref().unwrap_or(&PathBuf::from("main.koto"))) {
                Err(e)
            } else {
                Ok(())
            }
        }
        Commands::Update => {
            commands::update::update()
        },
        Commands::Config { command } => match command {
            ConfigCommands::Set { key, value, local } => config_figment.set(key, value, to_scope(*local)),
            ConfigCommands::Get { key } => {
                config_figment.get(key).map(|v| {
                    println!("{}", v);
                    ()
                })
            },
            ConfigCommands::Remove { key, local } => config_figment.remove(key, to_scope(*local)),
            ConfigCommands::Show => config_figment.show(),
            ConfigCommands::List => config_figment.list(),
            ConfigCommands::SetToken { server, token, local } => config_figment.set_token(server, token, to_scope(*local)),
            ConfigCommands::GetToken { server } => {
                config_figment.get_token(server).map(|v| {
                    println!("{}", v);
                    ()
                })
            },
            ConfigCommands::RemoveToken { server, local } => config_figment.remove_token(server, to_scope(*local)),
        },
    }
}
