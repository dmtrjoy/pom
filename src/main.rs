mod cli;
mod database;
mod project;
mod session;
mod task;

use clap::Parser;
use cli::{Args, Cli};
use database::Database;

fn main() {
    // Open the database connection singleton.
    let database = Database::new();
    let conn = database.conn();

    // Load the session (active project).
    // let session_dao = SessionDao::new(&conn);
    // let session = session_dao.load();

    // Parse the command line arguments.
    let args = Args::parse();

    // Interpret the command line arguments.
    let cli = Cli::new(&conn);
    cli.interpret(args);
}
