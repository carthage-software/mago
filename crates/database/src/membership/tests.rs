#![allow(clippy::unwrap_used)]

use std::borrow::Cow;
use std::path::Path;

use tempfile::TempDir;

use crate::DatabaseConfiguration;
use crate::DatabaseReader;
use crate::GlobSettings;
use crate::exclusion::Exclusion;
use crate::file::FileType;
use crate::loader::DatabaseLoader;

use super::WorkspaceMatcher;

fn write(root: &Path, name: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "<?php\n").unwrap();
}

fn fixture(files: &[&str]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    for file in files {
        write(dir.path(), file);
    }
    dir
}

fn configuration(dir: &TempDir, paths: &[&str], includes: &[&str], patches: &[&str]) -> DatabaseConfiguration<'static> {
    let patterns = |values: &[&str]| values.iter().map(|value| Cow::Owned(value.as_bytes().to_vec())).collect();
    DatabaseConfiguration {
        workspace: Cow::Owned(dir.path().canonicalize().unwrap()),
        paths: patterns(paths),
        includes: patterns(includes),
        patches: patterns(patches),
        excludes: Vec::new(),
        extensions: vec![Cow::Borrowed(b"php")],
        glob: GlobSettings::default(),
    }
}

fn assert_rows(config: &DatabaseConfiguration<'_>, matcher: &WorkspaceMatcher, rows: &[(&str, Option<FileType>)]) {
    let database = DatabaseLoader::new(config.clone()).load().unwrap();
    for (name, expected) in rows {
        let path = config.workspace.join(name);
        let canonical = path.canonicalize().unwrap();
        let loaded = database
            .files()
            .find(|file| file.path.as_ref().is_some_and(|path| path.canonicalize().unwrap() == canonical))
            .map(|file| file.file_type);
        assert_eq!(loaded, *expected, "loader: {name}");
        assert_eq!(matcher.classify(&path), *expected, "matcher: {name}");
    }
}

#[test]
fn partial_filename_globs_match_loader_and_new_files() {
    let dir =
        fixture(&["src/FooOne.php", "src/FooOne.txt", "src/fooLower.php", "src/Other.php", "src/nested/FooTwo.php"]);
    let mut config = configuration(&dir, &["src/Foo*.php"], &[], &[]);
    config.glob.case_insensitive = true;
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_rows(
        &config,
        &matcher,
        &[
            ("src/FooOne.php", Some(FileType::Host)),
            ("src/FooOne.txt", None),
            ("src/fooLower.php", None),
            ("src/Other.php", None),
            ("src/nested/FooTwo.php", None),
        ],
    );
    let future = config.workspace.join("src/FooCreated.php");
    assert_eq!(matcher.classify(&future), Some(FileType::Host));
    write(dir.path(), "src/FooCreated.php");
    assert_rows(&config, &matcher, &[("src/FooCreated.php", Some(FileType::Host))]);
}

#[test]
fn source_globs_keep_directory_and_recursive_boundaries() {
    let dir =
        fixture(&["src/Top.php", "src/FeatureOne/Item.php", "src/FeatureOne/deep/Item.php", "src/Other/Item.php"]);
    for (pattern, expected) in
        [("src/*.php", [true, false, false, false]), ("src/Feature*/**/*.php", [false, true, true, false])]
    {
        let config = configuration(&dir, &[pattern], &[], &[]);
        let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
        let names = ["src/Top.php", "src/FeatureOne/Item.php", "src/FeatureOne/deep/Item.php", "src/Other/Item.php"];
        let rows = names
            .into_iter()
            .zip(expected)
            .map(|(name, tracked)| (name, tracked.then_some(FileType::Host)))
            .collect::<Vec<_>>();
        assert_rows(&config, &matcher, &rows);
    }
}

#[test]
fn existing_literal_paths_keep_glob_characters_and_exact_extensions() {
    let dir = fixture(&["src[one]/Tracked.php", "src[one]/entry", "entry{dev}", "src/Other.php"]);
    let config = configuration(&dir, &["src[one]", "entry{dev}"], &[], &[]);
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_rows(
        &config,
        &matcher,
        &[
            ("src[one]/Tracked.php", Some(FileType::Host)),
            ("src[one]/entry", None),
            ("entry{dev}", Some(FileType::Host)),
            ("src/Other.php", None),
        ],
    );
}

