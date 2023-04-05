use clap::{CommandFactory, Parser};
use std::panic;
use std::path::{Path, PathBuf};
use toml::Table;

mod cli;
mod commands;
mod git;
mod logging;
mod net;
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
            roots
                .add(&rustls::Certificate(cert.0))
                .unwrap();
        }

        rustls::ClientConfig::builder()
            .with_safe_defaults()
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
    println!(r#"  Copyright (c) 2023 Johannes Asal"#);
    println!();
}

fn parse_config(file: &Path) -> toml::Table {
    let content = std::fs::read_to_string(file).unwrap();
    content.parse::<Table>().unwrap()
}

fn to_scope(global: bool) -> commands::config::ConfigScope {
    if global {
        commands::config::ConfigScope::Global
    } else {
        commands::config::ConfigScope::Local
    }
}

pub fn main() -> Result<(), anyhow::Error> {
    print_header();

    panic::set_hook(Box::new(|panic_info| {
        logging::error(panic_info.to_string());
    }));

    let cli = Cli::parse();
    let mut config = commands::config::Config::new();

    match &cli.command {
        Commands::Install {
            file,
            username,
            password,
        } => {
            let dependency_config =
                parse_config(file.as_ref().unwrap_or(&PathBuf::from("dependencies.toml")));
            commands::install::install_script_modules(
                &dependency_config,
                username.as_deref(),
                password.as_deref(),
            );
            commands::install::install_executables(&dependency_config);
            Ok(())
        }
        Commands::Run { tasks, file, warn } => {
            if let Err(e) = scripting::run_tasks(
                file.as_ref().unwrap_or(&PathBuf::from("tasks.rn")),
                tasks,
                *warn,
            ) {
                Cli::command().print_help().unwrap();
                Err(e)
            } else {
                Ok(())
            }
        }
        Commands::Update => commands::update::update(),
        Commands::Config { command } => match &command {
            ConfigCommands::Set { key, value, global } => config.set(key, value, to_scope(*global)),
            ConfigCommands::Get { key } => config.get(key),
            ConfigCommands::List => config.list(),
        },
    }
}
