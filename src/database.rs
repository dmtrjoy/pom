use directories::ProjectDirs;
use rusqlite::Connection;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Represents a local database reference for storing projects, tasks, and more.
pub struct Database {
    database_path: PathBuf,
}

/// Local database reference implementation.
impl Database {
    const DATABASE_NAME: &str = "pomdb.sqlite";

    /// Constructs a new database reference.
    pub fn new() -> Self {
        // Read the local data path.
        let project_dirs = ProjectDirs::from("com", "Ode", "pom").unwrap();
        let data_dir = project_dirs.data_dir();

        // Create the data directory if it does not exist.
        Self::create_dir(data_dir);

        // Construct the database path.
        let database_path = data_dir.join(Self::DATABASE_NAME);

        Self { database_path }
    }

    /// Opens and returns a reference to the database connection.
    pub fn conn(&self) -> Connection {
        Connection::open(&self.database_path).expect("failed to open database connection")
    }

    /// Creates a new directory. Panics if an error is encountered.
    fn create_dir(dir: &Path) {
        // Create the data directory if necessary.
        if !fs::exists(&dir)
            .unwrap_or_else(|_| panic!("failed to check if path `{}` exists", dir.display()))
        {
            fs::create_dir_all(&dir)
                .unwrap_or_else(|_| panic!("failed to create directory: `{}`", dir.display()));
        }
    }
}
