//! Rendering results.
//!
//! Confined to the CLI crate on purpose: how a result is displayed is an
//! interface concern, and a web front end would render the same use-case output
//! entirely differently.

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::error::CliError;

/// Render `value` to stdout in the requested format.
///
/// `summary` is the human-readable line used for [`OutputFormat::Text`]; it is
/// ignored for JSON, where the structured value is what callers want.
///
/// # Errors
///
/// Returns [`CliError::Render`] if the value cannot be serialised.
pub(crate) fn render<T: Serialize>(
    format: OutputFormat,
    summary: &str,
    value: &T,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Text => println!("{summary}"),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value)?),
    }
    Ok(())
}
