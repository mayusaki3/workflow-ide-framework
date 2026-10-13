//! Shared change notification handle for framework and consumer-owned editors.
//! The caller decides whether an edit changes project-persisted data.
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

#[derive(Debug, Clone, Default)]
pub struct UnsavedNotification {
    revision: Arc<AtomicU64>,
}

impl UnsavedNotification {
    /// Mark project-persisted data as modified. Repeated calls are safe.
    pub fn mark_unsaved(&self) {
        self.revision.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::UnsavedNotification;

    #[test]
    fn cloned_handle_notifies_same_revision() {
        let handle = UnsavedNotification::default();
        let clone = handle.clone();
        clone.mark_unsaved();
        assert_eq!(handle.revision(), 1);
        clone.mark_unsaved();
        assert_eq!(handle.revision(), 2);
    }
}
