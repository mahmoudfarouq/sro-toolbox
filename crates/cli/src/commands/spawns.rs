//! `toolbox spawns ...`

use crate::cli::{OutputFormat, SpawnsCommand};
use crate::error::CliError;
use crate::wiring::Context;

/// Dispatch a spawns subcommand.
///
/// # Errors
///
/// Returns [`CliError::NotImplemented`] for every variant: the command
/// surface is defined, the use cases behind it are not written yet.
pub(crate) async fn handle(
    _context: &Context,
    _format: OutputFormat,
    command: SpawnsCommand,
) -> Result<(), CliError> {
    let name = match command {
        SpawnsCommand::Add { .. } => "spawns add",
        SpawnsCommand::List { .. } => "spawns list",
    };
    Err(CliError::NotImplemented { command: name })
}
