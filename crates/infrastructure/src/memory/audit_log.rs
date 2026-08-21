//! Audit entries held in a vector, and mirrored to `tracing`.

use std::sync::Mutex;

use async_trait::async_trait;
use sro_toolbox_application::error::PortResult;
use sro_toolbox_application::ports::{AuditEntry, AuditLog};

/// An audit log that keeps entries in memory and emits each one at info level.
#[derive(Debug, Default)]
pub struct InMemoryAuditLog {
    entries: Mutex<Vec<AuditEntry>>,
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
    async fn record(&self, entry: AuditEntry) -> PortResult<()> {
        tracing::info!(
            action = entry.action,
            target = %entry.target,
            detail = %entry.detail,
            "administrative action"
        );
        self.entries
            .lock()
            .expect("lock is never poisoned: no panics while held")
            .push(entry);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn records_entries_in_order() {
        let log = InMemoryAuditLog::new();

        for action in ["accounts.ban", "accounts.unban"] {
            log.record(AuditEntry {
                action,
                target: "player01".into(),
                detail: String::new(),
            })
            .await
            .expect("no failure");
        }

        let entries = log.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].action, "accounts.ban");
        assert_eq!(entries[1].action, "accounts.unban");
    }
}
