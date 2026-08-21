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