#[test]
fn source_globs_preserve_specificity_and_exclusions_across_tiers() {
    let dir = fixture(&[
        "src/Main.php",
        "src/Service.php",
        "src/PatchService.php",
        "src/PatchOnly.php",
        "src/ExactService.php",
        "src/SkipService.php",
        "src/SkipPatch.php",
        "excluded/Hidden.php",
    ]);
    let mut config = configuration(
        &dir,
        &["src/*.php", "src/ExactService.php"],
        &["src/*Service.php", "excluded"],
        &["src/Patch*.php", "excluded"],
    );
    config.excludes = vec![
        Exclusion::Pattern(Cow::Borrowed("src/Skip*.php")),
        Exclusion::Path(Cow::Owned(dir.path().join("excluded"))),
    ];
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_rows(
        &config,
        &matcher,
        &[
            ("src/Main.php", Some(FileType::Host)),
            ("src/Service.php", Some(FileType::Vendored)),
            ("src/PatchService.php", Some(FileType::Patch)),
            ("src/PatchOnly.php", Some(FileType::Host)),
            ("src/ExactService.php", Some(FileType::Host)),
            ("src/SkipService.php", None),
            ("src/SkipPatch.php", None),
            ("excluded/Hidden.php", None),
        ],
    );
}

#[cfg(unix)]
#[test]
fn literal_and_glob_symlink_roots_keep_canonical_membership() {
    let dir = fixture(&[]);
    let external = fixture(&["File.php"]);
    std::os::unix::fs::symlink(external.path(), dir.path().join("linked")).unwrap();
    for pattern in ["linked", "linked/*.php"] {
        let config = configuration(&dir, &[pattern], &[], &[]);
        let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
        assert_rows(&config, &matcher, &[("linked/File.php", Some(FileType::Host))]);
        assert_eq!(matcher.classify(&external.path().join("File.php").canonicalize().unwrap()), Some(FileType::Host));
    }
}

#[cfg(unix)]
#[test]
fn missing_files_resolve_existing_parent_aliases_without_widening_source_globs() {
    let dir = fixture(&["src/Existing.php"]);
    let alias = fixture(&[]);
    std::os::unix::fs::symlink(dir.path(), alias.path().join("workspace")).unwrap();
    let config = configuration(&dir, &["src/**/*.php"], &[], &[]);
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_eq!(matcher.classify(&config.workspace.join("src/new/deep/New.php")), Some(FileType::Host));
    assert_eq!(matcher.classify(&alias.path().join("workspace/src/new/deep/New.php")), Some(FileType::Host));
    assert_eq!(matcher.classify(&alias.path().join("workspace/src/new/deep/New.txt")), None);
    assert_eq!(matcher.classify(&alias.path().join("workspace/other/New.php")), None);
}

#[cfg(unix)]
#[test]
fn nested_symlink_packages_keep_the_loaders_source_type_and_exclusions() {
    let dir = fixture(&["src/Main.php"]);
    let package = fixture(&["Service.php", "Hidden.php"]);
    std::fs::create_dir_all(dir.path().join("vendor")).unwrap();
    std::os::unix::fs::symlink(package.path(), dir.path().join("vendor/package")).unwrap();
    let mut config = configuration(&dir, &["src"], &["vendor"], &[]);
    config.excludes = vec![Exclusion::Pattern(Cow::Borrowed("vendor/package/Hidden.php"))];
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_rows(
        &config,
        &matcher,
        &[("vendor/package/Service.php", Some(FileType::Vendored)), ("vendor/package/Hidden.php", None)],
    );

    assert!(matcher.can_contain_files(&config.workspace.join("vendor/package")));
    assert!(!matcher.can_contain_files(&config.workspace.join("cache")));
    assert_eq!(matcher.classify(&config.workspace.join("vendor/../../Outside.php")), None);
}
