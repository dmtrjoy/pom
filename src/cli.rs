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
}

/// The CLI interpretter.
pub struct Cli;

impl Cli {
    /// Interprets the parse arguments from the command line.
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
        let mut quest = quest_dao.get(quest_id);

        // Check if the quest is already abandoned.
        if quest.status() == Status::Abandoned {
            println!("Quest {} is already abandoned.", quest_id);
            return;
        }

        // Ask for confirmation before abandoning a main quest with at least one secondary quest.
        // if quest.is_main() && quest_dao.num_quests(quest.id()) > 0 {
        //     let message = "Abandoning a main quest will abandon the entire quest chain.";
        //     if !Self::confirmation_warning(message) {
        //         println!("Quest {} not abandoned.", quest_id);
        //         return;
        //     }
        // }

        *quest.status_mut() = Status::Abandoned;
        quest_dao.update(&quest);
        println!("Abandoned quest {}!", quest_id);
    }

    /// Accepts the specified quest.
    fn accept_quest(quest_id: i64) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Update the quest status to ongoing.
        let quest_dao = QuestDao::new(&conn);
        let mut quest = quest_dao.get(quest_id);

        if quest.status() == Status::Ongoing {
            println!("Quest is already accepted");
            return;
        }

        *quest.status_mut() = Status::Ongoing;
        quest_dao.update(&quest);
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
        quest_dao.add(&quest);
    }

    /// Completes a quest.
    fn complete_quest(quest_id: i64) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Update the quest status to completed.
        let quest_dao = QuestDao::new(&conn);
        let mut quest = quest_dao.get(quest_id);

        if quest.status() == Status::Completed {
            println!("Quest is already completed");
            return;
        }

        *quest.status_mut() = Status::Completed;
        quest_dao.update(&quest);
        println!("Quest {} completed!", quest_id);
    }

    /// Deletes a quest from the log.
    fn delete_quest(quest_id: i64) {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Delete the quest.
        let quest_dao = QuestDao::new(&conn);
        quest_dao.delete(quest_id);
    }

    /// Shows all quests in the log.
    fn show_quests() {
        // Open the database connection.
        let database = Database::new();
        let conn = database.conn();

        // Get all quests from the log.
        let quest_dao = QuestDao::new(&conn);
        let chains = quest_dao.chains();

        let cols: Vec<Cell> = vec![
            Cell::rich("ID".to_owned().underline()),
            Cell::rich("Objective".to_owned().underline()),
            Cell::rich("Status".to_owned().underline()),
            Cell::rich("Tier".to_owned().underline()),
        ];
        let mut table = Table::new(cols);

        for chain in chains {
            let row = vec![
                Cell::plain(chain.id().to_string()),
                Cell::plain("".to_owned() + chain.objective()),
                Cell::plain(chain.status().to_string()),
                Cell::rich(chain.tier().to_colored_string()),
            ];
            table.add(row);

            let mut chain_idx = 0;
            for sub in chain.chains() {
                Self::rec(sub, 0, chain_idx == chain.chains().len() - 1, &mut table, &mut vec![]);
                chain_idx += 1;
            }
        }

        table.show();
    }

    fn rec(chain: &Chain, depth: usize , is_last: bool, table: &mut Table, line_active: &mut Vec<bool>) {
        let mut prefix = String::new();

        for &active in &line_active[..depth] {
            if active {
                prefix.push_str("│   ");
            } else {
                prefix.push_str("    ");
            }
        }

        if is_last {
            prefix.push_str("└── ");
        } else {
            prefix.push_str("├── ");
        }

        let row = vec![
            Cell::rich(chain.id().to_string().black()),
            Cell::plain(prefix + chain.objective()),
            Cell::plain(chain.status().to_string()),
            Cell::rich(chain.tier().to_colored_string()),
        ];
        table.add(row);

        if depth < line_active.len() {
            line_active[depth] = !is_last; // Keep │ if it's not the last item
        } else {
            line_active.push(!is_last);
        }

        let chains = chain.chains();
        let last_index = chains.len().saturating_sub(1);

        for (chain_idx, chain) in chains.iter().enumerate() {
            Self::rec(chain, depth + 1, chain_idx == last_index, table, line_active);
        }

        if depth < line_active.len() {
            line_active[depth] = false;
        }
    }
}
