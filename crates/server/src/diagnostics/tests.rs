#![allow(clippy::unwrap_used)]

use crate::LinterContext;
use mago_database::DatabaseConfiguration;
use mago_database::GlobSettings;
use mago_database::file::File;
use mago_database::file::FileType;
use mago_linter::settings::Settings as LinterSettings;
use mago_reporting::Annotation;
use mago_reporting::IgnoreEntry;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::settings::ParserSettings;

use super::*;

fn messages(
    update: &DiagnosticUpdate,
    database: &Database<'_>,
    ignores: &CompiledIgnoreSet,
) -> HashMap<FileId, Vec<String>> {
    let mut result: HashMap<FileId, Vec<String>> = HashMap::default();
    for issue in update.issues(database, ignores) {
        result.entry(primary_annotation(issue).unwrap().span.file_id).or_default().push(issue.message.to_string());
    }
    result
}

fn issues(target: FileId, related: FileId) -> Arc<IssueCollection> {
    Arc::new(IssueCollection::from([Issue::error("problem")
        .with_annotation(Annotation::primary(Span::new(target, Position::new(0), Position::new(1))))
        .with_annotation(Annotation::secondary(Span::new(related, Position::new(0), Position::new(1))))]))
}

#[test]
fn unchanged_sources_skip_rendering_but_related_file_edits_refresh_ranges() {
    let target = FileId::new(b"target.php");
    let related = FileId::new(b"related.php");
    let other = FileId::new(b"other.php");
    let key = SourceKey::Analyzer(Some(other), false);
    let issues = issues(target, related);
    let input = || std::iter::once((key, Source::Analyzer(Arc::clone(&issues))));
    let mut cache = DiagnosticCache::default();
    assert_eq!(cache.update(input(), &[]), HashSet::from_iter([target]));
    assert!(cache.update(input(), &[]).is_empty());
    assert_eq!(cache.update(input(), &[other]), HashSet::from_iter([other]));
    assert_eq!(cache.update(input(), &[related]), HashSet::from_iter([target, related]));
    assert_eq!(cache.update(std::iter::empty(), &[]), HashSet::from_iter([target]));
    assert!(cache.sources.is_empty());
}

#[test]
fn equal_replacements_release_old_owners_and_changed_targets_clear_old_output() {
    let a = FileId::new(b"a.php");
    let b = FileId::new(b"b.php");
    let key = SourceKey::Analyzer(None, true);
    let first = issues(a, b);
    let weak = Arc::downgrade(&first);
    let mut cache = DiagnosticCache::default();
    cache.update(std::iter::once((key, Source::Analyzer(Arc::clone(&first)))), &[]);
    assert!(weak.upgrade().is_some());
    let second = issues(a, b);
    assert!(cache.update(std::iter::once((key, Source::Analyzer(Arc::clone(&second)))), &[]).is_empty());
    assert_eq!(Arc::strong_count(&second), 1);
    assert!(
        matches!(cache.sources[&key].source.upgrade(), Some(Source::Analyzer(owner)) if Arc::ptr_eq(&owner, &second))
    );
    drop(first);
    assert!(weak.upgrade().is_none());
    assert_eq!(cache.update(std::iter::once((key, Source::Analyzer(issues(b, b)))), &[]), HashSet::from_iter([a, b]),);
    assert_eq!(cache.update(std::iter::once((key, Source::Analyzer(Arc::default()))), &[]), HashSet::from_iter([b]),);
}

