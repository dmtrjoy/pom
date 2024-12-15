use rusqlite::{params, Connection};
use uuid::Uuid;

/// Represent a session data entry.
pub struct Entry {
    key: Key,
    value_uuid: Uuid,
}

/// Session data entry implementation.
impl Entry {
    /// Constructs a new session data entry.
    pub fn new(key: Key, value_uuid: Uuid) -> Self {
        Self { key, value_uuid }
    }

    /// Borrows the session key.
    pub fn key(&self) -> Key {
        self.key
    }

    /// Borrows the session value ID, which maps to the primary key
    pub fn value_uuid(&self) -> &Uuid {
        &self.value_uuid
    }
}

// Represents a session data entry key.
#[derive(Copy, Clone, Debug)]
pub enum Key {
    ActiveProject = 0,
}

/// Stores and loads persistent session data to and from a database. Session data are modeled as
/// key-value pairs.
pub struct Session<'a> {
    conn: &'a Connection,
}

/// Session implementation.
impl<'a> Session<'a> {
    /// Constructs a new session.
    pub fn new(conn: &'a Connection) -> Self {
        // Create the `session` table if it does not exist.
        Self::create_table(&conn);

        Self { conn }
    }

    /// Fetches the unique value identifier of the specified session data key.
    pub fn get(&self, key: Key) -> Uuid {
        // Prepare statement to fetch the unique value identifier.
        let mut stmt = self
            .conn
            .prepare(
                "SELECT value_uuid
                FROM    session
                WHERE   key = ?1",
            )
            .expect("failed to prepare fetch-value-uuid statement");

        // Fetch the unique value identifier.
        let value_uuid_iter = stmt
            .query_map([key as i64], |row| Ok(row.get(0)?))
            .expect("failed to fetch unique value identifier");

        // Ensure the unique value identifier exists.
        for value_uuid in value_uuid_iter {
            return value_uuid.expect("failed to extract unique value identifier from query map");
        }

        // Session data key does not exist.
        panic!("no session data entries found for key: `{:?}`", key);
    }

    /// Sets a session data entry in the `session` table.
    pub fn set(&self, entry: Entry) {
        // Create the session data entry if it does not exist. Otherwise, update the existing
        // session data entry.
        if !self.has(entry.key()) {
            return self.add(entry);
        }

        self.update(entry)
    }

    /// Adds a new session data entry to the `session` table.
    fn add(&self, entry: Entry) {
        self.conn
            .execute(
                "INSERT INTO session (key, value_uuid) VALUES (?1, ?2)",
                params![entry.key() as i64, entry.value_uuid()],
            )
            .expect("project name is not unique");
    }

    /// Checks if the `session` table has a specified session data key.
    fn has(&self, key: Key) -> bool {
        self.conn
            .prepare(
                "SELECT key
                FROM    session
                WHERE   key = ?1",
            )
            .expect("failed to prepare check-existence statement")
            .exists([key as i64])
            .expect("failed to check if key exists")
    }

    /// Updates an existing session data entry.
    fn update(&self, entry: Entry) {
        self.conn
            .prepare(
                "UPDATE session
                SET     value_uuid = ?1
                WHERE   key = ?2",
            )
            .expect("failed to prepare update statement")
            .execute(params![entry.value_uuid(), entry.key() as i64])
            .expect("failed to update session data entry");
    }

    /// Creates the `session` table if it does not exist. Panics if an error is encountered.
    fn create_table(conn: &Connection) {
        if !Self::exists(&conn) {
            conn.execute(
                "CREATE TABLE session (
                    key        TEXT NOT NULL UNIQUE,
                    value_uuid BLOB NOT NULL
                )",
                (), // Empty list of parameters.
            )
            .expect("failed to create table `session`");
        }
    }

    /// Checks if the `session` table exists. Panics if an error is encountered.
    fn exists(conn: &Connection) -> bool {
        conn.prepare(
            "SELECT name
            FROM    sqlite_master
            WHERE   type = 'table'
            AND     name = 'session'",
        )
        .expect("failed to prepare check-existence statement")
        .exists([])
        .expect("failed to check if table `session` exists")
    }
}
