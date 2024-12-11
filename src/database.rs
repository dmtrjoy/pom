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
        let project_dirs = ProjectDirs::from("com", "ode", "pom").unwrap();
        let data_path = project_dirs.data_dir();

        // Create the directory if necessary.
        if !fs::exists(&data_path).unwrap() {
            fs::create_dir_all(&data_path).unwrap();
        }

        // Construct the database path.
        let database_path = format!("{}/{}", data_path.display(), Self::DATABASE_NAME);

        Self { database_path }
    }

    pub fn conn(&self) -> Connection {
        Connection::open(&self.database_path).unwrap()
    }
}
