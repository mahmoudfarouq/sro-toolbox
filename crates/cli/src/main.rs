//! Entry point for the `toolbox` binary.
//!
//! Responsibilities, and nothing more: parse arguments, start logging, build the
//! composition root, dispatch, and turn a failure into an exit code.

mod cli;
mod commands;
mod error;
mod output;
mod wiring;

use std::io;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use tracing_subscriber::EnvFilter;

use crate::cli::{Cli, Command};
use crate::error::CliError;
use crate::wiring::Context;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    if cli.dry_run {
        tracing::warn!(
            "--dry-run is accepted but not yet enforced; the only backing store today is \
             in-memory and discards changes when the process exits"
        );
    }

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            error.exit_code()
        }
    }
}

/// Dispatch a parsed command.
async fn run(cli: Cli) -> Result<(), CliError> {
    // Completions need no context, so they are handled before anything is built.
    if let Command::Completions { shell } = cli.command {
        let mut command = Cli::command();
        let name = command.get_name().to_owned();
        generate(shell, &mut command, name, &mut io::stdout());
        return Ok(());
    }

    let context = Context::in_memory();

    match cli.command {
        Command::Accounts { command } => {
            commands::accounts::handle(&context, cli.output, command).await
        }
        Command::Characters { command } => {
            commands::characters::handle(&context, cli.output, command).await
        }
        Command::Items { command } => commands::items::handle(&context, cli.output, command).await,
        Command::Spawns { command } => {
            commands::spawns::handle(&context, cli.output, command).await
        }
        Command::Server { command } => {
            commands::server::handle(&context, cli.output, command).await
        }
        // Handled above, before the context is built.
        Command::Completions { .. } => Ok(()),
    }
}

/// Send logs to stderr, so stdout stays parseable.
fn init_tracing(verbosity: u8) {
    let default = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("TOOLBOX_LOG").unwrap_or_else(|_| EnvFilter::new(default)),
        )
        .with_writer(io::stderr)
        .without_time()
        .init();
}
