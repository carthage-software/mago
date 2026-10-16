#![allow(clippy::unwrap_used)]

use std::path::Path;

use mago_database::file::{File, FileId, FileType};

use super::{file_id_for, logical_name_for};

fn assert_loaded_identity(workspace: &Path, path: &Path, file_type: FileType) {
    let loaded = File::read(workspace, path, file_type).unwrap();
    assert_eq!(logical_name_for(workspace, path), loaded.name.as_ref());
    assert_eq!(file_id_for(workspace, path), loaded.id);
}

#[test]
fn path_names_match_disk_file_identities() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = directory.path().join("project");
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    let local = workspace.join("src/Local.php");
    let external = directory.path().join("External.php");
    std::fs::write(&local, "<?php").unwrap();
    std::fs::write(&external, "<?php").unwrap();

    assert_eq!(logical_name_for(&workspace, &local), b"src/Local.php");
    for file_type in [FileType::Host, FileType::Vendored, FileType::Patch] {
        assert_loaded_identity(&workspace, &local, file_type);
        assert_loaded_identity(&workspace, &external, file_type);
    }
}

#[cfg(unix)]
#[test]
fn native_path_bytes_survive_without_utf8_conversion() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let directory = tempfile::tempdir().unwrap();
    let workspace = directory.path().join(OsStr::from_bytes(b"project-\xff"));
    let local = workspace.join(OsStr::from_bytes(b"Local-\xfe.php"));
    let external = directory.path().join(OsStr::from_bytes(b"External-\xfe.php"));

    assert_eq!(logical_name_for(&workspace, &local), b"Local-\xfe.php");
    assert_eq!(file_id_for(&workspace, &local), FileId::new(b"Local-\xfe.php"));
    assert_eq!(logical_name_for(&workspace, &external), external.as_os_str().as_bytes());
    assert_eq!(file_id_for(&workspace, &external), FileId::new(external.as_os_str().as_bytes()));
}

#[cfg(windows)]
#[test]
fn windows_names_use_forward_slashes() {
    let workspace = Path::new(r"C:\project");
    for (path, expected) in [
        (Path::new(r"C:\project\src\Local.php"), b"src/Local.php".as_slice()),
        (Path::new(r"D:\patches\External.php"), b"D:/patches/External.php".as_slice()),
    ] {
        assert_eq!(logical_name_for(workspace, path), expected);
        assert_eq!(file_id_for(workspace, path), FileId::new(expected));
    }
}
