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

#[cfg(unix)]
#[test]
fn git_ignored_symlinks_keep_the_loaders_source_type() {
    let dir = fixture(&["src/Main.php"]);
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join(".gitignore"), "/vendor/\n").unwrap();
    let package = fixture(&["Service.php"]);
    std::fs::create_dir_all(dir.path().join("vendor")).unwrap();
    std::os::unix::fs::symlink(package.path(), dir.path().join("vendor/package")).unwrap();
    let root = dir.path().canonicalize().unwrap().to_string_lossy().into_owned();
    let config = configuration(&dir, &[&root], &[], &[]);
    let ignores = std::sync::Arc::new(crate::git::GitIgnores::discover_with_global(dir.path(), None).unwrap());
    let matcher =
        WorkspaceMatcher::from_configuration(&config).unwrap().with_git_ignores(std::sync::Arc::clone(&ignores));
    let database = DatabaseLoader::new(config.clone()).with_git_ignores(ignores).load().unwrap();
    for (name, expected) in [("src/Main.php", FileType::Host), ("vendor/package/Service.php", FileType::Vendored)] {
        let path = config.workspace.join(name);
        let loaded =
            database.files().find(|file| file.path.as_deref() == Some(path.as_path())).map(|file| file.file_type);
        assert_eq!(loaded, Some(expected), "loader: {name}");
        assert_eq!(matcher.classify(&path), Some(expected), "matcher: {name}");
    }
    assert_eq!(matcher.classify(&config.workspace.join("vendor/package/New.php")), Some(FileType::Vendored));
}

#[test]
fn directory_globs_exclude_nested_files_like_the_loader() {
    let dir = fixture(&["generated/Top.php", "generated/deep/Nested.php", "generatedx/Kept.php", "src/Main.php"]);
    let root = dir.path().canonicalize().unwrap().to_string_lossy().into_owned();
    let mut config = configuration(&dir, &[&root], &[], &[]);
    config.glob.literal_separator = true;
    config.excludes = vec![Exclusion::Pattern(Cow::Borrowed("generated/*"))];
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_rows(
        &config,
        &matcher,
        &[
            ("generated/Top.php", None),
            ("generated/deep/Nested.php", None),
            ("generatedx/Kept.php", Some(FileType::Host)),
            ("src/Main.php", Some(FileType::Host)),
        ],
    );
    assert_eq!(matcher.classify(&config.workspace.join("generated/new/Created.php")), None);
}

#[test]
fn directory_globs_prune_only_below_each_walked_source_root() {
    let dir = fixture(&["generated/Top.php", "generated/deep/Old.php"]);
    let shared = fixture(&["cache/Top.php", "cache/deep/Old.php", "Kept.php"]);
    let shared_root = shared.path().canonicalize().unwrap().to_string_lossy().into_owned();
    let mut config = configuration(&dir, &["generated", &shared_root], &[], &[]);
    config.glob.literal_separator = true;
    config.excludes = vec![
        Exclusion::Pattern(Cow::Borrowed("generated/*")),
        Exclusion::Pattern(Cow::Owned(format!("{shared_root}/cache/*"))),
    ];
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_rows(&config, &matcher, &[("generated/Top.php", None), ("generated/deep/Old.php", Some(FileType::Host))]);
    assert_eq!(matcher.classify(&config.workspace.join("generated/deep/New.php")), Some(FileType::Host));
    assert!(matcher.can_contain_files(&config.workspace.join("generated/deep")));
    assert!(matcher.can_contain_files(&config.workspace.join("generated/new")));

    let database = DatabaseLoader::new(config.clone()).load().unwrap();
    for (name, expected) in [("cache/Top.php", None), ("cache/deep/Old.php", None), ("Kept.php", Some(FileType::Host))]
    {
        let path = shared.path().canonicalize().unwrap().join(name);
        let loaded =
            database.files().find(|file| file.path.as_deref() == Some(path.as_path())).map(|file| file.file_type);
        assert_eq!(loaded, expected, "loader: {name}");
        assert_eq!(matcher.classify(&path), expected, "matcher: {name}");
    }
    assert_eq!(matcher.classify(&shared.path().canonicalize().unwrap().join("cache/deep/New.php")), None);
    assert!(!matcher.can_contain_files(&shared.path().canonicalize().unwrap().join("cache/deep")));
    assert!(matcher.can_contain_files(&shared.path().canonicalize().unwrap().join("other")));
}

#[test]
fn pruned_directories_cannot_contain_files_and_file_globs_never_prune_them() {
    let dir = fixture(&["generated/deep/Old.php", "vendor/package/Library.php"]);
    let mut config = configuration(&dir, &["."], &["vendor"], &[]);
    config.glob.literal_separator = true;
    config.excludes =
        vec![Exclusion::Pattern(Cow::Borrowed("generated/*")), Exclusion::Pattern(Cow::Borrowed("**/vendor"))];
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    assert_rows(
        &config,
        &matcher,
        &[("generated/deep/Old.php", None), ("vendor/package/Library.php", Some(FileType::Vendored))],
    );
    assert!(!matcher.can_contain_files(&config.workspace.join("generated")));
    assert!(!matcher.can_contain_files(&config.workspace.join("generated/deep")));
    assert!(matcher.can_contain_files(&config.workspace.join("vendor")));
    assert!(matcher.watch_roots().any(|root| root == config.workspace.join("vendor")));
}

