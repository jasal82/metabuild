use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Install {
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long)]
        password: Option<String>
    },
    Run {
        tasks: Vec<String>,
        #[arg(short, long)]
        file: Option<PathBuf>,
        #[arg(short, long, default_value = "false")]
        warn: bool
    },
    Update,
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Set {
        key: String,
        value: String,
        #[arg(short, long, default_value = "false")]
        global: bool,
    },
    Get {
        key: String,
    },
    List,
}