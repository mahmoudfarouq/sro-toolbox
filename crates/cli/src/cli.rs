//! The command surface, as clap sees it.
//!
//! This module is the only place that knows about command-line syntax. It
//! deliberately holds no behaviour: parsing produces plain values, and
//! [`crate::commands`] turns those into use-case inputs.

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// A toolbox for administering a Silkroad Online private server.
#[derive(Debug, Parser)]
#[command(name = "toolbox", version, about, long_about = None, propagate_version = true)]
pub(crate) struct Cli {
    /// How to render results.
    #[arg(long, short, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub(crate) output: OutputFormat,

    /// Describe what would happen without writing anything.
    #[arg(long, global = true)]
    pub(crate) dry_run: bool,

    /// Increase log verbosity. Repeat for more.
    #[arg(long, short, global = true, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// The area to operate on.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// How results are rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    /// Human-readable lines.
    Text,
    /// A single JSON document, for scripting.
    Json,
}

/// Top-level areas, one per domain concern.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Login accounts: bans, privileges, silk.
    Accounts {
        /// What to do.
        #[command(subcommand)]
        command: AccountsCommand,
    },

    /// Characters: levels, position, inventory.
    Characters {
        /// What to do.
        #[command(subcommand)]
        command: CharactersCommand,
    },

    /// The item catalogue and player-held items.
    Items {
        /// What to do.
        #[command(subcommand)]
        command: ItemsCommand,
    },

    /// Monster spawn points.
    Spawns {
        /// What to do.
        #[command(subcommand)]
        command: SpawnsCommand,
    },

    /// The running server itself.
    Server {
        /// What to do.
        #[command(subcommand)]
        command: ServerCommand,
    },

    /// Print a shell completion script.
    Completions {
        /// Which shell to generate for.
        shell: Shell,
    },
}

/// Account operations.
#[derive(Debug, Subcommand)]
pub(crate) enum AccountsCommand {
    /// Block an account from logging in.
    Ban {
        /// The account's login name.
        name: String,

        /// Why it is being blocked.
        #[arg(long, short)]
        reason: String,

        /// Days the block lasts. Omit for a permanent ban.
        #[arg(long, short)]
        days: Option<u32>,
    },

    /// Lift a block on an account.
    Unban {
        /// The account's login name.
        name: String,
    },

    /// Show an account's current state.
    Show {
        /// The account's login name.
        name: String,
    },

    /// Add silk to an account.
    GrantSilk {
        /// The account's login name.
        name: String,

        /// How much silk to add.
        #[arg(long, short)]
        amount: u32,

        /// Which silk pool to credit.
        #[arg(long, short, default_value = "purchased")]
        kind: String,
    },
}

/// Character operations.
#[derive(Debug, Subcommand)]
pub(crate) enum CharactersCommand {
    /// Show a character's current state.
    Show {
        /// The character's name.
        name: String,
    },

    /// Set a character's level.
    SetLevel {
        /// The character's name.
        name: String,

        /// The level to set.
        #[arg(long, short)]
        level: u8,
    },

    /// Move a character to a region.
    Teleport {
        /// The character's name.
        name: String,

        /// The destination region.
        #[arg(long, short)]
        region: u16,
    },
}

/// Item catalogue operations.
#[derive(Debug, Subcommand)]
pub(crate) enum ItemsCommand {
    /// Add a new item definition.
    Create {
        /// The item's code name, such as `ITEM_CH_SWORD_01_A`.
        #[arg(long, short)]
        code_name: String,

        /// The largest stack allowed in one slot.
        #[arg(long, default_value_t = 1)]
        max_stack: u16,
    },

    /// Show an item definition.
    Show {
        /// The item's code name.
        code_name: String,
    },

    /// Give an item to a character.
    Grant {
        /// The character to give it to.
        character: String,

        /// The item's code name.
        #[arg(long, short)]
        code_name: String,

        /// How many to give.
        #[arg(long, short, default_value_t = 1)]
        quantity: u16,
    },
}

/// Spawn operations.
#[derive(Debug, Subcommand)]
pub(crate) enum SpawnsCommand {
    /// Add a spawn point for a monster.
    Add {
        /// The monster's code name.
        #[arg(long, short)]
        monster: String,

        /// The region to spawn it in.
        #[arg(long, short)]
        region: u16,

        /// How many to keep alive at once.
        #[arg(long, short, default_value_t = 1)]
        count: u16,
    },

    /// List spawn points.
    List {
        /// Restrict to one region.
        #[arg(long, short)]
        region: Option<u16>,
    },
}

/// Server operations.
#[derive(Debug, Subcommand)]
pub(crate) enum ServerCommand {
    /// Report whether the server is reachable and who is online.
    Status,

    /// Send a notice to every player.
    Notice {
        /// The message to send.
        message: String,
    },
}
