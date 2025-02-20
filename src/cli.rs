use std::io::{stdin, stdout, Write};

use clap::builder::styling::AnsiColor;
use clap::builder::Styles;
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::database::Database;
use crate::quest::{Chain, Quest, QuestDao, Status, Tier};
use crate::table::{Cell, Table};

/// Default styles.
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Yellow.on_default());

/// Create and manage projects, set timers, and more!
#[derive(Parser)]
#[command(long_about, styles = STYLES, version)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

/// Args implementation.
impl Args {
    fn command(&self) -> Command {
        self.command.clone()
    }
}

/// Represents every possible `quest` command.
#[derive(Clone, Subcommand)]
enum Command {
    /// Abandon a quest
    #[command(long_about)]
    Abandon {
        /// Quest ID
        quest_id: i64,
    },

    /// Accept a quest
    #[command(long_about)]
    Accept {
        /// Quest ID
        quest_id: i64,
    },

    /// Add a quest
    #[command(long_about)]
    Add {
        /// Objective
        objective: String,

        /// Status
        #[arg(default_value_t = Status::Pending, long, short, value_enum)]
        status: Status,

        /// Tier
        #[arg(default_value_t = Tier::Common, long, short, value_enum)]
        tier: Tier,

        /// Create a quest chain
        #[arg(long = "sub", value_name = "QUEST_ID")]
        chain_id: Option<i64>,
    },

    /// Complete a quest
    #[command(long_about)]
    Complete {
        /// Quest ID
        quest_id: i64,
    },

    /// Delete a quest
    #[command(long_about)]
    Delete {
        /// Quest ID
        quest_id: i64,
    },

    /// Show all quests  
    #[command(long_about)]
    Log,

    /// Modify a quest
    #[command(long_about)]
    Modify {
        /// Quest ID
        quest_id: i64,

        /// Objective
        #[arg(long, short, value_enum)]
        objective: Option<String>,

        /// Status
        #[arg(long, short, value_enum)]
        status: Option<Status>,

        /// Tier
        #[arg(long, short, value_enum)]
        tier: Option<Tier>,
    },
}

/// The CLI interpreter.
pub struct Cli;

impl Cli {
    const WARNING_ABANDON_QUEST_CHAIN: &str =
        "Abandoning a main quest will abandon the entire quest chain.";
    const WARNING_COMPLETE_QUEST_CHAIN: &str =
        "Completing a main quest will complete the entire quest chain.";
    const WARNING_DELETE_QUEST: &str =
        "Deleting a quest will permanently delete the quest and its secondary quests.";

    /// Interprets the parsed arguments from the command line.
    pub fn interpret(args: Args) {
        match args.command() {
            Command::Abandon { quest_id } => {
                Self::abandon_quest(quest_id);
            }
            Command::Accept { quest_id } => {
                Self::accept_quest(quest_id);
            }
            Command::Add {
                objective,
                status,
                tier,
                chain_id,
            } => {
                Self::add_quest(objective, status, tier, chain_id);
            }
            Command::Complete { quest_id } => {
                Self::complete_quest(quest_id);
            }
            Command::Delete { quest_id } => {
                Self::delete_quest(quest_id);
            }
            Command::Log => {
                Self::show_quests();
            }
            Command::Modify {
                quest_id,
                objective,
                status,
                tier,
            } => {
                Self::modify_quest(quest_id, objective, status, tier);
            }
        }
    }

    /// Warns the user and asks for confirmation before proceeding.
    fn confirmation_warning(message: &str) -> bool {
        // Warn the user.
        println!("Warning: {}", message);
        print!("Proceed (y/N)? ");

        // Get the user input.
        let mut input = String::new();
        let _ = stdout().flush();
        stdin().read_line(&mut input).expect("invalid string");
        input = input.trim().to_lowercase();

        // Only proceed on "y" or "yes".
        input == "y" || input == "yes"
    }

    /// Abandons the specified quest.
    fn abandon_quest(quest_id: i64) {
        // Get the quest from the database.
        let database = Database::new();
        let conn = database.conn();
        let quest_dao = QuestDao::new(&conn);
        let quest = quest_dao.get_quest(quest_id);

        // Check if the quest is already abandoned.
        if quest.status() == Status::Abandoned {
            println!("Quest {} is already abandoned.", quest_id);
            return;
        }

        // Always ask for confirmation before accepting a quest chain.
        if quest_dao.is_main_quest(quest_id)
            && !Self::confirmation_warning(Self::WARNING_ABANDON_QUEST_CHAIN)
        {
            println!("Quest {} not abandoned.", quest_id);
            return;
        }

        quest_dao.update_chain_status(quest_id, Status::Abandoned);
        println!("Quest {} abandoned.", quest_id);
    }

    /// Accepts the specified quest.
    fn accept_quest(quest_id: i64) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Update the quest status to ongoing.
        let quest_dao = QuestDao::new(&conn);
        let mut quest = quest_dao.get_quest(quest_id);

