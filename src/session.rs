// use crate::project::Project;
// use rusqlite::{params, Connection};

// pub struct Session {
//     project: Project,
// }

// impl Session {
//     pub fn project(&self) -> &String {
//         &self.project
//     }
// }

// pub struct SessionDao<'a> {
//     conn: &'a Connection,
// }

// impl<'a> SessionDao<'a> {
//     pub fn new(conn: &'a Connection) -> Self {
//         // Check if the table already exists.
//         let exists = conn
//             .prepare(
//                 "SELECT name
//                 FROM sqlite_master
//                 WHERE type='table'
//                 AND name='session'",
//             )
//             .unwrap()
//             .exists([])
//             .unwrap();

//         // If not, create the table.
//         // 'value_id' is a foreign key that maps to primary key of any other table in the database.
//         if !exists {
//             conn.execute(
//                 "CREATE TABLE session (
//                     id       INTEGER PRIMARY KEY,
//                     key      TEXT NOT NULL UNIQUE,
//                     value_id INTEGER NOT NULL
//                 )",
//                 (), // Empty list of parameters.
//             )
//             .unwrap();
//         }

//         Self { conn }
//     }

//     pub fn load(&self) -> Session {
//         let mut stmt = self.conn.prepare("SELECT name FROM session").unwrap();
//         let rows = stmt.query_map([], |row| row.get(0)).unwrap();

//         assert!(rows.count() == 0);

//         rows
//     }
// }
