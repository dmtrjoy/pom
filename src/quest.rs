use clap::ValueEnum;
use colored::{ColoredString, Colorize};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::convert::From;
use std::fmt;

/// A collection of quests, containing one main quest and a list of quest chains.
#[derive(Clone, Debug)]
pub struct Chain {
    chains: Vec<Chain>,
    main: Quest,
}

impl Chain {
    /// Constructs a new quest chain.
    pub fn new(main: Quest) -> Self {
        Self {
            chains: Vec::new(),
            main,
        }
    }

    pub fn add(&mut self, quest: Quest) {
        self.chains.push(Chain::new(quest));
    }

    // Copies the identifier.
    pub fn id(&self) -> i64 {
        self.main.id()
    }

    fn chain_id(&self) -> Option<i64> {
        self.main.chain_id()
    }

    pub fn link(&mut self, chain: Chain) {
        self.chains.push(chain);
    }

    /// Borrows the objective.
    pub fn objective(&self) -> &String {
        &self.main.objective()
    }

    /// Copies the status.
    pub fn status(&self) -> Status {
        self.main.status()
    }

    /// Copies the tier.
    pub fn tier(&self) -> Tier {
        self.main.tier()
    }

    pub fn chains(&self) -> &Vec<Chain> {
        &self.chains
    }

    fn chains_mut(&mut self) -> &mut Vec<Chain> {
        &mut self.chains
    }
}

/// A quest to be completed, including a tier, status, due date, and more.
#[derive(Clone,Debug)]
pub struct Quest {
    id: i64,
    chain_id: Option<i64>,
    objective: String,
    status: Status,
    tier: Tier,
}

/// Quest implementation.
impl Quest {
    /// Default identifier when a quest is first initialized. The true identifer is assigned and
    /// maintained by the `QuestDao`.
    const UNINITIALIZED_ID: i64 = -1;

    /// Constructs a new quest.
    pub fn new(objective: String, status: Status, tier: Tier, chain_id: Option<i64>) -> Self {
        Self::new_impl(Self::UNINITIALIZED_ID, chain_id, objective, status, tier)
    }

    // Copies the identifier.
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn chain_id(&self) -> Option<i64> {
        self.chain_id
    }

    /// Borrows the objective.
    pub fn objective(&self) -> &String {
        &self.objective
    }

    /// Copies the status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Borrows a mutable reference to the status.
    pub fn status_mut(&mut self) -> &mut Status {
        &mut self.status
    }

    /// Copies the tier.
    pub fn tier(&self) -> Tier {
        self.tier
    }

    /// Constructs a new quest.
    fn new_impl(
        id: i64,
        chain_id: Option<i64>,
        objective: String,
        status: Status,
        tier: Tier,
    ) -> Self {
        Self {
            id,
            chain_id,
            objective,
            status,
            tier,
        }
    }
}

/// Stores and loads quest data to and from the database.
pub struct QuestDao<'a> {
    conn: &'a Connection,
}

impl<'a> QuestDao<'a> {
    /// Constructs a new quest data access object.
    pub fn new(conn: &'a Connection) -> Self {
        // Create the `quest` table if it does not exist.
        Self::create_table(&conn);
        Self { conn }
    }

