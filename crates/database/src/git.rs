//! Git's ignore rules for a work tree, read without running `git`.
//!
//! [`GitIgnores`] collects the `.gitignore` files Git would read, plus each
//! repository's `info/exclude` and `core.excludesFile`, and answers whether
//! Git ignores a path. Files below an ignored directory stay ignored, as in Git.
//! Matching follows the patterns only: tracked files and `core.ignoreCase` are
//! not consulted.

use std::env::home_dir;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use foldhash::HashMap;
use ignore::Match;
use ignore::gitignore::Gitignore;
use ignore::gitignore::GitignoreBuilder;
use ignore::gitignore::gitconfig_excludes_path;
use walkdir::WalkDir;

/// The ignore rules of the Git work tree that contains a workspace.
#[derive(Debug)]
pub struct GitIgnores {
    root: PathBuf,
    workspace: PathBuf,
    /// Every directory the rules were read from: the work tree root, the
    /// ancestors of the workspace, and the workspace directories Git does not ignore.
    directories: HashMap<PathBuf, Directory>,
    rules: Vec<Gitignore>,
    /// Each repository's `info/exclude`, then its excludes file, in descending precedence.
    repositories: Vec<Vec<Gitignore>>,
    /// The exclude files each repository reads, whether or not they exist.
    excludes: Vec<PathBuf>,
    configs: Vec<RepositoryConfig>,
}

/// A repository configuration and the excludes file it named.
#[derive(Debug)]
struct RepositoryConfig {
    root: PathBuf,
    path: PathBuf,
    excludes: Option<PathBuf>,
}

/// The rules that apply to a directory's entries.
#[derive(Debug, Clone)]
struct Directory {
    /// `.gitignore` files from this directory up to its repository root, deepest first.
    chain: Arc<[u32]>,
    repository: u32,
}

impl GitIgnores {
    /// Read the rules for the work tree that contains `workspace`.
    ///
    /// Returns `None` outside a work tree, or when Git ignores `workspace` itself.
    #[must_use]
    pub fn discover(workspace: &Path) -> Option<Self> {
        Self::discover_with_global(workspace, gitconfig_excludes_path().as_deref())
    }

    pub(crate) fn discover_with_global(workspace: &Path, global: Option<&Path>) -> Option<Self> {
        let workspace = workspace.canonicalize().ok()?;
        let root = workspace.ancestors().find(|directory| directory.join(".git").exists())?.to_path_buf();
        let mut ignores = Self {
            root,
            workspace: workspace.clone(),
            directories: HashMap::default(),
            rules: Vec::new(),
            repositories: Vec::new(),
            excludes: Vec::new(),
            configs: Vec::new(),
        };

        let chain: Vec<_> =
            workspace.ancestors().take_while(|directory| directory.starts_with(&ignores.root)).collect();
        let mut parent = None;
        for directory in chain.into_iter().rev() {
            if let Some(parent) = &parent
                && ignores.decide(directory, true, parent)
            {
                return None;
            }

            parent = Some(ignores.visit(directory, parent.as_ref(), global));
        }

        let walker = WalkDir::new(&workspace).into_iter().filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }

            if !entry.file_type().is_dir() || entry.file_name() == ".git" {
                return false;
            }

            let Some(parent) = entry.path().parent().and_then(|parent| ignores.directories.get(parent)).cloned() else {
                return false;
            };

            if ignores.decide(entry.path(), true, &parent) {
                return false;
            }

