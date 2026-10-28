use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use workflow_ide_framework::{
    framework_settings::{FrameworkSettings, StoredResourceEntry, StoredResourceScope},
    project_save_generation::{GenerationResourceStatus, diagnose_generation_project_resources},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "wfide-res-diag-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn root(&self) -> &Path {
        &self.0
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn entry(id: &str, scope: StoredResourceScope, path: &str) -> StoredResourceEntry {
    StoredResourceEntry {
        resource_id: id.to_owned(),
        scope,
        path: PathBuf::from(path),
    }
}

fn diagnose(root: &Path, entries: Vec<StoredResourceEntry>) -> Vec<GenerationResourceStatus> {
    let settings = FrameworkSettings {
        resources: entries,
        ..FrameworkSettings::default()
    };
    diagnose_generation_project_resources(root, &settings)
        .into_iter()
        .map(|diagnostic| diagnostic.status)
        .collect()
}

#[test]
fn res_diag_001_existing_project_file_is_available() {
    let fixture = Fixture::new("available");
    let folder = fixture.root().join("resources/nested");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("file.txt"), b"data").unwrap();
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry(
                "file",
                StoredResourceScope::Project,
                "nested/file.txt"
            )]
        ),
        vec![GenerationResourceStatus::Available]
    );
}

#[test]
fn res_diag_002_missing_project_file_is_missing() {
    let fixture = Fixture::new("missing");
    fs::create_dir_all(fixture.root().join("resources")).unwrap();
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry("missing", StoredResourceScope::Project, "absent.txt")]
        ),
        vec![GenerationResourceStatus::Missing]
    );
}

#[test]
fn res_diag_003_directory_instead_of_file_is_unsafe() {
    let fixture = Fixture::new("directory");
    fs::create_dir_all(fixture.root().join("resources/folder")).unwrap();
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry("folder", StoredResourceScope::Project, "folder")]
        ),
        vec![GenerationResourceStatus::Unsafe]
    );
}

#[test]
fn res_diag_004_non_directory_ancestor_is_unsafe() {
    let fixture = Fixture::new("ancestor");
    fs::create_dir_all(fixture.root().join("resources")).unwrap();
    fs::write(fixture.root().join("resources/parent"), b"not a directory").unwrap();
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry(
                "nested",
                StoredResourceScope::Project,
                "parent/file.txt"
            )]
        ),
        vec![GenerationResourceStatus::Unsafe]
    );
}

#[test]
fn res_diag_005_other_scopes_are_excluded() {
    let fixture = Fixture::new("scopes");
    assert!(
        diagnose(
            fixture.root(),
            vec![
                entry("application", StoredResourceScope::Application, "icon.png"),
                entry("external", StoredResourceScope::External, "external.png"),
            ],
        )
        .is_empty()
    );
}

#[test]
fn res_diag_006_diagnostics_do_not_mutate_files_or_settings() {
    let fixture = Fixture::new("readonly");
    let file = fixture.root().join("resources/file.txt");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, b"unchanged").unwrap();
    let settings = FrameworkSettings {
        resources: vec![entry("file", StoredResourceScope::Project, "file.txt")],
        ..FrameworkSettings::default()
    };
    let original_settings = settings.clone();
    let original_bytes = fs::read(&file).unwrap();
    let result = diagnose_generation_project_resources(fixture.root(), &settings);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].status, GenerationResourceStatus::Available);
    assert_eq!(settings, original_settings);
    assert_eq!(fs::read(&file).unwrap(), original_bytes);
}

#[cfg(unix)]
#[test]
fn res_diag_007_unix_symlink_ancestor_is_unsafe() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("unix-link");
    let outside = fixture.root().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("file.txt"), b"outside").unwrap();
    fs::create_dir_all(fixture.root().join("resources")).unwrap();
    symlink(&outside, fixture.root().join("resources/linked")).unwrap();
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry(
                "linked",
                StoredResourceScope::Project,
                "linked/file.txt"
            )]
        ),
        vec![GenerationResourceStatus::Unsafe]
    );
}

#[cfg(windows)]
#[test]
fn res_diag_007_windows_junction_ancestor_is_unsafe() {
    use std::process::Command;
    let fixture = Fixture::new("win-junction");
    let outside = fixture.root().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("file.txt"), b"outside").unwrap();
    fs::create_dir_all(fixture.root().join("resources")).unwrap();
    let link = fixture.root().join("resources/linked");
    let script = fixture.root().join("create-junction.cmd");
    fs::write(
        &script,
        format!(
            "@echo off\r\nmklink /J \"{}\" \"{}\"\r\n",
            link.display(),
            outside.display()
        ),
    )
    .unwrap();
    let output = Command::new("cmd")
        .args(["/D", "/C"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "mklink failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry(
                "linked",
                StoredResourceScope::Project,
                "linked/file.txt"
            )]
        ),
        vec![GenerationResourceStatus::Unsafe]
    );
    fs::remove_dir(&link).unwrap();
}

#[test]
fn res_diag_008_missing_resource_root_is_missing() {
    let fixture = Fixture::new("missing-root");
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry("missing", StoredResourceScope::Project, "file.txt")]
        ),
        vec![GenerationResourceStatus::Missing]
    );
}

#[test]
fn res_diag_009_non_directory_resource_root_is_unsafe() {
    let fixture = Fixture::new("bad-root");
    fs::write(fixture.root().join("resources"), b"not a directory").unwrap();
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry("bad", StoredResourceScope::Project, "file.txt")]
        ),
        vec![GenerationResourceStatus::Unsafe]
    );
}

#[test]
fn res_diag_010_diagnostics_keep_resource_identity_and_order() {
    let fixture = Fixture::new("identity");
    let settings = FrameworkSettings {
        resources: vec![
            entry("first", StoredResourceScope::Project, "first.txt"),
            entry("excluded", StoredResourceScope::External, "outside.txt"),
            entry("second", StoredResourceScope::Project, "second.txt"),
        ],
        ..FrameworkSettings::default()
    };
    let diagnostics = diagnose_generation_project_resources(fixture.root(), &settings);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].resource_id, "first");
    assert_eq!(diagnostics[1].resource_id, "second");
    assert_eq!(diagnostics[0].reference.path, PathBuf::from("first.txt"));
    assert_eq!(diagnostics[1].reference.path, PathBuf::from("second.txt"));
}

#[cfg(unix)]
#[test]
fn res_diag_011_unix_resource_root_symlink_is_unsafe() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("root-link");
    let outside = fixture.root().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("file.txt"), b"outside").unwrap();
    symlink(&outside, fixture.root().join("resources")).unwrap();
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry("linked", StoredResourceScope::Project, "file.txt")]
        ),
        vec![GenerationResourceStatus::Unsafe]
    );
}

#[cfg(windows)]
#[test]
fn res_diag_011_windows_resource_root_junction_is_unsafe() {
    use std::process::Command;
    let fixture = Fixture::new("root-junction");
    let outside = fixture.root().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("file.txt"), b"outside").unwrap();
    let link = fixture.root().join("resources");
    let script = fixture.root().join("create-root-junction.cmd");
    fs::write(
        &script,
        format!(
            "@echo off\r\nmklink /J \"{}\" \"{}\"\r\n",
            link.display(),
            outside.display()
        ),
    )
    .unwrap();
    let output = Command::new("cmd")
        .args(["/D", "/C"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "mklink failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        diagnose(
            fixture.root(),
            vec![entry("linked", StoredResourceScope::Project, "file.txt")]
        ),
        vec![GenerationResourceStatus::Unsafe]
    );
    fs::remove_dir(&link).unwrap();
}
