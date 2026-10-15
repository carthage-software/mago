//! Build scoped LSP globs from the source matcher's resolved roots.

use std::path::Path;

use mago_database::membership::WorkspaceMatcher;

pub(super) fn for_matcher(matcher: &WorkspaceMatcher) -> Vec<String> {
    let mut patterns = Vec::new();
    for root in matcher.watch_roots() {
        let pattern = path_pattern(root);
        // An exact source may have no extension or may not exist yet.
        patterns.push(pattern.clone());
        if !root.is_file() {
            // Clients may report a directory move without events for its children.
            patterns.push(format!("{}/**", pattern.trim_end_matches('/')));
        }
    }
    patterns
}

pub(super) fn for_directory(root: &Path) -> [String; 2] {
    let pattern = path_pattern(root);
    let children = format!("{}/**", pattern.trim_end_matches('/'));
    [pattern, children]
}

fn path_pattern(path: &Path) -> String {
    let path = path.to_string_lossy();
    #[cfg(windows)]
    let path = if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!("//{unc}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(&path).to_owned()
    }
    .replace('\\', "/");
    escape_glob(&path)
}

fn escape_glob(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '?' | '*' | '[' | ']' | '{' | '}' => {
                escaped.push('[');
                escaped.push(character);
                escaped.push(']');
            }
            '\\' => escaped.push_str(r"\\"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use globset::GlobBuilder;

    use super::escape_glob;

    #[test]
    fn path_metacharacters_match_only_the_literal_path() {
        let path = "root[one]/{two}*.?";
        let pattern = escape_glob(path);
        let matcher = GlobBuilder::new(&pattern).literal_separator(true).build().unwrap().compile_matcher();
        assert!(matcher.is_match(path));
        assert!(!matcher.is_match("rooto/twoABC.x"));
    }
}
