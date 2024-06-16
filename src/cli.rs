use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "metabuild - Build automation tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install dependencies from a manifest file
    Install {
        /// Manifest file (defaults to manifest.toml)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Run a metabuild script
    Run {
        /// Script file (defaults to main.koto)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Update metabuild
    Update,
    /// Interact with metabuild configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration name
        key: String,
        /// Configuration value
        value: String,
        /// Set in global configuration file instead of local one
        #[arg(short, long, default_value = "false")]
        global: bool,
    },
    /// Retrieve a configuration value
    Get {
        /// Configuration name
        key: String,
    },
    /// Remove a configuration value
    Remove {
        /// Configuration name
        key: String,
        /// Remove from global configuration file instead of local one
        #[arg(short, long, default_value = "false")]
        global: bool,
    },
    /// Display all set configuration values
    Show,
    /// List all available configuration names
    List,
}
