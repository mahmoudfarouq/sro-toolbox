//! `toolbox accounts ...`
//!
//! Mirrors `sro_toolbox_domain::accounts`: the subcommand surface here, one file
//! per action beside it. Adding an action is a variant plus a file, and the
//! compiler finds the match that needs updating.

mod ban;

use clap::Subcommand;

use crate::cli::OutputFormat;
use crate::error::CliError;
use crate::wiring::Context;

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

/// Dispatch an accounts subcommand to its handler.
///
/// # Errors
///
/// Returns [`CliError`] if the use case fails or the action is a stub.
pub(crate) async fn handle(
    context: &Context,
    format: OutputFormat,
    command: AccountsCommand,
) -> Result<(), CliError> {
    match command {
        AccountsCommand::Ban { name, reason, days } => {
            ban::run(context, format, name, reason, days).await
        }
        AccountsCommand::Unban { .. } => Err(CliError::NotImplemented {
            command: "accounts unban",
        }),
        AccountsCommand::Show { .. } => Err(CliError::NotImplemented {
            command: "accounts show",
        }),
        AccountsCommand::GrantSilk { .. } => Err(CliError::NotImplemented {
            command: "accounts grant-silk",
        }),
    }
}
