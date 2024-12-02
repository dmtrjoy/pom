use clap::{Parser, Subcommand};

/// Create and manage projects, set timers, and more!
#[derive(Parser)]
#[command(version, long_about)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

impl Args {
    pub fn command(&self) -> Command {
        self.command.clone()
    }
}

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Add an item
    #[command(long_about)]
    Add { message: String },

    /// Create a new project
    #[command(long_about)]
    New { name: String },

    /// List all projects
    #[command(long_about)]
    Project,

    /// Switch projects
    #[command(long_about)]
    Switch { name: String },
}
