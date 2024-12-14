use rusqlite::{params, Connection};
use uuid::Uuid;

/// Represents a project or goal.
pub struct Project {
    uuid: Uuid,
    name: String,
}

/// Project implementation.
impl Project {
    /// Constructs a new project with a guaranteed unique project identifier.
    pub fn new(name: String) -> Self {
        // Generate a unique project identifer based on the current timestamp.
        let uuid = Uuid::now_v7();

        Self::new_impl(uuid, name)
    }

    /// Returns the unique project identifier.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Returns the project name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns and transfers ownership of the unique project identifier.
    pub fn take_uuid(&self) -> Uuid {
        self.uuid
    }

    /// Constructs a new project.
    fn new_impl(uuid: Uuid, name: String) -> Self {
        Self { uuid, name }
    }
}

/// Stores and loads project data to and from a database.
pub struct ProjectDao<'a> {
    conn: &'a Connection,
}

/// Project data access object implementation.
impl<'a> ProjectDao<'a> {
    /// Constructs a new project data access object.
    pub fn new(conn: &'a Connection) -> Self {
        // Create the `project` table if it does not exist.
        Self::create_table(&conn);

        Self { conn }
    }

    /// Adds a new project to the `project` table.
    pub fn add(&self, project: &Project) {
        self.conn
            .execute(
                "INSERT INTO project (uuid, name) VALUES (?1, ?2)",
                params![project.uuid, project.name],
            )
            .expect("failed to add project");
    }

    /// Fetches all projects from the `project` table.
    pub fn all(&self) -> Vec<Project> {
        // Prepare statement to fetch all projects.
        let mut stmt = self
            .conn
            .prepare("SELECT uuid, name FROM project")
            .expect("failed to prepare fetch-all-projects statement");

        // Fetch and store all projects.
        let project_iter = stmt
            .query_map([], |row: &rusqlite::Row<'_>| {
                Ok(Project::new_impl(row.get(0)?, row.get(1)?))
            })
            .expect("failed to fetch all projects");

        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project.expect("failed to extract project from query map"));
        }

        projects
    }

    /// Fetches the project with the specified name.
    pub fn get(&self, name: String) -> Project {
        // Prepare statement to fetch specified project.
        let mut stmt = self
            .conn
            .prepare(
                "SELECT uuid, name
                FROM    project
                WHERE   name = ?1")
            .expect("failed to prepare fetch-project statement");

        // Fetch the specified project.
        let project_iter = stmt
            .query_map([&name], |row: &rusqlite::Row<'_>| {
                Ok(Project::new_impl(row.get(0)?, row.get(1)?))
            })
            .expect("failed to fetch project");

        for project in project_iter {
            return project.expect("failed to extract project from query map");
        }

        // Project does not exist.
        panic!("no project found for name: `{:?}`", name);
    }

    /// Creates the `project` table if it does not exist. Panics if an error is encountered.
    fn create_table(conn: &Connection) {
        if !Self::exists(&conn) {
            conn.execute(
                "CREATE TABLE project (
                    uuid BLOB NOT NULL UNIQUE,
                    name TEXT NOT NULL UNIQUE
                )",
                (), // Empty list of parameters.
            )
            .expect("failed to create table `project`");
        }
    }

    /// Checks if the `project` table exists. Panics if an error is encountered.
    fn exists(conn: &Connection) -> bool {
        conn.prepare(
            "SELECT name
            FROM    sqlite_master
            WHERE   type = 'table'
            AND     name = 'project'",
        )
        .expect("failed to prepare check-existence statement")
        .exists([])
        .expect("failed to check if table `project` exists")
    }
}
