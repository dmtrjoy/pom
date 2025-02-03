mod cli;
mod database;
mod quest;
mod table;

use clap::Parser;
use cli::{Args, Cli};

fn main() {
    // Parse and interpret the command line arguments.
    let args = Args::parse();
    Cli::interpret(args);
}
