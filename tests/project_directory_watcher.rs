use std::path::PathBuf;
use std::time::{Duration, Instant};
use workflow_ide_framework::project_directory_watcher::ProjectDirectoryWatcher;
use workflow_ide_framework::project_panel_api::ProjectPanelRefreshHandle;

fn temporary_project() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "wfide-watch-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(root.join("models")).unwrap();
    root
}

fn await_refresh(handle: &ProjectPanelRefreshHandle) -> bool {
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until {
        if handle.take_pending() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

#[test]
fn pp_dir_005_os_notifications_for_create_rename_and_delete() {
    let root = temporary_project();
    let handle = ProjectPanelRefreshHandle::default();
    let watcher =
        ProjectDirectoryWatcher::start(&root, &[PathBuf::from("models")], handle.clone()).unwrap();
    assert_eq!(watcher.watched_paths().len(), 1);

    std::fs::write(root.join("models/first.urdf"), "robot").unwrap();
    assert!(await_refresh(&handle), "create did not trigger refresh");
    std::thread::sleep(Duration::from_millis(150));
    handle.take_pending();

    std::fs::rename(
        root.join("models/first.urdf"),
        root.join("models/second.urdf"),
    )
    .unwrap();
    assert!(await_refresh(&handle), "rename did not trigger refresh");
    std::thread::sleep(Duration::from_millis(150));
    handle.take_pending();

    std::fs::remove_file(root.join("models/second.urdf")).unwrap();
    assert!(await_refresh(&handle), "delete did not trigger refresh");
    drop(watcher);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pp_dir_005_refresh_requests_coalesce() {
    let root = temporary_project();
    let handle = ProjectPanelRefreshHandle::default();
    let watcher =
        ProjectDirectoryWatcher::start(&root, &[PathBuf::from("models")], handle.clone()).unwrap();

    for index in 0..5 {
        std::fs::write(root.join(format!("models/{index}.urdf")), "robot").unwrap();
    }
    assert!(await_refresh(&handle));
    std::thread::sleep(Duration::from_millis(300));
    handle.take_pending();
    assert!(!handle.take_pending(), "pending refresh was not coalesced");
    drop(watcher);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pp_dir_005_dropped_watcher_stops_notifications() {
    let root = temporary_project();
    let handle = ProjectPanelRefreshHandle::default();
    let watcher =
        ProjectDirectoryWatcher::start(&root, &[PathBuf::from("models")], handle.clone()).unwrap();
    drop(watcher);
    handle.take_pending();
    std::fs::write(root.join("models/after-drop.urdf"), "robot").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    assert!(!handle.take_pending(), "dropped watcher still notified");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pp_dir_005_watcher_identity_tracks_project_and_mounts() {
    let first = temporary_project();
    let second = temporary_project();
    let handle = ProjectPanelRefreshHandle::default();
    let watcher =
        ProjectDirectoryWatcher::start(&first, &[PathBuf::from("models")], handle).unwrap();

    assert!(watcher.matches(&first, &[PathBuf::from("models")]));
    assert!(!watcher.matches(&second, &[PathBuf::from("models")]));
    assert!(!watcher.matches(&first, &[PathBuf::from("other")]));
    drop(watcher);
    std::fs::remove_dir_all(first).unwrap();
    std::fs::remove_dir_all(second).unwrap();
}

#[test]
fn pp_dir_005_missing_mount_appears_and_notifies() {
    let root = temporary_project();
    let handle = ProjectPanelRefreshHandle::default();
    let watcher =
        ProjectDirectoryWatcher::start(&root, &[PathBuf::from("future")], handle.clone()).unwrap();
    assert_eq!(watcher.watched_paths().len(), 1);

    std::fs::create_dir(root.join("future")).unwrap();
    assert!(
        await_refresh(&handle),
        "creating a missing mount did not notify"
    );
    drop(watcher);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pp_dir_005_mount_deletion_and_recreation_notifies() {
    let root = temporary_project();
    let handle = ProjectPanelRefreshHandle::default();
    let watcher =
        ProjectDirectoryWatcher::start(&root, &[PathBuf::from("models")], handle.clone()).unwrap();

    std::fs::remove_dir(root.join("models")).unwrap();
    assert!(await_refresh(&handle), "removing mount did not notify");
    std::thread::sleep(Duration::from_millis(150));
    handle.take_pending();

    std::fs::create_dir(root.join("models")).unwrap();
    assert!(await_refresh(&handle), "recreating mount did not notify");
    std::thread::sleep(Duration::from_millis(150));
    handle.take_pending();

    std::fs::write(root.join("models/restored.urdf"), "robot").unwrap();
    assert!(
        await_refresh(&handle),
        "new file in restored mount did not notify"
    );
    drop(watcher);
    std::fs::remove_dir_all(root).unwrap();
}
