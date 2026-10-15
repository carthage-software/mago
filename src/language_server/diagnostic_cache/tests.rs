use mago_database::DatabaseConfiguration;
use mago_database::file::{File, FileType};

use super::*;

#[test]
fn clean_file_changes_keep_empty_output_without_retaining_uris() {
    let directory = tempfile::tempdir().unwrap();
    let configuration = DatabaseConfiguration::new(directory.path(), vec![], vec![], vec![], vec![]);
    let mut database = Database::new(configuration);
    let mut cache = DiagnosticCache::default();
    for file_type in [FileType::Host, FileType::Vendored] {
        let path = directory.path().join("clean.php");
        let uri = Uri::from_file_path(&path).unwrap();
        for _ in 0..2 {
            let file = database.add(File::new(
                b"clean.php".as_slice().into(),
                file_type,
                Some(path.clone()),
                b"<?php".as_slice().into(),
            ));
            let dirty = HashSet::from_iter([file]);
            let mut diagnostics = HashMap::default();
            cache.update_document_uris(&database, &dirty, &mut diagnostics);
            assert_eq!(diagnostics, HashMap::from_iter([(uri.clone(), Vec::new())]));
            assert!(cache.uris.is_empty());
            assert_eq!(cache.uris.capacity(), 0);

            assert!(database.delete(file));
            diagnostics.clear();
            cache.update_document_uris(&database, &dirty, &mut diagnostics);
            assert!(diagnostics.is_empty());
        }
    }
}

#[test]
fn uri_bookkeeping_preserves_current_output_and_clears_replaced_or_missing_paths() {
    let directory = tempfile::tempdir().unwrap();
    let old_path = directory.path().join("old.php");
    let new_path = directory.path().join("new.php");
    let old_uri = Uri::from_file_path(&old_path).unwrap();
    let new_uri = Uri::from_file_path(&new_path).unwrap();
    let file = |path| File::new(b"target.php".as_slice().into(), FileType::Host, path, b"<?php".as_slice().into());
    for (deleted, clear_before_move) in [(false, false), (false, true), (true, false), (true, true)] {
        let configuration = DatabaseConfiguration::new(directory.path(), vec![], vec![], vec![], vec![]);
        let mut database = Database::new(configuration);
        let target = database.add(file(Some(old_path.clone())));
        let dirty = HashSet::from_iter([target]);
        let mut cache = DiagnosticCache::default();
        let current = vec![Diagnostic { message: "current".into(), ..Diagnostic::default() }];
        let mut diagnostics = HashMap::from_iter([(old_uri.clone(), current.clone())]);
        cache.update_document_uris(&database, &dirty, &mut diagnostics);
        assert_eq!(diagnostics[&old_uri], current);

        if clear_before_move {
            diagnostics.clear();
            cache.update_document_uris(&database, &dirty, &mut diagnostics);
            assert_eq!(diagnostics, HashMap::from_iter([(old_uri.clone(), Vec::new())]));
            assert!(cache.uris.is_empty());
        }

        database.add(file(Some(new_path.clone())));
        diagnostics = HashMap::from_iter([(new_uri.clone(), current.clone())]);
        cache.update_document_uris(&database, &dirty, &mut diagnostics);
        let mut expected = HashMap::from_iter([(new_uri.clone(), current)]);
        if !clear_before_move {
            expected.insert(old_uri.clone(), Vec::new());
        }
        assert_eq!(diagnostics, expected);
        assert_eq!(cache.uris.get(&target), Some(&new_uri));

        if deleted {
            assert!(database.delete(target));
        } else {
            database.add(file(None));
        }
        diagnostics.clear();
        cache.update_document_uris(&database, &dirty, &mut diagnostics);
        assert_eq!(diagnostics, HashMap::from_iter([(new_uri.clone(), Vec::new())]));
        assert!(cache.uris().next().is_none());
        diagnostics.clear();
        cache.update_document_uris(&database, &dirty, &mut diagnostics);
        assert!(diagnostics.is_empty());
    }
}
