use directories::ProjectDirs;
use rusqlite::Connection;
use std::fs;

pub struct Database {
    database_path: String,
}

impl Database {
    const DATABASE_NAME: &str = "pomdb.sqlite";

    pub fn new() -> Self {
        // Read the local data path.
        let project_dirs = ProjectDirs::from("com", "Root", "pom").unwrap();
        let local_data_path = project_dirs.data_local_dir();

        // Create the directory if necessary.
        if !fs::exists(&local_data_path).unwrap() {
            fs::create_dir(&local_data_path).unwrap();
        }

        // Read the database path.
        let database_path = format!("{}/{}", local_data_path.display(), Self::DATABASE_NAME);

        Self { database_path }
    }

    pub fn conn(&self) -> Connection {
        Connection::open(&self.database_path).unwrap()
    }
}
