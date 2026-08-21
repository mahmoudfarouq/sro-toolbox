//! `toolbox accounts ...`

use sro_toolbox_application::use_cases::BanAccountInput;

use crate::cli::{AccountsCommand, OutputFormat};
use crate::error::CliError;
use crate::output::render;
use crate::wiring::Context;

/// Dispatch an accounts subcommand.
///
/// # Errors
///
/// Returns [`CliError`] if the use case fails or the command is a stub.
pub(crate) async fn handle(
    context: &Context,
    format: OutputFormat,
    command: AccountsCommand,
) -> Result<(), CliError> {
    match command {
        AccountsCommand::Ban { name, reason, days } => {
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