#[test]
fn expired_sources_release_analysis_and_preserve_diagnostic_clears() {
    let configuration = DatabaseConfiguration::new(std::path::Path::new("/workspace"), vec![], vec![], vec![], vec![]);
    let mut database = Database::new(configuration);
    let file =
        database.add(File::new(b"target.php".as_slice().into(), FileType::Host, None, b"<?php".as_slice().into()));
    let ignores = CompiledIgnoreSet::compile(&[], GlobSettings::default());
    let linter = LinterContext::new(LinterSettings::default(), ParserSettings::default(), false);
    let mut analysis = crate::file_analysis::build(database.get_ref(&file).unwrap(), &linter, false);
    analysis.lint_issues = (*issues(file, file)).clone();

    for (key, source) in [
        (SourceKey::Analyzer(Some(file), false), Source::Analyzer(issues(file, file))),
        (SourceKey::Linter(file), Source::Linter(Arc::new(analysis))),
    ] {
        let weak = source.downgrade();
        let mut cache = DiagnosticCache::default();
        let dirty = cache.update(std::iter::once((key, weak.upgrade().unwrap())), &[]);
        let update = cache.select(dirty);
        assert!(!messages(&update, &database, &ignores)[&file].is_empty());
        drop(update);

        drop(source);
        assert!(weak.upgrade().is_none());
        let dirty = cache.update(std::iter::empty(), &[]);
        assert_eq!(dirty, HashSet::from_iter([file]));
        let update = cache.select(dirty);
        assert_eq!(update.files, HashSet::from_iter([file]));
        assert!(update.issues(&database, &ignores).next().is_none());
        assert!(cache.sources.is_empty());
    }
}

#[test]
fn equal_replacement_of_an_expired_source_refreshes_diagnostics() {
    let file = FileId::new(b"target.php");
    let key = SourceKey::Analyzer(Some(file), false);
    let mut cache = DiagnosticCache::default();
    cache.update(std::iter::once((key, Source::Analyzer(issues(file, file)))), &[]);
    assert!(cache.sources[&key].source.upgrade().is_none());

    let next = issues(file, file);
    let dirty = cache.update(std::iter::once((key, Source::Analyzer(Arc::clone(&next)))), &[]);
    assert_eq!(dirty, HashSet::from_iter([file]));
    assert_eq!(Arc::strong_count(&next), 1);
}

#[test]
fn rendering_keeps_source_order_and_applies_analyzer_ignores_only_to_affected_targets() {
    let configuration = DatabaseConfiguration::new(std::path::Path::new("/workspace"), vec![], vec![], vec![], vec![]);
    let mut database = Database::new(configuration);
    let target =
        database.add(File::new(b"target.php".as_slice().into(), FileType::Host, None, b"<?php".as_slice().into()));
    let other =
        database.add(File::new(b"other.php".as_slice().into(), FileType::Host, None, b"<?php".as_slice().into()));
    let issue = |message, code, file| {
        Issue::error(message).with_code(code).with_annotation(Annotation::primary(Span::new(
            file,
            Position::new(0),
            Position::new(1),
        )))
    };
    let first = Arc::new(IssueCollection::from([
        issue("first", "visible", target),
        issue("ignored", "hidden", target),
        issue("unaffected", "visible", other),
    ]));
    let second = Arc::new(IssueCollection::from([issue("second", "visible", target)]));
    let linter = LinterContext::new(LinterSettings::default(), ParserSettings::default(), false);
    let mut analysis = crate::file_analysis::build(database.get_ref(&target).unwrap(), &linter, false);
    analysis.lint_issues = IssueCollection::from([issue("linter", "hidden", target)]);
    let analysis = Arc::new(analysis);
    let ignores = CompiledIgnoreSet::compile(
        &[IgnoreEntry::Scoped { code: "hidden".into(), paths: vec!["target.php".into()] }],
        GlobSettings::default(),
    );
    for reverse in [false, true] {
        let mut sources = vec![
            (SourceKey::Analyzer(None, false), Source::Analyzer(Arc::clone(&first))),
            (SourceKey::Analyzer(Some(other), false), Source::Analyzer(Arc::clone(&second))),
            (SourceKey::Linter(target), Source::Linter(Arc::clone(&analysis))),
        ];
        if reverse {
            sources.reverse();
        }
        let mut cache = DiagnosticCache::default();
        cache.update(sources.into_iter(), &[]);
        let diagnostics = messages(&cache.select(HashSet::from_iter([target])), &database, &ignores);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[&target], ["first", "second", "linter"]);
    }
}

