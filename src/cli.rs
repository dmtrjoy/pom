use std::collections::BTreeMap;

use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone};
use clap::{
    builder::{styling::Ansi256Color, Styles},
    Parser, Subcommand,
};

use crate::color::{Color, Foreground};
use crate::database::Database;
use crate::project::{Project, ProjectDao};
use crate::session::{Entry, Key, Session};
use crate::task::{Priority, Status, Task, TaskDao};

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

    /// Show or modify projects
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
                Self::add_task(what, priority, status, due_date);
            }
            Command::New { name } => {
                Self::add_project(name);
            }
            Command::Project => {
                // Get the active project.
                let project_uuid = session.get(Key::ActiveProject);

                let project_dao = ProjectDao::new(&conn);
                let projects = project_dao.all();

                // Print all projects, distinguishing the active project.
                for project in projects {
                    if project.uuid().eq(&project_uuid) {
                        println!("-> {}", project.name());
                    } else {
                        println!("   {}", project.name());
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
                Self::display_tasks();
            }
        }
    }

    /// Creates a new project.
    fn add_project(name: String) {
        // Open the database connection. Shared across all data access objects.
        let database = Database::new();
        let conn = database.conn();

        // Load persistent session data.
        let session = Session::new(&conn);

        // Create and add a new project to the database.
        let project = Project::new(name);
        let project_dao = ProjectDao::new(&conn);
        project_dao.add(&project);

        // Update the active project.
        let entry: Entry = Entry::new(Key::ActiveProject, project.take_uuid());
        session.set(entry);
    }

    /// Adds a task to the active project.
    fn add_task(what: String, priority: Priority, status: Status, due_date: String) {
        // Open the database connection. Shared across all data access objects.
        let database = Database::new();
        let conn = database.conn();

        // Load persistent session data.
        let session = Session::new(&conn);

        // Parse the due date as a local datetime.
        let due_datetime = due_date + "00:00:00"; // Due at midnight.
        let naive_due_date_epoch =
            NaiveDateTime::parse_from_str(&due_datetime, "%m-%d-%Y %H:%M:%S")
                .expect("failed to parse due date");

        let creation_epoch = Local::now();
        let due_date_epoch = Local.from_local_datetime(&naive_due_date_epoch).unwrap();

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

    /// Displays all tasks that belong to the active project.
    fn display_tasks() {
        // Open the database connection. Shared across all data access objects.
        let database = Database::new();
        let conn = database.conn();

        // Load persistent session data.
        let session = Session::new(&conn);

        // Get the active project and its tasks.
        let project_uuid = session.get(Key::ActiveProject);
        let task_dao = TaskDao::new(&conn);
        let tasks = task_dao.all(project_uuid);

        // Group tasks by naive due date (no notion of time zone).
        let mut tasks_grouped_by_due_date = BTreeMap::<NaiveDate, Vec<Task>>::new();
        for task in tasks {
            let naive_due_date = task.due_date().date_naive();
            if tasks_grouped_by_due_date.contains_key(&naive_due_date) {
                tasks_grouped_by_due_date
                    .entry(naive_due_date)
                    .and_modify(|tasks| tasks.push(task));
            } else {
                tasks_grouped_by_due_date.insert(naive_due_date, Vec::<Task>::from([task]));
            }
        }

        // Print all tasks that belong to the active project.
        let mut naive_due_date_idx = 0;
        for (naive_due_date, tasks) in tasks_grouped_by_due_date.iter().rev() {
            for task in tasks {
                let uuid = format!("task {}", task.uuid().to_string());
                println!("{}", Foreground::color(&uuid, task.status().color()));
                println!("due:      {}", naive_due_date);
                println!("status:   {}", task.status().as_display());
                println!("priority: {}", task.priority().as_display());
                println!("\n    {}", Foreground::color(&task.what(), Color::White));
                if naive_due_date_idx != tasks_grouped_by_due_date.len() - 1 {
                    println!("");
                }
            }

            naive_due_date_idx += 1;
        }
    }
}
