use clap::Parser;
use std::panic;
use std::path::{Path, PathBuf};

mod cli;
mod commands;
mod git;
mod logging;
mod net;
mod pinning;
mod scripting;

use cli::*;

#[macro_use]
extern crate lazy_static;

const VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref TLS_CONFIG: rustls::ClientConfig = {
        // Create TLS config with root certificates; do this only once and pass the
        // client config down to the other components for performance reasons.
        let mut roots = rustls::RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
            roots.add(cert).unwrap();
        }

        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth()
    };
}

fn print_header() {
    println!(r#"               __       __        _ __   __"#);
    println!(r#"    __ _  ___ / /____ _/ /  __ __(_) /__/ /"#);
    println!(r#"   /  ' \/ -_) __/ _ `/ _ \/ // / / / _  / "#);
    println!(r#"  /_/_/_/\__/\__/\_,_/_.__/\_,_/_/_/\_,_/  "#);
    println!();
    println!(r#"  metabuild v{VERSION} - Build automation tool"#);
    println!(r#"  Copyright (c) 2023-2024 Johannes Asal"#);
    println!();
}

fn parse_manifest(file: &Path) -> toml::Table {
    let content = std::fs::read_to_string(file)
        .expect(format!("Could not read manifest file '{}'", file.display()).as_str());
    content
        .parse::<toml::Table>()
        .expect(format!("Could not parse manifest file '{}'", file.display()).as_str())
}

fn to_scope(global: bool) -> commands::config::ConfigScope {
    if global {
        commands::config::ConfigScope::Global
    } else {
        commands::config::ConfigScope::Local
    }
}

pub fn main() -> Result<(), anyhow::Error> {
    panic::set_hook(Box::new(|panic_info| {
        logging::error(panic_info.to_string());
    }));

    // Check if metabuild version is pinned
    if let Some(pinned_version) = pinning::pinned_version() {
        if let Ok(current_version) = semver::Version::parse(VERSION) {
            if current_version != pinned_version {
                if pinning::running_on_buildserver() {
                    logging::error("Running on buildserver, but pinned version is set. Aborting.");
                    println!("Please use the appropriate Docker image for this build.");
                    std::process::exit(1);
                } else {
                    pinning::download_and_run(&pinned_version)?;
                }
            }
        }
    }
    
    let cli = Cli::parse();
    let mut config = commands::config::Config::new();

    match &cli.command {
        Commands::Install {
            file,
        } => {
            print_header();
            let manifest = parse_manifest(file.as_ref().unwrap_or(&PathBuf::from("manifest.toml")));
            commands::install::install_script_modules(
                &config.merged,
                &manifest
            )?;
            commands::install::install_executables(&manifest);
            Ok(())
        }
        Commands::Run { file } => {
            print_header();
            if let Err(e) = scripting::run_file(file.as_ref().unwrap_or(&PathBuf::from("main.koto"))) {
                //Cli::command().print_help().unwrap();
                Err(e)
            } else {
                Ok(())
            }
        }
        Commands::Update => {
            print_header();
            commands::update::update()
        },
        Commands::Config { command } => match &command {
            ConfigCommands::Set { key, value, global } => config.set(key, value, to_scope(*global)),
            ConfigCommands::Get { key } => { config.get(key).map(|v| {
                    println!("{}", v);
                    ()
                })
            },
            ConfigCommands::Remove { key, global } => config.remove(key, to_scope(*global)),
            ConfigCommands::List => config.list(),
        },
    }
}
