//! `toolbox accounts ban`
//!
//! Thin by design: build the use-case input, call it, render the output. Any
//! logic here is logic a web interface would have to duplicate.

use sro_toolbox_domain::accounts::BanAccountInput;

use crate::cli::OutputFormat;
use crate::error::CliError;
use crate::output::render;
use crate::wiring::Context;

/// Block an account.
///
/// # Errors
///
/// Returns [`CliError`] if the use case rejects the input, cannot find the
/// account, or the store fails.
pub(super) async fn run(
    context: &Context,
    format: OutputFormat,
    name: String,
    reason: String,
    days: Option<u32>,
) -> Result<(), CliError> {
    let output = context
        .ban_account()
        .execute(BanAccountInput {
            account_name: name,
            reason,
            days,
        })
        .await?;

    let duration = if output.permanent {
        "permanently".to_owned()
    } else {
        days.map_or_else(|| "temporarily".to_owned(), |d| format!("for {d} day(s)"))
    };
    let note = if output.was_already_banned {
        " (replacing an existing ban)"
    } else {
        ""
    };

    render(
        format,
        &format!("Banned {} {duration}{note}.", output.account_name),
        &output,
    )
}