            ignores.visit(entry.path(), Some(&parent), global);
            true
        });

        for _ in walker {}

        Some(ignores)
    }

    /// Whether Git ignores `path`, an absolute path below the work tree.
    #[must_use]
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let Ok(relative) = path.strip_prefix(&self.root) else { return false };
        if relative.as_os_str().is_empty() {
            return false;
        }

        if relative.components().any(|component| component.as_os_str() == ".git") {
            return true;
        }

        let Some((read, directory)) =
            path.ancestors().skip(1).find_map(|directory| Some((directory, self.directories.get(directory)?)))
        else {
            return false;
        };

        path.ancestors()
            .take_while(|level| *level != read)
            .enumerate()
            .any(|(depth, level)| self.decide(level, depth > 0 || is_dir, directory))
    }

    /// Whether a change at `path` can change these rules: a `.gitignore` Git reads for the
    /// workspace, an exclude file, or a repository configuration that now names another one.
    #[must_use]
    pub fn depends_on(&self, path: &Path) -> bool {
        if self.excludes.iter().any(|file| file == path) {
            return true;
        }

        if let Some(config) = self.configs.iter().find(|config| config.path == path) {
            return configured_excludes_file(&config.root, &config.path) != config.excludes;
        }

        path.file_name().is_some_and(|name| name == ".gitignore")
            && path.parent().is_some_and(|directory| {
                self.directories.contains_key(directory)
                    || (directory.starts_with(&self.workspace) && !self.is_ignored(directory, true))
            })
    }

    /// The files the rules depend on outside the workspace directory.
    pub fn sources_outside(&self) -> impl Iterator<Item = PathBuf> + '_ {
        let ancestors = self
            .workspace
            .ancestors()
            .skip(1)
            .take_while(|directory| directory.starts_with(&self.root))
            .map(|directory| directory.join(".gitignore"));
        let files = self.excludes.iter().chain(self.configs.iter().map(|config| &config.path)).cloned();
        ancestors.chain(files.filter(|file| !file.starts_with(&self.workspace)))
    }

    /// Match `path` against the rules that apply inside `directory`, an already read
    /// ancestor: the deepest `.gitignore` first, then the repository's excludes.
    fn decide(&self, path: &Path, is_dir: bool, directory: &Directory) -> bool {
        for &rules in directory.chain.iter() {
            match self.rules[rules as usize].matched(path, is_dir) {
                Match::None => {}
                decided => return decided.is_ignore(),
            }
        }

        self.repositories[directory.repository as usize]
            .iter()
            .map(|rules| rules.matched(path, is_dir))
            .find(|decided| !decided.is_none())
            .is_some_and(|decided| decided.is_ignore())
    }

    /// Record `directory` below `parent`, or as the work tree root without one.
    fn visit(&mut self, directory: &Path, parent: Option<&Directory>, global: Option<&Path>) -> Directory {
        let nested = parent.filter(|_| !directory.join(".git").exists());
        let repository = match nested {
            Some(parent) => parent.repository,
            None => {
                let excludes = self.repository_excludes(directory, global);
                self.repositories.push(excludes);
                (self.repositories.len() - 1) as u32
            }
        };
        let inherited = nested.map(|parent| Arc::clone(&parent.chain));
        let chain = match read(directory, &directory.join(".gitignore")) {
            Some(rules) => {
                self.rules.push(rules);
                let own = (self.rules.len() - 1) as u32;
                std::iter::once(own).chain(inherited.iter().flat_map(|chain| chain.iter().copied())).collect()
            }
            None => inherited.unwrap_or_else(|| Arc::from([])),
        };

        let entry = Directory { chain, repository };
        self.directories.insert(directory.to_path_buf(), entry.clone());
        entry
    }

    /// A repository's own `core.excludesFile` takes precedence over the global one.
    fn repository_excludes(&mut self, root: &Path, global: Option<&Path>) -> Vec<Gitignore> {
        let common = git_directory(root).map(|git| resolved(&common_directory(&git)));
        let config = common.as_ref().map(|common| common.join("config"));
        let configured = config.as_deref().and_then(|config| configured_excludes_file(root, config));
        let info = common.map(|common| common.join("info").join("exclude"));
        let file = configured.as_deref().or(global).filter(|file| !file.as_os_str().is_empty()).map(Path::to_path_buf);
        let mut excludes = Vec::new();
        excludes.extend(info.as_deref().and_then(|info| read(root, info)));
        excludes.extend(file.as_deref().and_then(|file| read(root, file)));
        for source in info.into_iter().chain(file) {
            let target = resolved(&source);
            if target != source {
                self.excludes.push(target);
            }
            self.excludes.push(source);
        }
        if let Some(path) = config {
            self.configs.push(RepositoryConfig { root: root.to_path_buf(), path, excludes: configured });
        }

        excludes
    }
}

/// A file's target, resolved through its directory when the file does not exist yet.
fn resolved(path: &Path) -> PathBuf {
    if let Ok(path) = path.canonicalize() {
        return path;
    }

    match (path.parent().and_then(|parent| parent.canonicalize().ok()), path.file_name()) {
        (Some(parent), Some(name)) => parent.join(name),
        _ => path.to_path_buf(),
    }
}

fn read(root: &Path, file: &Path) -> Option<Gitignore> {
    if !file.is_file() {
        return None;
    }

    let mut builder = GitignoreBuilder::new(root);
    builder.add(file);
    builder.build().ok().filter(|rules| !rules.is_empty())
}

