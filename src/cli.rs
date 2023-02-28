use clap::{Parser, Subcommand};

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
    Run { tasks: Vec<String> },
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