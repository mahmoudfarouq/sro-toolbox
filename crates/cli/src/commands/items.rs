//! `toolbox items ...`

use crate::cli::{ItemsCommand, OutputFormat};
use crate::error::CliError;
use crate::wiring::Context;

/// Dispatch a items subcommand.
///
/// # Errors
///
/// Returns [`CliError::NotImplemented`] for every variant: the command
/// surface is defined, the use cases behind it are not written yet.
pub(crate) async fn handle(
    _context: &Context,
    _format: OutputFormat,
    command: ItemsCommand,
) -> Result<(), CliError> {
    let name = match command {
        ItemsCommand::Create { .. } => "items create",
        ItemsCommand::Show { .. } => "items show",
        ItemsCommand::Grant { .. } => "items grant",
    };
    Err(CliError::NotImplemented { command: name })
}
