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
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Interact with the index
    Index {
        #[command(subcommand)]
        command: IndexCommands,
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
pub enum IndexCommands {
    /// Add a git source to the index
    AddGit {
        /// Name
        name: String,
        /// Url (SSH format)
        url: String,
        /// Index repository Url
        #[arg(short, long)]
        index: Option<String>,
    },
    /// Add an artifactory source to the index
    AddArtifactory {
        /// Name
        name: String,
        /// Server Url
        server: String,
        /// Repository
        repo: String,
        /// Path
        path: String,
        /// Index repository Url
        #[arg(short, long)]
        index: Option<String>,
    },
    /// Remove a source from the index
    Remove {
        /// Name
        name: String,
        /// Index repository Url
        #[arg(short, long)]
        index: Option<String>,
    },
    /// Revert the local index changes
    Revert {
        /// Index repository Url
        #[arg(short, long)]
        index: Option<String>,
    },
    /// Push the local index changes to the upstream repository
    Push {
        /// Index repository Url
        #[arg(short, long)]
        index: Option<String>,
    },
    /// List all sources in the index
    List {
        /// Index repository Url
        #[arg(short, long)]
        index: Option<String>,
    }
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration name
        key: String,
        /// Configuration value
        value: String,
        /// Set in local configuration file instead of global one
        #[arg(short, long, default_value = "false")]
        local: bool,
    },
    SetToken {
        /// Artifactory server name
        server: String,
        /// Artifactory token
        token: String,
        /// Set in local configuration file instead of global one
        #[arg(short, long, default_value = "false")]
        local: bool,
    },
    /// Retrieve a configuration value
    Get {
        /// Configuration name
        key: String,
    },
    GetToken {
        /// Artifactory server name
        server: String,
    },
    /// Remove a configuration value
    Remove {
        /// Configuration name
        key: String,
        /// Set in local configuration file instead of global one
        #[arg(short, long, default_value = "false")]
        local: bool,
    },
    RemoveToken {
        /// Artifactory server name
        server: String,
        /// Set in local configuration file instead of global one
        #[arg(short, long, default_value = "false")]
        local: bool,
    },
    /// Display all set configuration values
    Show,
    /// List all available configuration names
    List,
}
