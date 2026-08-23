//! Recording what the operator did.
//!
//! Cross-cutting rather than owned by one subdomain, so it lives at the crate
//! root. Every mutating use case writes an entry: the tool edits live player
//! data, so "who changed what" is a requirement, not a nicety.

use async_trait::async_trait;
use serde::Serialize;

use crate::error::RepositoryResult;

/// One recorded administrative action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditEntry {
    /// What was done, as a stable machine-readable key such as `accounts.ban`.
    pub action: &'static str,
    /// What it was done to.
    pub target: String,
    /// Free-text detail for a human reading the log later.
    pub detail: String,
}

/// Where administrative actions are recorded.
#[async_trait]
pub trait AuditLog: Send + Sync {
    /// Record an action.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::RepositoryError`] if the entry cannot be written.
    async fn record(&self, entry: AuditEntry) -> RepositoryResult<()>;
}
