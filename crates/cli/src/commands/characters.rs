//! `toolbox characters ...`

use crate::cli::{CharactersCommand, OutputFormat};
use crate::error::CliError;
use crate::wiring::Context;

/// Dispatch a characters subcommand.
///
/// # Errors
///
/// Returns [`CliError::NotImplemented`] for every variant: the command
/// surface is defined, the use cases behind it are not written yet.
pub(crate) async fn handle(
    _context: &Context,
    _format: OutputFormat,
    command: CharactersCommand,
) -> Result<(), CliError> {
    let name = match command {
        CharactersCommand::Show { .. } => "characters show",
        CharactersCommand::SetLevel { .. } => "characters set-level",
        CharactersCommand::Teleport { .. } => "characters teleport",
    };
    Err(CliError::NotImplemented { command: name })
}
