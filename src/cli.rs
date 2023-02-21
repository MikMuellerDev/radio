use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// The path where the configuration file should be located
    #[clap(short, long, value_parser)]
    pub config_path: Option<String>,

    /// Subcommands
    #[clap(subcommand)]
    pub subcommand: Command,
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum Command {
    /// Run the server
    Run,
    /// Create / read config and settings files
    Files,
}
