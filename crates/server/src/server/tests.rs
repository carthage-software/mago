#![allow(clippy::unwrap_used)]

use std::path::Path;

use mago_database::DatabaseConfiguration;
use mago_database::file::File;
use mago_database::file::FileType;

use super::*;
use crate::{Features, Range, RefusalReason, RenameRefusal, SymbolLocation};

fn server(source: &'static str) -> (Server, FileId) {
    server_with_features(source, Features { analyzer: true, linter: false, formatter: false })
}

fn server_with_features(source: &'static str, features: Features) -> (Server, FileId) {
    let file = File::ephemeral(b"memory.php".as_slice().into(), source.as_bytes().into());
    assert!(file.path.is_none());
    let id = file.id;
    let mut database = Database::new(DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]));
    database.add(file);
    let php_version = PHPVersion::PHP85;
    let server = Server::new(
        database,
        CodebaseMetadata::new(),
        SymbolReferences::new(),
        Settings {
            php_version,
            features,
            parser: mago_syntax::settings::ParserSettings::default(),
            analyzer: mago_analyzer::settings::Settings { version: php_version, ..Default::default() },
            linter: mago_linter::settings::Settings { php_version, ..Default::default() },
            formatter: FormatSettings::default(),
            plugin_registry: Arc::new(mago_analyzer::plugin::PluginRegistry::new()),
            excludes: crate::Excludes::default(),
        },
    );
    (server, id)
}

fn analyze(server: &mut Server, collect: bool) {
    if collect {
        server.analyze().unwrap();
    } else {
        server.analyze_without_collecting().unwrap();
    }
}

#[test]
fn full_analysis_uses_current_database_contents() {
    let before = "<?php class Before { public function previous(): int { return 1; } } (new Before())->previous();";
    let after = "<?php class After { public function current(): int { return 2; } } (new After())->current();";
    for collect in [false, true] {
        let (mut server, file) = server(before);
        analyze(&mut server, collect);
        assert!(server.codebase().get_method(b"Before", b"previous").is_some());
        assert!(!server.get_definitions(file, before.rfind("previous").unwrap() as u32).is_empty());

        assert!(server.database_mut().update(file, after.as_bytes().into()));
        analyze(&mut server, collect);

        assert!(server.codebase().get_class_like(b"Before").is_none(), "collect={collect}");
        assert!(server.codebase().get_method(b"After", b"current").is_some(), "collect={collect}");
        let declaration = after.find("current").unwrap() as u32;
        assert_eq!(
            server.get_definitions(file, after.rfind("current").unwrap() as u32),
            [SymbolLocation { file, range: Range::new(declaration, declaration + 7) }],
            "collect={collect}",
        );
    }
}

#[test]
fn file_analyses_follow_contents_and_explicit_invalidation() {
    let (mut server, file) = server("<?php class Before {}");
    let before = server.file_analysis_for(file).unwrap();
    assert!(Arc::ptr_eq(&before, &server.file_analysis_for(file).unwrap()));

    assert!(server.database_mut().update(file, b"<?php class After {}".as_slice().into()));
    let after = server.file_analysis_for(file).unwrap();
    assert!(!Arc::ptr_eq(&before, &after));
    assert_eq!(after.document_symbols[0].name, "After");
    assert!(Arc::ptr_eq(&after, &server.file_analysis_for(file).unwrap()));

    server.invalidate_artifacts(&[file]);
    assert_eq!(server.analyses().count(), 0);
    assert!(!Arc::ptr_eq(&after, &server.file_analysis_for(file).unwrap()));

    assert!(server.database_mut().delete(file));
    server.refresh_analyses(&[file]);
    assert_eq!(server.analyses().count(), 0);
    assert!(server.file_analysis_for(file).is_none());
}

#[test]
fn changed_files_build_editor_data_only_when_diagnostics_need_it() {
    for analyzer in [false, true] {
        for linter in [false, true] {
            let (mut server, file) =
                server_with_features("<?php class Example {}", Features { analyzer, linter, formatter: false });
            server.analyze_without_collecting().unwrap();
            server.refresh_changed_files(&[file]).unwrap();
            assert_eq!(server.analyses().count(), usize::from(linter || !analyzer));

            let analysis = server.file_analysis_for(file).unwrap();
            assert!(Arc::ptr_eq(&analysis, &server.file_analysis_for(file).unwrap()));
        }
    }
}

