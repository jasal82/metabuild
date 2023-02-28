use clap::{CommandFactory, Parser};
use toml::Table;

mod cli;
mod commands;
mod git;
mod net;
mod scripting;

use cli::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_header() {
    println!(r#"               __       __        _ __   __"#);
    println!(r#"    __ _  ___ / /____ _/ /  __ __(_) /__/ /"#);
    println!(r#"   /  ' \/ -_) __/ _ `/ _ \/ // / / / _  / "#);
    println!(r#"  /_/_/_/\__/\__/\_,_/_.__/\_,_/_/_/\_,_/  "#);
    println!();
    println!(r#"  metabuild v{VERSION} - Build automation tool"#);
    println!(r#"  Copyright (C) 2023 Johannes Asal"#);
    println!();
}

fn parse_config() -> toml::Table {
    let content = std::fs::read_to_string("resources/dependencies.toml").unwrap();
    content.parse::<Table>().unwrap()
}

fn to_scope(global: bool) -> commands::config::ConfigScope {
    if global {
        commands::config::ConfigScope::Global
    } else {
        commands::config::ConfigScope::Local
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let cli = Cli::parse();
    let mut config = commands::config::Config::new();

    match &cli.command {
        Commands::Install { username, password } => {
            let dependency_config = parse_config();
            commands::install::install_script_modules(&dependency_config, &username, &password);
            commands::install::install_executables(&dependency_config);
            Ok(())
        },
        Commands::Run { tasks } => {
            if let Err(e) = scripting::run_tasks(tasks) {
                Cli::command().print_help().unwrap();
                Err(e)
            } else {
                Ok(())
            }
        },
        Commands::Update => { commands::update::update() },
        Commands::Config { command } => match &command {
            ConfigCommands::Set { key, value, global } => {
                config.set(key, value, to_scope(*global))
            },
            ConfigCommands::Get { key } => {
                config.get(key)
            },
            ConfigCommands::List => {
                config.list()
            },
        }
    }
}