/// The repository directory: `.git` itself, or the target of a worktree's `.git` file.
fn git_directory(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }

    let contents = fs::read_to_string(&dot_git).ok()?;
    let target = Path::new(contents.strip_prefix("gitdir:")?.trim());
    Some(if target.is_absolute() { target.to_path_buf() } else { root.join(target) })
}

/// Linked worktrees keep `info/exclude` and the repository configuration in the main repository.
fn common_directory(git: &Path) -> PathBuf {
    match fs::read_to_string(git.join("commondir")) {
        Ok(common) => {
            let common = Path::new(common.trim());
            if common.is_absolute() { common.to_path_buf() } else { git.join(common) }
        }
        Err(_) => git.to_path_buf(),
    }
}

/// The last `core.excludesFile` in a repository configuration file. An empty
/// value yields an empty path, which turns the global file off.
fn configured_excludes_file(root: &Path, config: &Path) -> Option<PathBuf> {
    let contents = fs::read_to_string(config).ok()?;
    let mut core = false;
    let mut file = None;
    for line in contents.lines() {
        let mut entry = line.trim_start();
        if let Some(header) = entry.strip_prefix('[') {
            let Some((section, rest)) = header.split_once(']') else { continue };
            core = section.trim().eq_ignore_ascii_case("core");
            entry = rest;
        }
        if core
            && let Some((key, value)) = entry.split_once('=')
            && key.trim().eq_ignore_ascii_case("excludesfile")
        {
            file = Some(config_value(value));
        }
    }

    let file = file?;
    if file.is_empty() {
        return Some(PathBuf::new());
    }
    Some(match file.strip_prefix("~/") {
        Some(relative) => home_dir()?.join(relative),
        None => root.join(file),
    })
}