    /// Adds a new quest to the database.
    pub fn add(&self, quest: &Quest) {
        let query = "INSERT INTO quest (
            objective,
            status,
            tier,
            chain_id
        ) VALUES (?1, ?2, ?3, ?4)";
        let params = params![
            quest.objective,
            quest.status as i64,
            quest.tier as i64,
            quest.chain_id
        ];
        self.conn
            .execute(query, params)
            .expect("failed to add quest");
    }

    /// Gets all quest chains from the database.
    pub fn chains(&self) -> Vec<Chain> {
        // Prepare the query.
        let query = "
SELECT id, chain_id, objective, status, tier
FROM quest
ORDER BY id";
        let mut stmt = self
            .conn
            .prepare(query)
            .expect("failed to prepare get-all-chains statement");

        // Execute the query.
        let rows = stmt.query_map([], |row| {
            Ok(Quest::new_impl(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                Status::from(row.get::<_, i64>(3)?),
                Tier::from(row.get::<_, i64>(4)?),
            ))
        });
        let quest_iter = rows.expect("failed to get all quests");

        let mut chains = Vec::new();
        let mut prev_chains = HashMap::new();
        let mut roots = Vec::new();

        for quest in quest_iter {
            let quest = quest.unwrap();
            let chain_id: Option<i64> = quest.chain_id;
            let chain = Chain::new(quest);
            prev_chains.insert(chain.main.id, chain.clone());

            if let Some(chain_id) = chain_id {
                let chainx = prev_chains.get_mut(&chain_id).unwrap();
                chainx.chains.push(chain);
            } else {
                roots.push(chain.main.id);
            }
        }

        for root in roots {
            let chain = &mut prev_chains.remove(&root).unwrap();
            Self::rec(chain, &mut prev_chains);
            let x = chain.clone();
            chains.push(x);
        }

        chains
    }

    fn rec(chain: &mut Chain, chain_map: &mut HashMap<i64, Chain>) {
        if chain.chains.is_empty() {
            return;
        }
        
        for c in &mut chain.chains {
            c.chains = chain_map.remove(&c.main.id).unwrap().chains;
            Self::rec(c, chain_map);
        }
    }

    /// Deletes the specified quest from the database.
    pub fn delete(&self, quest_id: i64) {
        self.conn
            .execute(
                "DELETE FROM quest
                WHERE id = ?1",
                params![quest_id],
            )
            .expect("failed to delete quest");
    }

    /// Gets the specified quest from the database.
    pub fn get(&self, quest_id: i64) -> Quest {
        // Prepare the query.
        let query = "SELECT id, chain_id, objective, status, tier FROM quest WHERE id = ?1";
        let mut stmt = self
            .conn
            .prepare(query)
            .expect("failed to prepare get-quest statement");

        // Execute the query, and return the result.
        let params = [quest_id];
        stmt.query_row(params, |row: &rusqlite::Row<'_>| {
            Ok(Quest::new_impl(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                Status::from(row.get::<_, i64>(3)?),
                Tier::from(row.get::<_, i64>(4)?),
            ))
        })
        .expect("failed to get quest")
    }

    pub fn num_quests(&self, chain_id: i64) -> i64 {
        // Prepare the query.
        let query = "SELECT COUNT(*) FROM quest WHERE chain_id = ?1";
        let mut stmt = self
            .conn
            .prepare(query)
            .expect("failed to prepare get-all-quests statement");

        // Execute the query.
        let params = [chain_id];
        stmt.query_row(params, |row| {
            Ok(row.get(0)?)
        }).expect("failed to get all quests")
    }

    /// Gets all quests from the database.
    pub fn quests(&self) -> Vec<Quest> {
        // Prepare the query.
        let query = "SELECT id, chain_id, objective, status, tier FROM quest";
        let mut stmt = self
            .conn
            .prepare(query)
            .expect("failed to prepare get-all-quests statement");

        // Execute the query.
        let rows = stmt.query_map([], |row| {
            Ok(Quest::new_impl(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                Status::from(row.get::<_, i64>(3)?),
                Tier::from(row.get::<_, i64>(4)?),
            ))
        });
        let quest_iter = rows.expect("failed to get all quests");

        // Extract and return the results.
        let mut quests = Vec::new();
        for quest in quest_iter {
            quests.push(quest.expect("failed to extract quest from query map"));
        }

        quests
    }

    /// Updates the specified quest in the database.
    pub fn update(&self, quest: &Quest) {
        let query = "UPDATE quest
        SET chain_id  = ?1,
            objective = ?2,
            status    = ?3,
            tier      = ?4
        WHERE id = ?5";
        let params = params![
            quest.chain_id,
            quest.objective,
            quest.status as i64,
            quest.tier as i64,
            quest.id
        ];
        self.conn
            .execute(query, params)
            .expect("failed to update quest");
    }

    /// Creates the `quest` table if it does not exist.
    fn create_table(conn: &Connection) {
        let query = "CREATE TABLE quest (
                               id        INTEGER PRIMARY KEY,
                               chain_id  INTEGER,
                               objective TEXT NOT NULL,
                               status    INTEGER NOT NULL,
                               tier      INTEGER NOT NULL,
                               FOREIGN KEY (chain_id) REFERENCES quest(id)
                           )";
        if !Self::exists(&conn) {
            conn.execute(
                query,
                (), // Empty list of parameters.
            )
            .expect("failed to create table `quest`");
        }
    }

    /// Checks if the `quest` table exists.
    fn exists(conn: &Connection) -> bool {
        conn.prepare(
            "SELECT name
            FROM    sqlite_master
            WHERE   type = 'table'
            AND     name = 'quest'",
        )
        .expect("failed to prepare check-existence statement")
        .exists([])
        .expect("failed to check if table `quest` exists")
    }
}

/// A quest status, such as pending, ongoing, or completed.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Status {
    Pending = 0,
    Ongoing = 1,
    Completed = 2,
    Waiting = 3,
    Abandoned = 4,
}

// Display trait implementation for the `Status` enum.
impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pending => write!(formatter, "Pending"),
            Self::Ongoing => write!(formatter, "Ongoing"),
            Self::Completed => write!(formatter, "Completed"),
            Self::Waiting => write!(formatter, "Waiting"),
            Self::Abandoned => write!(formatter, "Abandoned"),
        }
    }
}

impl From<i64> for Status {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Pending,
            1 => Self::Ongoing,
            2 => Self::Completed,
            3 => Self::Waiting,
            4 => Self::Abandoned,
            _ => panic!("unknown status: `{}`", value),
        }
    }
}

/// A quest tier, indicating its difficulty or importance.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Tier {
    Common = 0,
    Rare = 1,
    Epic = 2,
    Legendary = 3,
}

impl Tier {
    pub fn to_colored_string(&self) -> ColoredString {
        match self {
            Self::Common => self.to_string().black(),
            Self::Rare => self.to_string().blue(),
            Self::Epic => self.to_string().purple(),
            Self::Legendary => self.to_string().yellow(),
        }
    }
}

// Display trait implementation for the `Tier` enum.
impl fmt::Display for Tier {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Common => write!(formatter, "✦ Common"),
            Self::Rare => write!(formatter, "✦ Rare"),
            Self::Epic => write!(formatter, "✦ Epic"),
            Self::Legendary => write!(formatter, "✦ Legendary"),
        }
    }
}

impl From<i64> for Tier {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Common,
            1 => Self::Rare,
            2 => Self::Epic,
            3 => Self::Legendary,
            _ => panic!("unknown tier: `{}`", value),
        }
    }
}
