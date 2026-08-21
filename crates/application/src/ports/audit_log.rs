//! Recording what the operator did.

use async_trait::async_trait;
use serde::Serialize;

use crate::error::PortResult;

/// One recorded administrative action.
///
/// Every mutating use case writes one of these. The tool edits live player data,
/// so "who changed what" is a requirement rather than a nicety.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditEntry {
    /// What was done, as a stable machine-readable key.
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
    /// Returns [`crate::error::PortError`] if the entry cannot be written.
    async fn record(&self, entry: AuditEntry) -> PortResult<()>;
}