/// A Git configuration value: quotes and escapes resolved, a comment dropped, and
/// the whitespace around the value trimmed unless it is quoted.
fn config_value(raw: &str) -> String {
    let mut value = String::new();
    let mut kept = 0;
    let mut quoted = false;
    let mut characters = raw.chars();
    while let Some(character) = characters.next() {
        match character {
            '"' => {
                quoted = !quoted;
                kept = value.len();
            }
            '\\' => {
                match characters.next() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('b') => value.push('\u{8}'),
                    Some(escaped) => value.push(escaped),
                    None => break,
                }
                kept = value.len();
            }
            '#' | ';' if !quoted => break,
            character if !quoted && character.is_whitespace() => {
                if !value.is_empty() {
                    value.push(character);
                }
            }
            character => {
                value.push(character);
                kept = value.len();
            }
        }
    }

    value.truncate(kept);
    value
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn write(root: &Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn repository(files: &[(&str, &str)]) -> TempDir {
        let directory = TempDir::new().unwrap();
        fs::create_dir_all(directory.path().join(".git/info")).unwrap();
        for (relative, contents) in files {
            write(directory.path(), relative, contents);
        }
        directory
    }

    fn ignored(ignores: &GitIgnores, root: &Path, relative: &str) -> bool {
        ignores.is_ignored(&root.canonicalize().unwrap().join(relative), false)
    }

    #[test]
    fn nested_rules_negations_and_info_exclude_follow_git_precedence() {
        let directory = repository(&[
            (".gitignore", "/vendor/\n*.log.php\ncache/\n!keep.log.php\n/build/*\n!/build/keep/\n"),
            (".git/info/exclude", "local.php\n"),
            ("src/.gitignore", "generated/\n!cache/\n"),
            ("vendor/package/src/Library.php", "<?php"),
            ("src/generated/Proxy.php", "<?php"),
            ("src/cache/Kept.php", "<?php"),
            ("build/keep/Kept.php", "<?php"),
            ("build/other/Dropped.php", "<?php"),
        ]);
        let root = directory.path();
        let global = root.join("global-ignore");
        fs::write(&global, "*.swp.php\n").unwrap();
        let ignores = GitIgnores::discover_with_global(root, Some(&global)).unwrap();

        assert!(ignored(&ignores, root, "src/Editor.swp.php"));
        assert!(ignored(&ignores, root, "vendor/package/src/Library.php"));
        assert!(ignored(&ignores, root, "src/generated/Proxy.php"));
        assert!(ignored(&ignores, root, "var/cache/Container.php"));
        assert!(ignored(&ignores, root, "src/debug.log.php"));
        assert!(ignored(&ignores, root, "config/local.php"));
        assert!(ignored(&ignores, root, ".git/hooks/pre-commit.php"));
        assert!(ignored(&ignores, root, "build/other/Dropped.php"));
        assert!(!ignored(&ignores, root, "build/keep/Kept.php"));
        assert!(!ignored(&ignores, root, "src/keep.log.php"));
        assert!(!ignored(&ignores, root, "src/cache/Kept.php"));
        assert!(!ignored(&ignores, root, "src/Service.php"));
        assert!(!ignored(&ignores, root, "vendor.php"));
    }

    #[test]
    fn rules_inside_ignored_directories_are_never_read() {
        let directory = repository(&[(".gitignore", "build/\n"), ("build/.gitignore", "!*.php\n")]);
        let root = directory.path();
        let ignores = GitIgnores::discover_with_global(root, None).unwrap();

        assert!(ignored(&ignores, root, "build/Kept.php"));
        assert!(!ignores.directories.contains_key(&root.canonicalize().unwrap().join("build")));
    }

    #[test]
    fn workspaces_below_the_root_inherit_parent_rules_and_skip_ignored_roots() {
        let directory = repository(&[
            (".gitignore", "copies/\n"),
            ("app/.gitignore", "/var/\n"),
            ("app/src/Service.php", "<?php"),
            ("copies/app/src/Service.php", "<?php"),
        ]);
        let root = directory.path();
        let ignores = GitIgnores::discover_with_global(&root.join("app"), None).unwrap();

        assert!(ignored(&ignores, root, "app/var/Cache.php"));
        assert!(!ignored(&ignores, root, "app/src/Service.php"));
        assert!(GitIgnores::discover_with_global(&root.join("copies/app"), None).is_none());
    }

    #[test]
    fn nested_repositories_use_their_own_rules_inside_their_root() {
        let directory = repository(&[
            (".gitignore", "*.local.php\npackages/ignored/\n"),
            ("packages/tool/.git/info/exclude", "scratch/\n"),
            ("packages/tool/src/Service.php", "<?php"),
            ("packages/ignored/.git/HEAD", "ref: refs/heads/main\n"),
        ]);
        let root = directory.path();
        let ignores = GitIgnores::discover_with_global(root, None).unwrap();

        assert!(ignored(&ignores, root, "packages/tool/scratch/Notes.php"));
        assert!(!ignored(&ignores, root, "packages/tool/config.local.php"));
        assert!(!ignored(&ignores, root, "packages/tool/src/Service.php"));
        assert!(ignored(&ignores, root, "packages/ignored/src/Service.php"));
        assert!(ignored(&ignores, root, "app.local.php"));
        assert!(!ignored(&ignores, root, "scratch/Notes.php"));
    }

    #[test]
    fn repository_excludes_files_replace_the_global_one() {
        let directory = repository(&[
            (
                ".git/config",
                "[user]\n\texcludesFile = user-ignore\n[core] # local settings\n\texcludesFile = \"repository-ignore\" # local rules\n",
            ),
            ("repository-ignore", ".claude/worktrees/\n"),
            ("user-ignore", "src/\n"),
        ]);
        let root = directory.path();
        let global = root.join("global-ignore");
        fs::write(&global, "*.php\n").unwrap();
        let ignores = GitIgnores::discover_with_global(root, Some(&global)).unwrap();

        assert!(ignored(&ignores, root, ".claude/worktrees/copy/src/Service.php"));
        assert!(!ignored(&ignores, root, "src/Service.php"));

        fs::write(root.join(".git/config"), "[core]\n\texcludesFile = repository-ignore\n\texcludesFile = \"\"\n")
            .unwrap();
        let ignores = GitIgnores::discover_with_global(root, Some(&global)).unwrap();
        assert!(!ignored(&ignores, root, "src/Service.php"));
        assert!(!ignored(&ignores, root, ".claude/worktrees/copy/src/Service.php"));
    }

    #[test]
    fn rules_depend_on_read_gitignores_new_ones_git_would_read_and_exclude_sources() {
        let directory = repository(&[
            (".gitignore", "/vendor/\n"),
            (".git/config", "[core]\n\texcludesFile = repository-ignore\n"),
            ("app/.gitignore", "/var/\n"),
            ("app/src/Service.php", "<?php"),
            ("other/.gitignore", "*.php\n"),
        ]);
        let root = directory.path().canonicalize().unwrap();
        let ignores = GitIgnores::discover_with_global(&root.join("app"), Some(&root.join("global-ignore"))).unwrap();
        for (relative, expected) in [
            (".gitignore", true),
            ("app/.gitignore", true),
            ("app/src/.gitignore", true),
            ("app/new/deep/.gitignore", true),
            ("app/var/.gitignore", false),
            ("other/.gitignore", false),
            ("vendor/.gitignore", false),
            (".git/info/exclude", true),
            ("repository-ignore", true),
            ("global-ignore", false),
            ("app/src/Service.php", false),
        ] {
            assert_eq!(ignores.depends_on(&root.join(relative)), expected, "{relative}");
        }

        let config = root.join(".git/config");
        assert!(!ignores.depends_on(&config));
        fs::write(&config, "[core]\n\texcludesFile = other-ignore\n").unwrap();
        assert!(ignores.depends_on(&config));

        let mut sources: Vec<_> = ignores.sources_outside().collect();
        sources.sort_unstable();
        let expected = [".git/config", ".git/info/exclude", ".gitignore", "repository-ignore"];
        assert_eq!(sources, expected.map(|relative| root.join(relative)));
    }

    #[test]
    fn config_values_follow_git_quoting_and_comments() {
        for (raw, expected) in [
            (" plain ", "plain"),
            ("\"with # hash\" # comment", "with # hash"),
            ("two words ; comment", "two words"),
            ("\" padded \"", " padded "),
            (r#"escaped\"quote"#, "escaped\"quote"),
            ("\"\"", ""),
            (r"ab\bcd", "ab\u{8}cd"),
        ] {
            assert_eq!(config_value(raw), expected, "{raw}");
        }
    }

    #[test]
    fn deep_paths_created_after_discovery_use_their_nearest_read_directory() {
        let directory = repository(&[(".gitignore", "*.cache.php\n"), ("src/.gitignore", "!kept.cache.php\n")]);
        let root = directory.path();
        let ignores = GitIgnores::discover_with_global(root, None).unwrap();
        let src = root.canonicalize().unwrap().join("src");
        assert_eq!(ignores.directories[&src].chain.len(), 2);

        assert!(ignored(&ignores, root, "src/a/b/c/d/e/drop.cache.php"));
        assert!(!ignored(&ignores, root, "src/a/b/c/d/e/kept.cache.php"));
        assert!(!ignored(&ignores, root, "src/a/b/c/d/e/Service.php"));
    }

    #[test]
    fn worktree_files_resolve_info_exclude_from_the_common_directory() {
        let directory = repository(&[(".git/info/exclude", "scratch/\n")]);
        let main = directory.path();
        fs::create_dir_all(main.join(".git/worktrees/feature")).unwrap();
        fs::write(main.join(".git/worktrees/feature/commondir"), "../..\n").unwrap();
        let worktree = TempDir::new().unwrap();
        let link = main.canonicalize().unwrap().join(".git/worktrees/feature");
        fs::write(worktree.path().join(".git"), format!("gitdir: {}\n", link.display())).unwrap();

        let ignores = GitIgnores::discover_with_global(worktree.path(), None).unwrap();
        assert!(ignored(&ignores, worktree.path(), "scratch/Notes.php"));
        assert!(!ignored(&ignores, worktree.path(), "src/Service.php"));
        let exclude = main.canonicalize().unwrap().join(".git/info/exclude");
        assert!(ignores.depends_on(&exclude));
        assert!(ignores.sources_outside().any(|source| source == exclude));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_excludes_files_depend_on_the_link_and_its_target() {
        let directory = repository(&[]);
        let home = TempDir::new().unwrap();
        let home = home.path().canonicalize().unwrap();
        let target = home.join("dotfiles/ignore");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "*.log\n").unwrap();
        let link = home.join(".gitignore_global");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let ignores = GitIgnores::discover_with_global(directory.path(), Some(&link)).unwrap();
        assert!(ignores.depends_on(&link));
        assert!(ignores.depends_on(&target));
    }

    #[test]
    fn excludes_files_named_gitignore_outside_the_workspace_are_sources() {
        let directory = repository(&[]);
        let home = TempDir::new().unwrap();
        let global = home.path().canonicalize().unwrap().join(".gitignore");
        fs::write(&global, "*.log\n").unwrap();
        let ignores = GitIgnores::discover_with_global(directory.path(), Some(&global)).unwrap();
        assert!(ignores.depends_on(&global));
        assert!(ignores.sources_outside().any(|source| source == global));
    }

    #[test]
    fn paths_outside_a_work_tree_have_no_rules() {
        let directory = TempDir::new().unwrap();
        write(directory.path(), ".gitignore", "*\n");
        assert!(GitIgnores::discover_with_global(directory.path(), None).is_none());
    }
}
