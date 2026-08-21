//! `toolbox server ...`

use crate::cli::{OutputFormat, ServerCommand};
use crate::error::CliError;
use crate::wiring::Context;

/// Dispatch a server subcommand.
///
/// # Errors
///
/// Returns [`CliError::NotImplemented`] for every variant: the command
/// surface is defined, the use cases behind it are not written yet.
pub(crate) async fn handle(
    _context: &Context,
    _format: OutputFormat,
    command: ServerCommand,
) -> Result<(), CliError> {
    let name = match command {
        ServerCommand::Status => "server status",
        ServerCommand::Notice { .. } => "server notice",
    };
    Err(CliError::NotImplemented { command: name })
}
