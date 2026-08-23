//! Record an administrative action.

use sro_toolbox_domain::RepositoryResult;
use sro_toolbox_domain::audit::AuditEntry;

use super::Entries;

/// Append `entry`, mirroring it to `tracing` so it is visible without the
/// caller having to ask the log for its contents.
pub(super) fn execute(entries: &Entries, entry: AuditEntry) -> RepositoryResult<()> {
    tracing::info!(
        action = entry.action,
        target = %entry.target,
        detail = %entry.detail,
        "administrative action"
    );
    entries
        .lock()
        .expect("lock is never poisoned: no panics while held")
        .push(entry);
    Ok(())
}

#[cfg(test)]
mod tests {
    use sro_toolbox_domain::audit::AuditLog;

    use super::super::InMemoryAuditLog;
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
