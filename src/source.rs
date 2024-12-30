use ahash::HashSet;
use mago_interner::ThreadedInterner;
use mago_source::SourceCategory;
use mago_source::SourceManager;
use std::path::Path;

use crate::config::source::SourceConfiguration;
use crate::consts::PHP_STUBS;
use crate::error::Error;

/// Load the source manager by scanning and processing the sources
/// as per the given configuration.
///
/// # Arguments
///
/// * `interner` - The interner to use for string interning.
/// * `configuration` - The configuration to use for loading the sources.
/// * `include_stubs` - Whether to include stubs in the source manager.
///
/// # Returns
///
/// A `Result` containing the new source manager or a `SourceError` if
/// an error occurred during the build process.
pub async fn load(
    interner: &ThreadedInterner,
    configuration: &SourceConfiguration,
    include_stubs: bool,
) -> Result<SourceManager, Error> {
    let SourceConfiguration { root, paths, includes, excludes, extensions } = configuration;

    let manager = SourceManager::new(interner.clone());
    let extensions: HashSet<&String> = extensions.iter().collect();
    let has_paths = !paths.is_empty();
    let has_includes = !includes.is_empty();
    let has_excludes = !excludes.is_empty();

    let entries = jwalk::WalkDir::new(root.clone()).process_read_dir(|_, _, _, children| {
        children.iter_mut().for_each(|dir_entry_result| {
            if let Ok(dir_entry) = dir_entry_result {
                if dir_entry.path().starts_with(".") || dir_entry.file_name.eq_ignore_ascii_case("node_modules") {
                    dir_entry.read_children_path = None;
                }
            }
        });
    });

    for entry in entries {
        if let Err(_) = entry {
            continue;
        }

        let path = entry.unwrap().path();

        if !path.is_file() {
            continue;
        }

        if !is_accepted_file(&path, &extensions) {
            continue;
        }

        let name = match path.strip_prefix(root.clone()) {
            Ok(rel_path) => rel_path.display().to_string(),
            Err(_) => path.display().to_string(),
        };

        if has_excludes
            && excludes.iter().any(|p| {
                name.starts_with(p)
                    || glob_match::glob_match(p, name.as_str())
                    || glob_match::glob_match(p, path.to_string_lossy().as_ref())
            })
        {
            mago_feedback::debug!("Skipping: {:?}", name);
            continue;
        }

        let is_path = has_paths && paths.iter().any(|p| path.starts_with(p));

        let is_include = has_includes && includes.iter().any(|p| path.starts_with(p));

        if !is_path && !is_include {
            continue;
        }

        manager.insert_path(name, path.clone(), if is_include { SourceCategory::UserDefined } else { SourceCategory::External });
    }

    if include_stubs {
        for (stub, content) in PHP_STUBS {
            manager.insert_content(stub.to_owned(), content.to_owned(), SourceCategory::BuiltIn);
        }
    }

    Ok(manager)
}

fn is_accepted_file(path: &Path, extensions: &HashSet<&String>) -> bool {
    if extensions.is_empty() {
        path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("php")).unwrap_or(false)
    } else {
        path.extension().and_then(|s| s.to_str()).map(|ext| extensions.contains(&ext.to_string())).unwrap_or(false)
    }
}
