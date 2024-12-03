use crate::project::{Project, ProjectDao};
use crate::task::{Priority, Status, Task, TaskDao};
use clap::{Parser, Subcommand};
use rusqlite::Connection;
//use crate::session::{Session, SessionDao};

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Add a task
    #[command(long_about)]
    Do {
        what: String,
        #[arg(default_value_t = Priority::Low, long, short, value_enum)]
        priority: Priority,
        #[arg(default_value_t = Status::NotStarted, long, short, value_enum)]
        status: Status,
        #[arg(long, short)]
        due_date: String,
        //default_value_t [= <expr>]
    },

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

pub struct Cli<'a> {
    conn: &'a Connection,
}

impl<'a> Cli<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn interpret(&self, args: Args) {
        match args.command() {
            Command::Do { what, priority, status, due_date } => {
                let task = Task::new(what);
                let task_dao = TaskDao::new(&self.conn);
                task_dao.add(&task);
            }
            Command::New { name } => {
                let project = Project::new(name);
                let project_dao = ProjectDao::new(&self.conn);
                project_dao.add(&project);
            }
            Command::Project => {
                let project_dao = ProjectDao::new(&self.conn);
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
}


