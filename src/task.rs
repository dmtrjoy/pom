use core::panic;

use chrono::{DateTime, Local, TimeZone, Utc};
use clap::ValueEnum;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::color::{Color, Foreground};

/// Represents possible task priorities.
#[derive(Clone, Copy, ValueEnum)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Priority implementation.
impl Priority {
    /// Maps the priority to the corresponding rich display representation,
    /// including colors, formatting, and sizing.
    pub fn as_display(&self) -> String {
        match self {
            Priority::Low => Foreground::color(&self.as_str().to_string(), Color::LightGreen),
            Priority::Medium => Foreground::color(&self.as_str().to_string(), Color::Orange),
            Priority::High => Foreground::color(&self.as_str().to_string(), Color::Red),
            Priority::Critical => Foreground::color(&self.as_str().to_string(), Color::BrightRed),
        }
    }

    /// Maps the priority to the corresponding string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }

    /// Maps a 64-bit integer to the corresponding `Priority` enum.
    pub fn from_i64(value: i64) -> Priority {
        match value {
            0 => Self::Low,
            1 => Self::Medium,
            2 => Self::High,
            3 => Self::Critical,
            _ => panic!("unknown priority: `{}`", value),
        }
    }
}

/// Represents possible task statuses.
#[derive(Clone, Copy, ValueEnum)]
pub enum Status {
    NotStarted = 0,
    InProgress = 1,
    Completed = 2,
    Blocked = 3,
    Overdue = 4,
}

/// Status implementation.
impl Status {
    /// Maps the priority to the corresponding rich display representation,
    /// including colors, formatting, and sizing.
    pub fn as_display(&self) -> String {
        match self {
            Status::NotStarted => Foreground::color(&self.as_str().to_string(), self.color()),
            Status::InProgress => Foreground::color(&self.as_str().to_string(), self.color()),
            Status::Completed => Foreground::color(&self.as_str().to_string(), self.color()),
            Status::Blocked => Foreground::color(&self.as_str().to_string(), self.color()),
            Status::Overdue => Foreground::color(&self.as_str().to_string(), self.color()),
        }
    }

    /// Maps the status to the corresponding string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Status::NotStarted => "not-started",
            Status::InProgress => "in-progress",
            Status::Completed => "completed",
            Status::Blocked => "blocked",
            Status::Overdue => "overdue",
        }
    }

    /// Copies the status color.
    pub fn color(&self) -> Color {
        match self {
            Status::NotStarted => Color::Blue,
            Status::InProgress => Color::Yellow,
            Status::Completed => Color::Green,
            Status::Blocked => Color::Red,
            Status::Overdue => Color::BrightRed,
        }
    }

    /// Maps a 64-bit integer to the corresponding `Status` enum.
    fn from_i64(value: i64) -> Status {
        match value {
            0 => Self::NotStarted,
            1 => Self::InProgress,
            2 => Self::Completed,
            3 => Self::Blocked,
            4 => Self::Overdue,
            _ => panic!("unknown status: `{}`", value),
        }
    }
}

/// Represents a to-do item with a priority, current status, due date, and more.
pub struct Task {
    uuid: Uuid,
    what: String,
    priority: Priority,
    status: Status,
    creation_date: DateTime<Local>,
    due_date: DateTime<Local>,
    project_uuid: Uuid,
}

/// Task implementation.
impl Task {
    /// Constructs a new task with a guaranteed unique task identifier.
    pub fn new(
        what: String,
        priority: Priority,
        status: Status,
        creation_date: DateTime<Local>,
        due_date: DateTime<Local>,
        project_uuid: Uuid,
    ) -> Self {
        // Generate a unique task identifer based on the current timestamp.
        let uuid = Uuid::now_v7();

        Self::new_impl(
            uuid,
            what,
            priority,
            status,
            creation_date,
            due_date,
            project_uuid,
        )
    }

    /// Borrows the unique task identifier.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Borrows the task objective.
    pub fn what(&self) -> &String {
        &self.what
    }

    /// Borrows a mutable reference to the task objective.
    pub fn what_mut(&mut self) -> &mut String {
        &mut self.what
    }

    /// Copies the status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Borrows a mutable reference to the status.
    pub fn status_mut(&mut self) -> &mut Status {
        &mut self.status
    }

    /// Copies the priority.
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Borrows a mutable reference to the priority.
    pub fn priority_mut(&mut self) -> &mut Priority {
        &mut self.priority
    }

    /// Borrows the due date.
    pub fn due_date(&self) -> &DateTime<Local> {
        &self.due_date
    }

    /// Borrows a mutable reference to the due date.
    pub fn due_date_mut(&mut self) -> &mut DateTime<Local> {
        &mut self.due_date
    }

    /// Constructs a new task.
    fn new_impl(
        uuid: Uuid,
        what: String,
        priority: Priority,
        status: Status,
        creation_date: DateTime<Local>,
        due_date: DateTime<Local>,
        project_uuid: Uuid,
    ) -> Self {
        Self {
            uuid,
            what,
            priority,
            status,
            creation_date,
            due_date,
            project_uuid,
        }
    }
}

/// Stores and loads task data to and from a database.
pub struct TaskDao<'a> {
    conn: &'a Connection,
}

