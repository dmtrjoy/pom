mod cli;
mod database;
mod item;
mod project;
mod session;

use clap::Parser;
use cli::{Args, Command};
use database::Database;
use item::{Item, ItemDao};
use project::{Project, ProjectDao};
use session::{Session, SessionDao};

fn main() {
    // Open the database connection.
    let database = Database::new();
    let conn = database.conn();

    // Load the session (active project).
    let session_dao = SessionDao::new(&conn);
    let session = session_dao.load();

    // Parse the command line arguments.
    let args = Args::parse();

    match args.command() {
        Command::Add { message } => {
            let item = Item::new(message);
            // let item_dao = ItemDao::new(&conn);
            // item_dao.add(&item);
        }
        Command::New { name } => {
            let project = Project::new(name);
            let project_dao = ProjectDao::new(&conn);
            project_dao.add(&project);
        }
        Command::Project => {
            let project_dao = ProjectDao::new(&conn);
            let projects = project_dao.all();
            for project in projects {
                println!("{}", project.name());
            }
        }
        Command::Switch { name } => {
            println!("'pom new' was used with name '{name:?}'");
        }
    }
}