#[test]
fn host_refreshes_leave_dependency_queries_lazy() {
    let (mut server, host) = server("<?php class HostClass {}");
    let dependency = File::new(
        b"dependency.php".as_slice().into(),
        mago_database::file::FileType::Vendored,
        None,
        b"<?php class DependencyClass {}".as_slice().into(),
    );
    let dependency_id = dependency.id;
    server.database_mut().add(dependency);

    server.refresh_analyses(&[host, dependency_id]);
    assert_eq!(server.lint_issue_sources().map(|(file, _)| file).collect::<Vec<_>>(), [host]);
    server.refresh_all_host_analyses();
    assert_eq!(server.lint_issue_sources().map(|(file, _)| file).collect::<Vec<_>>(), [host]);

    let analysis = server.file_analysis_for(dependency_id).unwrap();
    assert_eq!(analysis.document_symbols[0].name, "DependencyClass");
    assert_eq!(server.analyses().count(), 2);
}

#[test]
fn dependency_queries_do_not_create_lint_or_semantic_issues() {
    use mago_database::file::FileType;

    let source = "<?php function run(int $value, int $value): void {}";
    for analyzer in [false, true] {
        let (mut server, host) = server_with_features(source, Features { analyzer, linter: true, formatter: false });
        assert!(!server.file_analysis_for(host).unwrap().lint_issues.is_empty());
        for file_type in [FileType::Vendored, FileType::Builtin, FileType::Patch, FileType::External] {
            let file =
                File::new(format!("{file_type:?}.php").into_bytes().into(), file_type, None, source.as_bytes().into());
            let id = server.database_mut().add(file);
            let analysis = server.file_analysis_for(id).unwrap();
            assert_eq!(analysis.document_symbols[0].name, "run");
            assert!(analysis.lint_issues.is_empty(), "{file_type:?}, analyzer={analyzer}");
        }
    }
}

#[test]
fn formatting_respects_disabled_features_and_read_only_sources() {
    const SOURCE: &str = "<?php\nfunction run(){return 1+2;}\n";
    for formatter in [false, true] {
        let (mut server, host) = server_with_features(SOURCE, Features { analyzer: false, linter: false, formatter });
        let readonly = read_only_sources(&mut server, SOURCE);
        let range = Range::new(0, SOURCE.len() as u32);
        assert_eq!(server.get_formatting(host).is_some(), formatter);
        assert_eq!(server.get_range_formatting(host, range).is_some(), formatter);
        for file in readonly.into_iter().chain([FileId::new(b"unknown.php")]) {
            assert!(server.get_formatting(file).is_none());
            assert!(server.get_range_formatting(file, range).is_none());
        }
    }
}

#[test]
fn rename_and_code_actions_accept_pathless_host_buffers_but_reject_read_only_sources() {
    const SOURCE: &str = "<?php function run() { $value = 1; return $value + 2; }";
    let (mut server, host) = server(SOURCE);
    let readonly = read_only_sources(&mut server, SOURCE);
    let offset = SOURCE.find("$value").unwrap() as u32;
    let start = SOURCE.find("$value + 2").unwrap() as u32;
    let end = start + "$value + 2".len() as u32;
    assert_eq!(server.get_prepare_rename(host, offset), Ok(Range::new(offset + 1, offset + 6)));
    let edits = server.get_rename(host, offset, "renamed").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[&host].len(), 2);
    assert!(!server.get_code_actions(host, start, end, Some(&["refactor.extract"])).is_empty());
    let unknown = FileId::new(b"unknown.php");
    for file in readonly.into_iter().chain([unknown]) {
        let expected = if file == unknown {
            RenameRefusal::new(RefusalReason::NoSymbol)
        } else {
            RenameRefusal::at(RefusalReason::ReadOnly, file, Range::new(offset, offset))
        };
        assert_eq!(server.get_prepare_rename(file, offset).unwrap_err(), expected);
        assert_eq!(server.get_rename(file, offset, "renamed").unwrap_err(), expected);
        assert_eq!(server.get_member_rename(file, offset, Some(b"renamed")).unwrap_err(), expected);
        assert!(server.get_code_actions(file, start, end, None).is_empty());
    }
    assert_eq!(server.analyses().count(), 1);
}

#[test]
fn edit_batches_reject_unknown_or_read_only_targets_as_a_whole() {
    let (mut server, host) = server("<?php");
    let second =
        server.database_mut().add(File::ephemeral(b"second.php".as_slice().into(), b"<?php".as_slice().into()));
    let readonly = read_only_sources(&mut server, "<?php");
    assert!(server.can_edit_files([host, second]));
    for file in readonly {
        assert!(!server.can_edit_files([host, file]));
    }
    assert!(!server.can_edit_files([host, FileId::new(b"unknown.php")]));
    server.database_mut().delete(second);
    assert!(!server.can_edit_files([host, second]));
}

fn read_only_sources(server: &mut Server, source: &'static str) -> [FileId; 2] {
    [FileType::Builtin, FileType::External].map(|file_type| {
        server.database_mut().add(File::new(
            format!("{file_type:?}.php").into_bytes().into(),
            file_type,
            None,
            source.as_bytes().into(),
        ))
    })
}
