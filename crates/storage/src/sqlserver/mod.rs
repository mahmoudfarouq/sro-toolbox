//! Adapters talking to the server's own SQL Server databases.
//!
//! Targets `SRO_VT_ACCOUNT`. See `docs/sqlserver-schema.md` for the tables
//! involved and how confident we are about each of them.
//!
//! ## How these are kept testable
//!
//! Every operation separates what can be checked without a database from what
//! cannot:
//!
//! - the SQL itself, as a constant;
//! - a plain row struct, and a pure function mapping it to a domain type;
//! - a pure function turning a domain type into query parameters;
//! - a thin `execute` that binds and runs.
//!
//! The first three are unit-tested here. Only `execute` needs a live server,
//! and that is covered by tests gated behind `TOOLBOX_TEST_DATABASE`.

pub mod account;
pub mod schema;

use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use sro_toolbox_domain::{RepositoryError, RepositoryResult};

pub use account::SqlServerAccountRepository;

/// A connected client, over a Tokio socket adapted to the futures I/O traits
/// tiberius expects.
pub type SqlClient = Client<Compat<TcpStream>>;

/// How to reach the databases.
///
/// Holds the parsed configuration rather than a live connection, so building
/// one cannot fail at a distance from where the string was supplied.
#[derive(Debug, Clone)]
pub struct SqlServerContext {
    config: Config,
}

impl SqlServerContext {
    /// Parse an ADO-style connection string.
    ///
    /// For example:
    /// `Server=tcp:127.0.0.1,1433;Database=SRO_VT_ACCOUNT;User Id=sa;Password=...;TrustServerCertificate=true`
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the string cannot be parsed.
    pub fn from_ado_string(connection_string: &str) -> RepositoryResult<Self> {
        let config = Config::from_ado_string(connection_string).map_err(|error| {
            RepositoryError::Storage(format!("invalid connection string: {error}"))
        })?;
        Ok(Self { config })
    }

    /// Open a connection.
    ///
    /// One connection per call. That is adequate for a CLI issuing a handful of
    /// statements and deliberately not a pool; pooling belongs here once a
    /// long-lived interface needs it.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the server cannot be reached or
    /// refuses the login.
    pub async fn connect(&self) -> RepositoryResult<SqlClient> {
        let tcp = TcpStream::connect(self.config.get_addr())
            .await
            .map_err(|error| {
                RepositoryError::Storage(format!("cannot reach the database: {error}"))
            })?;
        tcp.set_nodelay(true).map_err(|error| {
            RepositoryError::Storage(format!("cannot configure the socket: {error}"))
        })?;

        Client::connect(self.config.clone(), tcp.compat_write())
            .await
            .map_err(|error| RepositoryError::Storage(format!("cannot log in: {error}")))
    }
}

/// Turn a tiberius failure into a repository failure.
pub(crate) fn storage_error(operation: &str, error: &tiberius::error::Error) -> RepositoryError {
    RepositoryError::Storage(format!("{operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_connection_string() {
        let context = SqlServerContext::from_ado_string(
            "Server=tcp:127.0.0.1,1433;Database=SRO_VT_ACCOUNT;User Id=sa;Password=secret;TrustServerCertificate=true",
        );
        assert!(context.is_ok());
    }

    #[test]
    fn rejects_a_malformed_connection_string() {
        let error = SqlServerContext::from_ado_string("this is not a connection string")
            .expect_err("should fail");
        assert!(matches!(error, RepositoryError::Storage(_)));
    }
}
