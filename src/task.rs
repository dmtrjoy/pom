use chrono::{DateTime, Utc};
use clap::ValueEnum;
use rusqlite::{params, Connection};

#[derive(Clone, Copy, ValueEnum)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Status {
    NotStarted = 0,
    Complete = 1,
    Blocked = 2,
}

pub struct Task {
    what: String,
    priority: Priority,
    status: Status,
    creation_epoch: DateTime<Utc>,
    due_date_epoch: DateTime<Utc>,
}

impl Task {
    pub fn new(
        what: String,
        priority: Priority,
        status: Status,
        creation_epoch: DateTime<Utc>,
        due_date_epoch: DateTime<Utc>,
    ) -> Self {
        Self {
            what,
            priority,
            status,
            creation_epoch,
            due_date_epoch,
        }
    }

    // pub fn what(&self) -> &String {
    //     &self.what
    // }
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
                    what           TEXT NOT NULL UNIQUE,
                    priority       INTEGER NOT NULL,
                    status         INTEGER NOT NULL,
                    creation_epoch INTEGER NOT NULL,
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
            .execute(
                "INSERT INTO task (
                    what,
                    priority,
                    status,
                    creation_epoch, 
                    due_date_epoch
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    task.what,
                    task.priority as u8,
                    task.status as u8,
                    Self::to_milliseconds(&task.creation_epoch),
                    Self::to_milliseconds(&task.due_date_epoch)
                ],
            )
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

    fn to_milliseconds(datetime: &DateTime<Utc>) -> i64 {
        datetime.timestamp()
    }
}