#[test]
fn parser_errors_hide_analyzer_cascades_in_their_file_only() {
    let configuration = DatabaseConfiguration::new(std::path::Path::new("/workspace"), vec![], vec![], vec![], vec![]);
    let mut database = Database::new(configuration);
    let [broken, other] = ["broken.php", "other.php"].map(|name| {
        database.add(File::new(name.as_bytes().to_vec().into(), FileType::Host, None, b"<?php".as_slice().into()))
    });
    let issue = |code, file| {
        Issue::error(code).with_code(code).with_annotation(Annotation::primary(Span::new(
            file,
            Position::new(0),
            Position::new(1),
        )))
    };
    let codebase = Arc::new(IssueCollection::from([issue("codebase", broken), issue("other", other)]));
    let linter = LinterContext::new(LinterSettings::default(), ParserSettings::default(), false);
    for (code, ignored, lint_parse, expected) in [
        ("parse", false, false, vec!["lint", "parse"]),
        ("syntax", false, false, vec!["lint", "syntax"]),
        ("semantics", false, false, vec!["cascade", "codebase", "lint", "semantics"]),
        ("parse", true, false, vec!["cascade", "codebase", "lint"]),
        ("parse", true, true, vec!["lint", "parse"]),
    ] {
        let ignore_entries = if ignored {
            vec![IgnoreEntry::Scoped { code: code.into(), paths: vec!["broken.php".into()] }]
        } else {
            vec![]
        };
        let ignores = CompiledIgnoreSet::compile(&ignore_entries, GlobSettings::default());
        let mut analysis = crate::file_analysis::build(database.get_ref(&broken).unwrap(), &linter, false);
        analysis.lint_issues = IssueCollection::from([issue("lint", broken)]);
        if lint_parse {
            analysis.lint_issues.push(issue("parse", broken));
        }
        let analysis = Arc::new(analysis);
        let analyzer = Arc::new(IssueCollection::from([issue(code, broken), issue("cascade", broken)]));
        let mut cache = DiagnosticCache::default();
        let dirty = cache.update(
            [
                (SourceKey::Analyzer(Some(broken), false), Source::Analyzer(Arc::clone(&analyzer))),
                (SourceKey::Analyzer(None, true), Source::Analyzer(Arc::clone(&codebase))),
                (SourceKey::Linter(broken), Source::Linter(Arc::clone(&analysis))),
            ]
            .into_iter(),
            &[],
        );
        let diagnostics = messages(&cache.select(dirty), &database, &ignores);
        let mut messages: Vec<_> = diagnostics[&broken].iter().map(String::as_str).collect();
        messages.sort_unstable();
        assert_eq!(messages, expected, "{code}, ignored={ignored}, lint_parse={lint_parse}");
        assert_eq!(diagnostics[&other][0], "other");
    }
}

#[test]
fn removing_sources_releases_sparse_tables_and_clears_all_old_targets() {
    let owners: Vec<_> = (0..256)
        .map(|i| {
            let file = FileId::new(format!("file{i}.php").as_bytes());
            (file, issues(file, file))
        })
        .collect();
    let source = |(file, issues): &(FileId, Arc<IssueCollection>)| {
        (SourceKey::Analyzer(Some(*file), false), Source::Analyzer(Arc::clone(issues)))
    };
    let mut cache = DiagnosticCache::default();
    cache.update(owners.iter().map(source), &[]);
    assert!(cache.sources.capacity() >= owners.len());
    let cleared = cache.update(owners[..1].iter().map(source), &[]);
    assert_eq!(cleared, owners[1..].iter().map(|(file, _)| *file).collect());
    assert_eq!(cache.sources.len(), 1);
    assert!(cache.sources.capacity() < 64);
}