/// Task data access object implementation.
impl<'a> TaskDao<'a> {
    /// Constructs a new task data access object.
    pub fn new(conn: &'a Connection) -> Self {
        // Create the `task` table if it does not exist.
        Self::create_table(&conn);

        Self { conn }
    }

    /// Adds a new task to the `task` table.
    pub fn add(&self, task: &Task) {
        // All datetimes are stored in UTC and converted to the local timezone
        // when interpreting commands.
        let creation_date_utc: DateTime<Utc> = DateTime::from(task.creation_date);
        let due_date_utc: DateTime<Utc> = DateTime::from(task.due_date);

        self.conn
            .execute(
                "INSERT INTO task (
                    uuid,
                    what,
                    priority,
                    status,
                    creation_epoch, 
                    due_date_epoch,
                    project_uuid
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    task.uuid,
                    task.what,
                    task.priority as i64,
                    task.status as i64,
                    creation_date_utc.timestamp(),
                    due_date_utc.timestamp(),
                    task.project_uuid
                ],
            )
            .expect("failed to add task");
    }

    /// Fetches all tasks from the `task` table.
    pub fn all(&self, project_uuid: Uuid) -> Vec<Task> {
        // Prepare statement to fetch all tasks.
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    uuid,
                    what,
                    priority,
                    status,
                    creation_epoch,
                    due_date_epoch,
                    project_uuid
                FROM task
                WHERE project_uuid = ?1",
            )
            .expect("failed to prepare fetch-all-tasks statement");

        // Fetch and store all tasks.
        let task_iter = stmt
            .query_map([project_uuid], |row| {
                Ok(Task::new_impl(
                    row.get(0)?,
                    row.get(1)?,
                    Priority::from_i64(row.get(2)?),
                    Status::from_i64(row.get(3)?),
                    DateTime::from(Utc.timestamp_opt(row.get(4)?, 0).unwrap()),
                    DateTime::from(Utc.timestamp_opt(row.get(5)?, 0).unwrap()),
                    row.get(6)?,
                ))
            })
            .expect("failed to fetch all tasks");

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task.expect("failed to extract task from query map"));
        }

        tasks
    }

    /// Deletes a task from the `task` table.
    pub fn delete(&self, task_uuid: &Uuid) {
        self.conn
            .execute(
                "DELETE FROM task
                WHERE uuid = ?1",
                params![task_uuid],
            )
            .expect("failed to delete task");
    }

    /// Fetches a task from the `task` table.
    pub fn get(&self, task_uuid: &Uuid) -> Task {
        // Prepare statement to fetch all tasks.
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    uuid,
                    what,
                    priority,
                    status,
                    creation_epoch,
                    due_date_epoch,
                    project_uuid
                FROM task
                WHERE uuid = ?1",
            )
            .expect("failed to prepare fetch-all-tasks statement");

        // Fetch and store all tasks.
        let task_iter = stmt
            .query_map([task_uuid], |row| {
                Ok(Task::new_impl(
                    row.get(0)?,
                    row.get(1)?,
                    Priority::from_i64(row.get(2)?),
                    Status::from_i64(row.get(3)?),
                    DateTime::from(Utc.timestamp_opt(row.get(4)?, 0).unwrap()),
                    DateTime::from(Utc.timestamp_opt(row.get(5)?, 0).unwrap()),
                    row.get(6)?,
                ))
            })
            .expect("failed to fetch a task");

        for task in task_iter {
            return task.expect("failed to extract task from query map");
        }

        panic!("failed to fetch a task");
    }

    /// Updates a task in the `task` table.
    pub fn update(&self, task: &Task) {
        // All datetimes are stored in UTC and converted to the local timezone
        // when interpreting commands.
        let due_date_utc: DateTime<Utc> = DateTime::from(task.due_date);

        self.conn
            .execute(
                "UPDATE task
                SET
                    what = ?1,
                    priority = ?2,
                    status = ?3,
                    due_date_epoch = ?4
                WHERE uuid = ?5",
                params![
                    task.what,
                    task.priority as i64,
                    task.status as i64,
                    due_date_utc.timestamp(),
                    task.uuid
                ],
            )
            .expect("failed to update task");
    }

    /// Creates the `task` table if it does not exist. Panics if an error is encountered.
    fn create_table(conn: &Connection) {
        if !Self::exists(&conn) {
            conn.execute(
                "CREATE TABLE task (
                    uuid           BLOB NOT NULL UNIQUE,
                    what           TEXT NOT NULL,
                    priority       INTEGER NOT NULL,
                    status         INTEGER NOT NULL,
                    creation_epoch INTEGER NOT NULL,
                    due_date_epoch INTEGER NOT NULL,
                    project_uuid   BLOB NOT NULL
                )",
                (), // Empty list of parameters.
            )
            .expect("failed to create table `task`");
        }
    }

    /// Checks if the `task` table exists. Panics if an error is encountered.
    fn exists(conn: &Connection) -> bool {
        conn.prepare(
            "SELECT name
            FROM    sqlite_master
            WHERE   type = 'table'
            AND     name = 'task'",
        )
        .expect("failed to prepare check-existence statement")
        .exists([])
        .expect("failed to check if table `task` exists")
    }
}
