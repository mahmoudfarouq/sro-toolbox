//! The root command surface, as clap sees it.
//!
//! Only the global options and the top-level areas live here. Each area owns its
//! own subcommand enum and dispatch, in a module beside this one, so adding an
//! area does not grow this file.

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::accounts::AccountsCommand;

/// A toolbox for administering a Silkroad Online private server.
#[derive(Debug, Parser)]
#[command(name = "toolbox", version, about, long_about = None, propagate_version = true)]
pub(crate) struct Cli {
    /// How to render results.
    #[arg(long, short, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub(crate) output: OutputFormat,

    /// ADO connection string for the server's SQL Server.
    ///
    /// Without it the tool runs against an in-process sandbox, so nothing you
    /// type can reach live player data by accident.
    #[arg(
        long,
        global = true,
        env = "TOOLBOX_DATABASE",
        value_name = "CONNECTION_STRING"
    )]
    pub(crate) database: Option<String>,

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

/// Top-level areas, one per subdomain.
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