#[cfg(unix)]
#[test]
fn pruned_symlink_aliases_keep_the_path_the_loader_reads() {
    let dir = fixture(&["src/deep/Old.php"]);
    std::os::unix::fs::symlink(dir.path().join("src"), dir.path().join("generated")).unwrap();
    let root = dir.path().canonicalize().unwrap().to_string_lossy().into_owned();
    let mut config = configuration(&dir, &[&root], &[], &[]);
    config.glob.literal_separator = true;
    config.excludes = vec![Exclusion::Pattern(Cow::Borrowed("generated/*"))];
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
    let database = DatabaseLoader::new(config.clone()).load().unwrap();
    let paths: Vec<_> = database.files().filter_map(|file| file.path.clone()).collect();
    assert_eq!(paths, [config.workspace.join("src/deep/Old.php")]);

    let requested = config.workspace.join("generated/deep/New.php");
    let resolved = config.workspace.join("src/deep/New.php");
    assert_eq!(matcher.classify(&requested), Some(FileType::Host));
    assert_eq!(matcher.source_path(&requested, &resolved), resolved.as_path());
    let kept = config.workspace.join("src/deep/New.php");
    assert_eq!(matcher.source_path(&kept, &kept), kept.as_path());
}

#[test]
fn command_exclusions_follow_the_sources_they_are_added_to() {
    let dir = fixture(&[
        "generated/Top.php",
        "generated/deep/Old.php",
        "vendor/generated/Top.php",
        "vendor/generated/deep/Service.php",
        "vendor/other/Kept.php",
        "patches/generated/deep/Patch.php",
    ]);

    let excludes = [
        Exclusion::Pattern(Cow::Borrowed("generated/*")),
        Exclusion::Pattern(Cow::Borrowed("vendor/generated/*")),
        Exclusion::Pattern(Cow::Borrowed("patches/generated/*")),
    ];

    let glob = GlobSettings { literal_separator: true, ..GlobSettings::default() };
    for (paths, nested_excluded) in [(vec!["generated"], false), (vec!["."], true)] {
        let config = configuration(&dir, &paths, &["vendor"], &["patches"]);
        let matcher = WorkspaceMatcher::from_configuration(&config).unwrap();
        let exclusions = matcher.exclusions(&excludes, glob).unwrap();
        let mut command = config.clone();
        command.glob = glob;
        command.excludes = excludes.to_vec();
        let database = DatabaseLoader::new(command).load().unwrap();
        for (name, file_type, excluded) in [
            ("generated/Top.php", FileType::Host, true),
            ("generated/deep/Old.php", FileType::Host, nested_excluded),
            ("vendor/generated/Top.php", FileType::Vendored, true),
            ("vendor/generated/deep/Service.php", FileType::Vendored, true),
            ("vendor/other/Kept.php", FileType::Vendored, false),
            ("patches/generated/deep/Patch.php", FileType::Patch, true),
        ] {
            let path = config.workspace.join(name);
            assert_eq!(exclusions.is_excluded(&path, file_type), excluded, "exclusions: {name} {paths:?}");
            let loaded = database.files().find(|file| file.path.as_deref() == Some(path.as_path()));
            assert_eq!(loaded.map(|file| file.file_type), (!excluded).then_some(file_type), "loader: {name} {paths:?}");
        }
    }
}

#[cfg(unix)]
#[test]
fn git_ignores_use_the_path_the_registry_keeps() {
    let repository = fixture(&["app/src/Main.php"]);
    std::fs::create_dir_all(repository.path().join(".git")).unwrap();
    std::fs::write(repository.path().join(".gitignore"), "/app/cache/\n").unwrap();
    std::fs::create_dir_all(repository.path().join("app/cache")).unwrap();
    std::os::unix::fs::symlink(repository.path().join("app/cache"), repository.path().join("alias")).unwrap();
    let workspace = repository.path().join("app").canonicalize().unwrap();
    let config = DatabaseConfiguration {
        workspace: Cow::Owned(workspace.clone()),
        paths: vec![Cow::Owned(workspace.to_string_lossy().into_owned().into_bytes())],
        includes: Vec::new(),
        patches: Vec::new(),
        excludes: Vec::new(),
        extensions: vec![Cow::Borrowed(b"php")],
        glob: GlobSettings::default(),
    };
    let ignores = std::sync::Arc::new(crate::git::GitIgnores::discover_with_global(&workspace, None).unwrap());
    let matcher = WorkspaceMatcher::from_configuration(&config).unwrap().with_git_ignores(ignores);
    let alias = repository.path().canonicalize().unwrap().join("alias/New.php");
    let resolved = workspace.join("cache/New.php");

    assert_eq!(matcher.source_path(&alias, &resolved), resolved.as_path());
    assert_eq!(matcher.classify(&alias), Some(FileType::Vendored));
    assert_eq!(matcher.classify(&workspace.join("src/Main.php")), Some(FileType::Host));
}
