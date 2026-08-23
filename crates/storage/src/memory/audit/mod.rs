//! Audit entries held in a vector.

mod record;

use std::sync::Mutex;

use async_trait::async_trait;
use sro_toolbox_domain::RepositoryResult;
use sro_toolbox_domain::audit::{AuditEntry, AuditLog};

/// The shared state each operation acts on.
pub(super) type Entries = Mutex<Vec<AuditEntry>>;

/// An audit log that keeps entries in memory and emits each one at info level.
#[derive(Debug, Default)]
pub struct InMemoryAuditLog {
    entries: Entries,
}

impl InMemoryAuditLog {
    /// An empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Everything recorded so far, oldest first.
    #[must_use]
    pub fn entries(&self) -> Vec<AuditEntry> {
        self.entries
            .lock()
            .expect("lock is never poisoned: no panics while held")
            .clone()
    }
}

#[async_trait]
impl AuditLog for InMemoryAuditLog {
    async fn record(&self, entry: AuditEntry) -> RepositoryResult<()> {
        record::execute(&self.entries, entry)
    }
}
