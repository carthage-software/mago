#![allow(clippy::unwrap_used)]

use std::borrow::Cow;

use mago_database::DatabaseConfiguration;
use mago_database::exclusion::Exclusion;

use super::*;

#[test]
fn aliases_follow_current_targets_and_release_removed_or_excluded_links() {
    let workspace = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let first = first.path().canonicalize().unwrap();
    let second = second.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("vendor")).unwrap();
    let link = root.join("vendor/package");
    std::os::unix::fs::symlink(&first, &link).unwrap();
    let mut configuration = DatabaseConfiguration::new(&root, vec![b"vendor"], vec![], vec![], vec![]);
    let matcher = WorkspaceMatcher::from_configuration(&configuration).unwrap();
    let mut aliases = DirectoryAliases::default();
    for _ in 0..3 {
        aliases.observe_directory(&link, &matcher);
    }
    assert_eq!(aliases.links.len(), 1);

    std::fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&second, &link).unwrap();
    let old = first.join("Old.php");
    let new = second.join("New.php");
    let mut paths = Cow::Owned(vec![old.clone(), new.clone()]);
    let mut deleted = Cow::Owned(HashSet::from_iter([old]));
    aliases.translate_events(&matcher, &mut paths, &mut deleted);
    assert!(paths.contains(&link.join("New.php")));
    assert!(!paths.contains(&link.join("Old.php")));
    assert!(!deleted.contains(&link.join("Old.php")));
    assert_eq!(aliases.links.len(), 1);

    configuration.excludes.push(Exclusion::Path(Cow::Borrowed(&second)));
    let excluded = WorkspaceMatcher::from_configuration(&configuration).unwrap();
    aliases.translate_events(&excluded, &mut Cow::Owned(vec![new.clone()]), &mut Cow::Owned(HashSet::default()));
    assert!(aliases.links.is_empty());
    aliases.observe_directory(&link, &matcher);
    std::fs::remove_file(&link).unwrap();
    aliases.translate_events(&matcher, &mut Cow::Owned(vec![new]), &mut Cow::Owned(HashSet::default()));
    assert!(aliases.links.is_empty());
}

#[test]
fn dangling_aliases_respect_lexical_and_resolved_exclusions() {
    let workspace = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let target = external.path().canonicalize().unwrap().join("missing");
    std::fs::create_dir_all(root.join("vendor")).unwrap();
    let link = root.join("vendor/package");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let configuration = DatabaseConfiguration::new(&root, vec![b"vendor"], vec![], vec![], vec![]);
    let matcher = WorkspaceMatcher::from_configuration(&configuration).unwrap();
    let aliases = DirectoryAliases::new(vec![link.clone()], &matcher);
    assert_eq!(aliases.targets().collect::<Vec<_>>(), std::slice::from_ref(&target));
    assert_eq!(aliases.targets().next().unwrap().as_os_str(), target.as_os_str());
    for excluded in [&link, &target] {
        let mut configuration = configuration.clone();
        configuration.excludes.push(Exclusion::Path(Cow::Borrowed(excluded)));
        let matcher = WorkspaceMatcher::from_configuration(&configuration).unwrap();
        let aliases = DirectoryAliases::new(vec![link.clone()], &matcher);
        assert!(aliases.targets().next().is_none(), "{}", excluded.display());
    }
}

#[test]
fn observing_known_directory_aliases_keeps_capacity() {
    let workspace = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let configuration = DatabaseConfiguration::new(&root, vec![b"."], vec![], vec![], vec![]);
    let matcher = WorkspaceMatcher::from_configuration(&configuration).unwrap();
    let mut aliases = DirectoryAliases::default();
    while aliases.links.is_empty() || aliases.links.len() < aliases.links.capacity() {
        let link = root.join(format!("package{}", aliases.links.len()));
        std::os::unix::fs::symlink(external.path(), &link).unwrap();
        aliases.observe_directory(&link, &matcher);
    }
    let capacity = aliases.links.capacity();
    let links: Vec<_> = aliases.links.iter().cloned().collect();
    for _ in 0..3 {
        for link in &links {
            aliases.observe_directory(link, &matcher);
        }
        assert_eq!(aliases.links.len(), links.len());
        assert_eq!(aliases.links.capacity(), capacity);
    }
    let paths = vec![root.join("Other.php")];
    let deleted = HashSet::default();
    let mut routed_paths = Cow::Borrowed(paths.as_slice());
    let mut routed_deleted = Cow::Borrowed(&deleted);
    aliases.translate_events(&matcher, &mut routed_paths, &mut routed_deleted);
    assert!(matches!(routed_paths, Cow::Borrowed(_)));
    assert!(matches!(routed_deleted, Cow::Borrowed(_)));
}
