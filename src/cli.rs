use crate::database::Database;
use crate::project::{Project, ProjectDao};
use crate::session::{Entry, Key, Session};
use crate::task::{Priority, Status, Task, TaskDao};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use clap::{Parser, builder::{Styles, styling::Ansi256Color}, Subcommand};

const STYLES: Styles = Styles::styled()
    .header(Ansi256Color(47).on_default())
    .usage(Ansi256Color(227).on_default())
    .literal(Ansi256Color(231).on_default())
    .placeholder(Ansi256Color(156).on_default());

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
    },

    /// Create a new project
    #[command(color = clap::ColorChoice::Always, long_about)]
    New { name: String },

    /// List, modify, or delete projects
    #[command(long_about)]
    Project,

    /// Switch projects
    #[command(long_about)]
    Switch { name: String },

    /// List, modify, or delete tasks
    #[command(long_about)]
    Task,
}

/// Create and manage projects, set timers, and more!
#[derive(Parser)]
#[command(long_about, styles = STYLES, version)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

impl Args {
    pub fn command(&self) -> Command {
        self.command.clone()
    }
}

/// Represents the command line interface, which can interpret parsed arguments.
pub struct Cli {}

/// Command line interface implementation.
impl Cli {
    /// Interprets the parse arguments from the command line.
    pub fn interpret(args: Args) {
        // Open the database connection. Shared across all data access objects.
        let database = Database::new();
        let conn = database.conn();

        // Load persistent session data.
        let session = Session::new(&conn);

        match args.command() {
            Command::Do {
                what,
                priority,
                status,
                due_date,
            } => {
                // Parse the due date as a local datetime.
                let due_datetime = due_date + "00:00:00"; // Due at midnight.
                let naive_due_date_epoch = NaiveDateTime::parse_from_str(&due_datetime, "%m-%d-%Y %H:%M:%S")
                    .expect("failed to parse due date");
                let local_due_date_epoch =
                    Local.from_local_datetime(&naive_due_date_epoch).unwrap();

                // Convert datetimes to UTC before saving. All datetimes are stored in UTC and
                // converted to the local timezone when interpreting commands.
                let creation_epoch = Utc::now();
                let due_date_epoch = DateTime::from(local_due_date_epoch);

                // Get the active project, which the newly created task belongs to.
                let project_uuid = session.get(Key::ActiveProject);

                // Construct and save the task.
                let task = Task::new(
                    what,
                    priority,
                    status,
                    creation_epoch,
                    due_date_epoch,
                    project_uuid,
                );
                let task_dao = TaskDao::new(&conn);
                task_dao.add(&task);
            }
            Command::New { name } => {
                // Create and add a new project to the database.
                let project = Project::new(name);
                let project_dao = ProjectDao::new(&conn);
                project_dao.add(&project);

                // Update the active project.
                let entry: Entry = Entry::new(Key::ActiveProject, project.take_uuid());
                session.set(entry);
            }
            Command::Project => {
                // Get the active project.
                let project_uuid = session.get(Key::ActiveProject);

                let project_dao = ProjectDao::new(&conn);
                let projects = project_dao.all();

                // Print all projects, distinguishing the active project. 
                for project in projects {
                    if project.uuid().eq(&project_uuid) {
                        println!("* {}", project.name());
                    } else {
                        println!("  {}", project.name());
                    }
                }
            }
            Command::Switch { name } => {
                // Get the specified project.
                let project_dao = ProjectDao::new(&conn);
                let project = project_dao.get(name);

                // Update the active project.
                let entry: Entry = Entry::new(Key::ActiveProject, project.take_uuid());
                session.set(entry);
            }
            Command::Task => {
                // Get the active project.
                let project_uuid = session.get(Key::ActiveProject);

                let task_dao = TaskDao::new(&conn);
                let tasks = task_dao.all(project_uuid);

                // Print all tasks that belong to the active project.
                for task in tasks {
                    println!("{}", task.what());
                }
            }
        }
    }
}
