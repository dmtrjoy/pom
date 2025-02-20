use std::collections::HashMap;
use std::convert::From;
use std::fmt::{Display, Formatter, Result};

use clap::ValueEnum;
use colored::{ColoredString, Colorize};
use rusqlite::{params, Connection};

/// A collection of quests, containing one main quest and a list of secondary
/// quest chains.
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

    // Borrows the secondary quest chains.
    pub fn chains(&self) -> &Vec<Chain> {
        &self.chains
    }

    /// Copies the identifier.
    pub fn id(&self) -> i64 {
        self.main.id()
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
}

/// A quest to be completed, including a tier, status, due date, and more.
#[derive(Clone, Debug)]
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

    /// Borrows the objective.
    pub fn objective(&self) -> &String {
        &self.objective
    }

    /// Copies the status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Borrows a mutable reference of the status.
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
    pub fn add_quest(&self, quest: &Quest) {
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

    /// Deletes the specified quest chain from the database.
    pub fn delete_chain(&self, chain_id: i64) {
        self.conn
            .execute(
                "WITH RECURSIVE chain AS (
                    SELECT id FROM quest WHERE id = ?1 OR chain_id = ?1
                    UNION ALL
                    SELECT quest.id FROM quest
                    INNER JOIN chain ON quest.chain_id = chain.id
                ) DELETE FROM quest WHERE id IN (SELECT id FROM chain)",
                params![chain_id],
            )
            .expect("failed to delete quest chain");
    }

    /// Gets all quest chains from the database.
    pub fn get_all_chains(&self) -> Vec<Chain> {
        // Get all quests from the database.
        let quests = self.get_all_quests();

        // Construct a hashmap of chains, where each key is a chain identifer
        // corresponding to the chain itself and its immediate children. Root
        // chains will, therefore, be disjoint from chains beyond their
        // immediate children.
        let mut disjoint_chains = HashMap::new();
        let mut root_chains = Vec::new();

        for quest in quests {
            let chain = Chain::new(quest);
            disjoint_chains.insert(chain.main.id, chain.clone());

            if let Some(chain_id) = chain.main.chain_id {
                // Chain has a parent, which must already exist in the hash map
                // since the quests are sorted by identifier.
                let parent_chain = disjoint_chains.get_mut(&chain_id).unwrap();
                parent_chain.chains.push(chain);
            } else {
                // Chain does not have a parent and, thus, must be a root chain.
                root_chains.push(chain.main.id);
            }
        }

        let mut chains = Vec::new();

        for root_chain in root_chains {
            let root_chain = &mut disjoint_chains.remove(&root_chain).unwrap();
            Self::connect(root_chain, &mut disjoint_chains);
            let chain = root_chain.clone();
            chains.push(chain);
        }

        chains
    }

    /// Gets all quests from the database.
    pub fn get_all_quests(&self) -> Vec<Quest> {
        // Prepare the query.
        let query = "SELECT id, chain_id, objective, status, tier FROM quest ORDER BY id";
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

    /// Gets the specified quest from the database.
    pub fn get_quest(&self, quest_id: i64) -> Quest {
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

    // Checks if the specified quest is a main quest.
    pub fn is_main_quest(&self, quest_id: i64) -> bool {
        let query = "SELECT COUNT() FROM quest WHERE chain_id = ?1";
        let mut stmt = self
            .conn
            .prepare(query)
            .expect("failed to prepare get-quest statement");

        let params = [quest_id];
        stmt.query_row(params, |row: &rusqlite::Row<'_>| Ok(row.get::<_, i64>(0)?))
            .expect("failed to get quest")
            > 0
    }

    /// Updates the status of every quest in specified quest chain.
    pub fn update_chain_status(&self, chain_id: i64, status: Status) {
        self.conn
            .execute(
                "WITH RECURSIVE chain AS (
                    SELECT id FROM quest WHERE id = ?1 OR chain_id = ?1
                    UNION ALL
                    SELECT quest.id FROM quest
                    INNER JOIN chain ON quest.chain_id = chain.id
                ) UPDATE quest SET status = ?2 WHERE id IN (SELECT id FROM chain)",
                params![chain_id, status as i64],
            )
            .expect("failed to delete quest chain");
    }

    /// Updates the specified quest.
    pub fn update_quest(&self, quest: &Quest) {
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

    /// Connects a set of disjoint chains into one complete chain, where the
    /// root chain is the first chain passed into the function.
    fn connect(chain: &mut Chain, disjoint_chains: &mut HashMap<i64, Chain>) {
        if chain.chains.is_empty() {
            return;
        }

        for child_chain in &mut chain.chains {
            child_chain.chains = disjoint_chains.remove(&child_chain.main.id).unwrap().chains;
            Self::connect(child_chain, disjoint_chains);
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

impl Display for Status {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
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
            Self::Common => self.to_string().into(),
            Self::Rare => self.to_string().blue(),
            Self::Epic => self.to_string().purple(),
            Self::Legendary => self.to_string().yellow(),
        }
    }
}

impl Display for Tier {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
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
