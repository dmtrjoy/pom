mod cli;
mod color;
mod database;
mod project;
mod session;
mod task;

use clap::Parser;
use cli::{Args, Cli};

fn main() {
    // Parse and interpret the command line arguments.
    let args = Args::parse();
    Cli::interpret(args);
}
