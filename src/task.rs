use chrono::{DateTime, Utc};
use clap::ValueEnum;
use rusqlite::{params, Connection};

#[derive(Clone, ValueEnum)]
pub enum Priority {
    Critical,
    Low,
    Medium,
    High,
}

#[derive(Clone, ValueEnum)]
pub enum Status {
    Blocked,
    Complete,
    NotStarted,
}

pub struct Task {
    what: String,
    priority: Priority,
    status: Status,
    creation_epoch: DateTime<Utc>,
    due_date_epoch: DateTime<Utc>,
}

impl Task {
    pub fn new(what: String) -> Self {
        let priority = Priority::Low;
        let status = Status::NotStarted;
        let creation_epoch = Utc::now();
        let due_date_epoch = Utc::now();
        Self { what, priority, status, creation_epoch, due_date_epoch }
    }

    pub fn what(&self) -> &String {
        &self.what
    }
}

pub struct TaskDao<'a> {
    conn: &'a Connection,
}

impl<'a> TaskDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        // Check if the table already exists.
        let exists = conn
            .prepare(
                "SELECT name
                FROM    sqlite_master
                WHERE   type='table'
                AND     name='task'",
            )
            .unwrap()
            .exists([])
            .unwrap();

        // If not, create the table.
        if !exists {
            conn.execute(
                "CREATE TABLE task (
                    id             INTEGER PRIMARY KEY,
                    what           TEXT NOT NULL UNIQUE
                    priority       INTEGER NOT NULL
                    status         INTEGER NOT NULL
                    creation_epoch INTEGER NOT NULL
                    due_date_epoch INTEGER NOT NULL
                )",
                (), // Empty list of parameters.
            )
            .unwrap();
        }

        Self { conn }
    }

    pub fn add(&self, task: &Task) {
        self.conn
            .execute("INSERT INTO task (what) VALUES (?1)", params![task.what])
            .unwrap();
    }

    // pub fn all(&self) -> Vec<Item> {
    //     let mut stmt = self.conn.prepare("SELECT message FROM task").unwrap();
    //     let rows = stmt.query_map([], |row| row.get(0)).unwrap();

    //     let mut tasks = Vec::new();
    //     for name in rows {
    //         let item = Item::new(name.unwrap());
    //         items.push(item);
    //     }

    //     items
    // }
}