        // Check if the quest is already accepted.
        if quest.status() == Status::Ongoing {
            println!("Quest is already accepted.");
            return;
        }

        *quest.status_mut() = Status::Ongoing;
        quest_dao.update_quest(&quest);
        println!("Quest {} accepted!", quest_id);
    }

    /// Adds a quest to the log.
    fn add_quest(objective: String, status: Status, tier: Tier, chain_id: Option<i64>) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Construct and save the quest.
        let quest = Quest::new(objective.trim().to_owned(), status, tier, chain_id);
        let quest_dao = QuestDao::new(&conn);
        quest_dao.add_quest(&quest);
    }

    /// Completes a quest.
    fn complete_quest(quest_id: i64) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Update the quest status to completed.
        let quest_dao = QuestDao::new(&conn);
        let quest = quest_dao.get_quest(quest_id);

        if quest.status() == Status::Completed {
            println!("Quest is already completed.");
            return;
        }

        // Always ask for confirmation before completing a quest chain.
        if quest_dao.is_main_quest(quest_id)
            && !Self::confirmation_warning(Self::WARNING_COMPLETE_QUEST_CHAIN)
        {
            println!("Quest {} not completed.", quest_id);
            return;
        }

        quest_dao.update_chain_status(quest_id, Status::Completed);
        println!("Quest {} completed!", quest_id);
    }

    /// Deletes a quest, and its secondary quests, from the log.
    fn delete_quest(quest_id: i64) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Always ask for confirmation before deleting a quest.
        if !Self::confirmation_warning(Self::WARNING_DELETE_QUEST) {
            println!("Quest {} not deleted.", quest_id);
            return;
        }

        // Delete the quest (chain).
        let quest_dao = QuestDao::new(&conn);
        quest_dao.delete_chain(quest_id);
        println!("Quest {} deleted.", quest_id);
    }

    /// Modifies a quest from the log.
    fn modify_quest(
        quest_id: i64,
        objective: Option<String>,
        status: Option<Status>,
        tier: Option<Tier>,
    ) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Update the modified fields.
        let quest_dao = QuestDao::new(&conn);
        let mut quest = quest_dao.get_quest(quest_id);

        if let Some(objective) = objective {
            *quest.objective_mut() = objective;
        }

        if let Some(status) = status {
            *quest.status_mut() = status;
        }

        if let Some(tier) = tier {
            *quest.tier_mut() = tier;
        }

        quest_dao.update_quest(&quest);
        println!("Quest {} modified.", quest_id);
    }

    /// Populates the table with quest chains, where each secondary quest chain
    /// is nested underneath its parent.
    fn populate_table(
        chain: &Chain,
        depth: usize,
        is_terminal: bool,
        is_depth_nested: &mut Vec<bool>,
        table: &mut Table,
    ) {
        // Prepended to the quest objective. Necessary to show the chain connections and depth.
        let mut prefix = String::new();

        // Check if quest chains are nested
        for &is_nested in &is_depth_nested[..depth] {
            if is_nested {
                prefix.push_str("│   ");
            } else {
                prefix.push_str("    ");
            }
        }

        if is_terminal {
            // Close the nested list of chains.
            prefix.push_str("└── ");
        } else {
            prefix.push_str("├── ");
        }

        let row = vec![
            Cell::from(chain.id()),
            Cell::from(prefix + chain.objective()),
            Cell::from(chain.status()),
            Cell::from(chain.tier()),
        ];
        table.add(row);

        if depth < is_depth_nested.len() {
            is_depth_nested[depth] = !is_terminal; // Keep │ if it's not the last item
        } else {
            is_depth_nested.push(!is_terminal);
        }

        let chains = chain.chains();
        let last_index = chains.len().saturating_sub(1);

        for (chain_idx, chain) in chains.iter().enumerate() {
            Self::populate_table(
                chain,
                depth + 1,
                chain_idx == last_index,
                is_depth_nested,
                table,
            );
        }

        if depth < is_depth_nested.len() {
            is_depth_nested[depth] = false;
        }
    }

    /// Shows all quests in the log.
    fn show_quests() {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Get all quest chains from the log.
        let quest_dao = QuestDao::new(&conn);
        let chains = quest_dao.get_all_chains();

        // Populate and show the table.
        let columns: Vec<Cell> = vec![
            Cell::from("ID".underline()),
            Cell::from("Objective".underline()),
            Cell::from("Status".underline()),
            Cell::from("Tier".underline()),
        ];
        let mut table = Table::new(columns);

        for chain in chains {
            let row = vec![
                Cell::from(chain.id()),
                Cell::from(chain.objective()),
                Cell::from(chain.status()),
                Cell::from(chain.tier()),
            ];
            table.add(row);

            for (chain_idx, child_chain) in chain.chains().iter().enumerate() {
                Self::populate_table(
                    child_chain,
                    0,                                     // Chain depth.
                    chain_idx == chain.chains().len() - 1, // Is terminal chain.
                    &mut vec![],
                    &mut table,
                );
            }
        }

        table.show();
    }
}
