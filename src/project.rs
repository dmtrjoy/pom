use rusqlite::{params, Connection};

pub struct Project {
    name: String,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

pub struct ProjectDao<'a> {
    conn: &'a Connection,
}

impl<'a> ProjectDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        // Check if the table already exists.
        let exists = conn
            .prepare(
                "SELECT name
                FROM sqlite_master
                WHERE type='table'
                AND name='project'",
            )
            .unwrap()
            .exists([])
            .unwrap();

        // If not, create the table.
        if !exists {
            conn.execute(
                "CREATE TABLE project (
                    id   INTEGER PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE
                )",
                (), // Empty list of parameters.
            )
            .unwrap();
        }

        Self { conn }
    }

    pub fn add(&self, project: &Project) {
        self.conn
            .execute(
                "INSERT INTO project (name) VALUES (?1)",
                params![project.name],
            )
            .expect("project name is not unique");
    }

    pub fn all(&self) -> Vec<Project> {
        let mut stmt = self.conn.prepare("SELECT name FROM project").unwrap();
        let rows = stmt.query_map([], |row| row.get(0)).unwrap();

        let mut projects = Vec::new();
        for name in rows {
            let project = Project::new(name.unwrap());
            projects.push(project);
        }

        projects
    }
}
