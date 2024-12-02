use rusqlite::{params, Connection};

pub struct Item {
    message: String,
}

impl Item {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

pub struct ItemDao<'a> {
    conn: &'a Connection,
}

impl<'a> ItemDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        // Check if the table already exists.
        let exists = conn
            .prepare(
                "SELECT name
                FROM    sqlite_master
                WHERE   type='table'
                AND     name='item'",
            )
            .unwrap()
            .exists([])
            .unwrap();

        // If not, create the table.
        if !exists {
            conn.execute(
                "CREATE TABLE item (
                    id      INTEGER PRIMARY KEY,
                    message TEXT NOT NULL UNIQUE
                )",
                (), // Empty list of parameters.
            )
            .unwrap();
        }

        Self { conn }
    }

    pub fn add(&self, item: &Item) {
        self.conn
            .execute("INSERT INTO item (name) VALUES (?1)", params![item.message])
            .expect("item name is not unique");
    }

    pub fn all(&self) -> Vec<Item> {
        let mut stmt = self.conn.prepare("SELECT message FROM item").unwrap();
        let rows = stmt.query_map([], |row| row.get(0)).unwrap();

        let mut items = Vec::new();
        for name in rows {
            let item = Item::new(name.unwrap());
            items.push(item);
        }

        items
    }
}
