use mago_database::file::FileType;
use mago_database::{Database, DatabaseConfiguration, DatabaseReader};
use mago_server::workspace::paths::builtin_file_at;

use super::*;

#[test]
fn sources_remain_queryable_across_sessions_and_keep_workspace_ownership() {
    let directory = tempfile::tempdir().unwrap();
    let configuration = DatabaseConfiguration::new(directory.path(), vec![], vec![], vec![], vec![]).into_static();
    let mut database = Database::new(configuration);
    let id = database.add(File::new(
        b"Core/strlen.php".as_slice().into(),
        FileType::Builtin,
        None,
        b"<?php function strlen(string $s): int {}".as_slice().into(),
    ));
    let file = database.get(&id).unwrap();
    let mut sources = SourceFiles::new(directory.path());
    assert_eq!(
        sources.root.parent(),
        Some(cache_root().join(VERSION).join(FileId::new(PRELUDE).to_string()).as_path())
    );
    let uri = sources.uri(&file).unwrap();
    let path = uri.to_file_path().unwrap();
    assert_eq!(path.as_ref(), path.canonicalize().unwrap());
    for root in [&sources.root, path.parent().unwrap(), path.as_ref()] {
        assert_eq!(
            SourceFiles::workspace_for_cached_path(root).unwrap(),
            Some(directory.path().canonicalize().unwrap())
        );
    }
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    let marker = sources.root.join(WORKSPACE_MARKER);
    let marker_modified = std::fs::metadata(&marker).unwrap().modified().unwrap();
    let mut reopened = SourceFiles::new(directory.path());
    assert_eq!(builtin_file_at(&database, &reopened.root, &path).unwrap().id, id);
    assert_eq!(reopened.uri(&file), Some(uri.clone()));
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(std::fs::metadata(&marker).unwrap().modified().unwrap(), marker_modified);
    let other = SourceFiles::new(&directory.path().join("other"));
    assert!(builtin_file_at(&database, &other.root, &path).is_none());
    assert!(builtin_file_at(&database, &reopened.root, &reopened.root.join("../Core/strlen.php")).is_none());
    #[cfg(windows)]
    {
        for path in [path.as_ref(), marker.as_path()] {
            let mut permissions = std::fs::metadata(path).unwrap().permissions();
            permissions.set_readonly(false);
            std::fs::set_permissions(path, permissions).unwrap();
        }
    }
    std::fs::remove_dir_all(&sources.root).unwrap();
}

#[test]
fn cached_workspace_roots_require_a_valid_matching_origin() {
    let directory = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let mut sources = SourceFiles::new(directory.path());
    let file = File::new(b"Core/test.php".as_slice().into(), FileType::Builtin, None, b"<?php".as_slice().into());
    let uri = sources.uri(&file).unwrap();
    let path = uri.to_file_path().unwrap();
    let marker = sources.root.join(WORKSPACE_MARKER);
    assert_eq!(SourceFiles::workspace_for_cached_path(directory.path()).unwrap(), None);
    #[cfg(windows)]
    {
        let mut permissions = std::fs::metadata(&marker).unwrap().permissions();
        permissions.set_readonly(false);
        std::fs::set_permissions(&marker, permissions).unwrap();
    }
    std::fs::remove_file(&marker).unwrap();
    assert!(SourceFiles::workspace_for_cached_path(&path).is_err());
    for contents in ["invalid URI".to_owned(), Uri::from_file_path(other.path()).unwrap().to_string()] {
        std::fs::write(&marker, contents).unwrap();
        assert!(SourceFiles::workspace_for_cached_path(&path).is_err());
    }
    std::fs::write(&marker, Uri::from_file_path(&sources.workspace).unwrap().as_str()).unwrap();
    assert!(SourceFiles::workspace_for_cached_path(&sources.root.join("../Core/test.php")).is_err());
    let cache = cache_root();
    let prelude = FileId::new(PRELUDE).to_string();
    let workspace_id = sources.root.file_name().unwrap();
    for prefix in [
        Path::new(&prelude).to_path_buf(),
        Path::new("0.0.0-test").join(&prelude),
        Path::new(VERSION).join("old-prelude"),
    ] {
        let foreign = cache.join(prefix).join(workspace_id);
        std::fs::create_dir_all(&foreign).unwrap();
        std::fs::write(foreign.join(WORKSPACE_MARKER), Uri::from_file_path(&sources.workspace).unwrap().as_str())
            .unwrap();
        assert!(
            SourceFiles::workspace_for_cached_path(&foreign.join("Core/test.php")).is_err(),
            "{}",
            foreign.display()
        );
        std::fs::remove_dir_all(&foreign).unwrap();
        for parent in foreign.ancestors().skip(1).take_while(|parent| *parent != cache) {
            if std::fs::remove_dir(parent).is_err() {
                break;
            }
        }
    }
    drop(directory);
    assert!(SourceFiles::workspace_for_cached_path(&path).is_err());
    #[cfg(windows)]
    {
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(false);
        std::fs::set_permissions(&path, permissions).unwrap();
    }
    std::fs::remove_dir_all(&sources.root).unwrap();
}

#[test]
fn source_names_cannot_escape_the_cache() {
    let directory = tempfile::tempdir().unwrap();
    let mut sources = SourceFiles::new(directory.path());
    for name in ["", "../outside.php", "/absolute.php", "Core/../../outside.php"] {
        let file = File::new(name.as_bytes().to_vec().into(), FileType::Builtin, None, b"<?php".as_slice().into());
        assert!(sources.uri(&file).is_none(), "{name}");
    }
    assert!(!sources.root.exists());
}
