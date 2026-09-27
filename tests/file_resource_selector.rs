use std::{fs, path::PathBuf};
use workflow_ide_framework::{
    file_resource_selector::{FileSelectionError, FileSelectionRequest, FileSelectionResult},
    project_resource::{FileTypeFilter, ResourceScope, ResourceSelectionMode},
    resource_state::ResourceRoots,
};

fn setup(name: &str) -> (PathBuf, ResourceRoots) {
    let base=std::env::temp_dir().join(format!("wfide-selector-{name}-{}",std::process::id()));
    let _=fs::remove_dir_all(&base);
    let app=base.join("app"); let project=base.join("project");
    fs::create_dir_all(&app).unwrap(); fs::create_dir_all(&project).unwrap();
    (base,ResourceRoots { application:app, project })
}
fn request(mode: ResourceSelectionMode) -> FileSelectionRequest {
    FileSelectionRequest { mode, filters: vec![FileTypeFilter::new("Images",["png",".jpg"])] }
}

#[test]
fn single_selection_returns_project_relative_reference() {
    let (base,roots)=setup("single"); let file=roots.project.join("image.PNG"); fs::write(&file,b"x").unwrap();
    let result=request(ResourceSelectionMode::Single).validate(&roots,ResourceScope::Project,vec![file]);
    assert!(matches!(result,FileSelectionResult::Selected(ref v) if v.len()==1 && v[0].path==PathBuf::from("image.PNG")));
    let _=fs::remove_dir_all(base);
}

#[test]
fn single_rejects_multiple_files() {
    let (base,roots)=setup("count");
    let result=request(ResourceSelectionMode::Single).validate(&roots,ResourceScope::Project,vec![roots.project.join("a"),roots.project.join("b")]);
    assert!(matches!(result,FileSelectionResult::Error(FileSelectionError::InvalidCount{count:2,..})));
    let _=fs::remove_dir_all(base);
}

#[test]
fn multiple_accepts_more_than_one_file() {
    let (base,roots)=setup("multiple"); let a=roots.project.join("a.png"); let b=roots.project.join("b.jpg");
    fs::write(&a,b"x").unwrap(); fs::write(&b,b"x").unwrap();
    let result=request(ResourceSelectionMode::Multiple).validate(&roots,ResourceScope::Project,vec![a,b]);
    assert!(matches!(result,FileSelectionResult::Selected(ref v) if v.len()==2));
    let _=fs::remove_dir_all(base);
}

#[test]
fn extension_is_revalidated_on_confirm() {
    let (base,roots)=setup("filter"); let file=roots.project.join("bad.txt"); fs::write(&file,b"x").unwrap();
    let result=request(ResourceSelectionMode::Single).validate(&roots,ResourceScope::Project,vec![file.clone()]);
    assert_eq!(result,FileSelectionResult::Error(FileSelectionError::ExtensionNotAllowed(file)));
    let _=fs::remove_dir_all(base);
}

#[test]
fn project_scope_rejects_file_outside_project_root() {
    let (base,roots)=setup("outside"); let outside=base.join("outside.png"); fs::write(&outside,b"x").unwrap();
    let result=request(ResourceSelectionMode::Single).validate(&roots,ResourceScope::Project,vec![outside.clone()]);
    assert_eq!(result,FileSelectionResult::Error(FileSelectionError::OutsideRoot{scope:ResourceScope::Project,path:outside}));
    let _=fs::remove_dir_all(base);
}

#[test]
fn external_selection_returns_absolute_reference() {
    let (base,roots)=setup("external"); let outside=base.join("outside.png"); fs::write(&outside,b"x").unwrap();
    let result=request(ResourceSelectionMode::Single).validate(&roots,ResourceScope::External,vec![outside]);
    assert!(matches!(result,FileSelectionResult::Selected(ref v) if v[0].path.is_absolute()));
    let _=fs::remove_dir_all(base);
}

#[test]
fn directory_is_not_a_selectable_file() {
    let (base,roots)=setup("directory"); let dir=roots.project.join("folder.png"); fs::create_dir_all(&dir).unwrap();
    let result=request(ResourceSelectionMode::Single).validate(&roots,ResourceScope::Project,vec![dir.clone()]);
    assert_eq!(result,FileSelectionResult::Error(FileSelectionError::NotRegularFile(dir)));
    let _=fs::remove_dir_all(base);
}
