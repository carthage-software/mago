use mago_word::WordSet;

use crate::metadata::CodebaseMetadata;

#[allow(clippy::set_contains_or_insert, reason = "A duplicate insert can grow a full set.")]
pub(super) fn update(codebase: &mut CodebaseMetadata, repopulated: &WordSet) {
    let mut parents_to_refresh = repopulated.clone();
    let class_likes = &codebase.class_likes;

    codebase.direct_classlike_descendants.retain(|parent, children| {
        let previous_len = children.len();
        children.retain(|child| {
            class_likes.get(child).is_some_and(|metadata| {
                !repopulated.contains(child)
                    || metadata.direct_parent_class == Some(*parent)
                    || metadata.direct_parent_interfaces.contains(parent)
            })
        });

        if children.len() != previous_len {
            parents_to_refresh.insert(*parent);
        }

        !children.is_empty()
    });

    codebase.all_class_like_descendants.retain(|parent, children| {
        children.retain(|child| {
            class_likes.get(child).is_some_and(|metadata| {
                !repopulated.contains(child)
                    || metadata.all_parent_classes.contains(parent)
                    || metadata.all_parent_interfaces.contains(parent)
                    || metadata.used_traits.contains(parent)
            })
        });

        !children.is_empty()
    });

    for name in repopulated {
        let Some(metadata) = class_likes.get(name) else {
            continue;
        };

        for parent in metadata.direct_parent_interfaces.iter().copied().chain(metadata.direct_parent_class) {
            let children = codebase.direct_classlike_descendants.entry(parent).or_default();
            if !children.contains(name) {
                children.insert(*name);
            }

            parents_to_refresh.insert(parent);
        }

        for parent in
            metadata.all_parent_interfaces.iter().chain(&metadata.all_parent_classes).chain(&metadata.used_traits)
        {
            let children = codebase.all_class_like_descendants.entry(*parent).or_default();
            if !children.contains(name) {
                children.insert(*name);
            }
        }
    }

    for parent in parents_to_refresh {
        if let Some(metadata) = codebase.class_likes.get_mut(&parent) {
            metadata.child_class_likes = codebase.direct_classlike_descendants.get(&parent).cloned();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::fmt::Write;

    use foldhash::HashSet;
    use foldhash::fast::FixedState;
    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_php_version::PHPVersion;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;
    use mago_word::empty_word;
    use mago_word::word;

    use crate::metadata::CodebaseMetadata;
    use crate::populator::populate_codebase;
    use crate::populator::populate_codebase_targeted;
    use crate::reference::SymbolReferences;
    use crate::scanner::scan_program;

    fn scan(source: &str) -> CodebaseMetadata {
        let file = File::ephemeral(Cow::Borrowed(b"descendants.php"), Cow::Owned(source.as_bytes().to_vec()));
        let arena = LocalArena::new();
        let program = parse_file(&arena, &file);
        assert!(!program.has_errors(), "fixture did not parse: {:?}", program.errors);
        let names = NameResolver::new(&arena).resolve(program);
        scan_program(&arena, &file, program, &names, PHPVersion::LATEST)
    }

    fn populate_full(codebase: &mut CodebaseMetadata) {
        populate_codebase(codebase, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());
    }

    fn check_update(codebase: &mut CodebaseMetadata, source: &str, changed: &[&str]) {
        let mut scanned = scan(source);
        let mut expected = scanned.clone();
        populate_full(&mut expected);

        codebase.class_likes.retain(|name, _| scanned.class_likes.contains_key(name));
        codebase.symbols = scanned.symbols;
        let changed: WordSet = changed.iter().map(|name| word(*name)).collect();
        for name in &changed {
            if let Some(metadata) = scanned.class_likes.remove(name) {
                codebase.class_likes.insert(*name, metadata);
            }
        }

        let safe = codebase.class_likes.keys().filter(|name| !changed.contains(*name)).copied().collect();
        populate_codebase_targeted(
            codebase,
            &mut SymbolReferences::new(),
            safe,
            HashSet::default(),
            changed.into_iter().map(|name| (name, empty_word())).collect(),
        );

        assert_eq!(codebase.direct_classlike_descendants, expected.direct_classlike_descendants, "{source}");
        assert_eq!(codebase.all_class_like_descendants, expected.all_class_like_descendants, "{source}");
        for (name, metadata) in &expected.class_likes {
            assert_eq!(codebase.class_likes[name].child_class_likes, metadata.child_class_likes, "{name}: {source}");
        }
    }

    #[test]
    fn targeted_descendants_match_full_after_parent_and_child_changes() {
        let roots = "class root {} class a extends root {} class b {} class safe {}";
        let source = format!("<?php {roots} class child extends a {{}}");
        let mut codebase = scan(&source);
        populate_full(&mut codebase);

        check_update(&mut codebase, &source, &["a"]);
        let source = format!("<?php {roots} class child extends b {{}}");
        check_update(&mut codebase, &source, &["child"]);

        check_update(&mut codebase, &format!("<?php {roots}"), &[]);
        check_update(&mut codebase, &source, &["child"]);

        check_update(
            &mut codebase,
            "<?php class root {} class a extends root {} class safe {} class child extends b {}",
            &[],
        );

        check_update(&mut codebase, &source, &["b"]);
    }

    #[test]
    fn targeted_descendants_match_full_after_trait_and_interface_changes() {
        let roots = "trait inner {} trait outer { use inner; } interface base {} interface contract extends base {} class safe {}";
        let source = format!("<?php {roots} class user implements contract {{ use outer; }}");
        let mut codebase = scan(&source);
        populate_full(&mut codebase);

        check_update(&mut codebase, &source, &["contract", "outer"]);
        check_update(&mut codebase, &format!("<?php {roots} class user {{ use inner; }}"), &["user"]);
        check_update(&mut codebase, &source, &["user"]);
        check_update(&mut codebase, &format!("<?php {roots}"), &[]);
        check_update(&mut codebase, &source, &["user"]);

        check_update(
            &mut codebase,
            "<?php trait inner {} interface base {} interface contract extends base {} class safe {} class user implements contract { use outer; }",
            &["user"],
        );

        check_update(&mut codebase, &source, &["outer", "user"]);
    }

    #[test]
    fn unchanged_descendant_edges_keep_full_set_capacity() {
        let child_count = WordSet::with_capacity_and_hasher(64, FixedState::default()).capacity();
        let mut source = String::from("<?php class root {} interface contract {} trait common {}");
        let mut children = Vec::with_capacity(child_count);
        for index in 0..child_count {
            let _ = write!(source, " class child{index} extends root implements contract {{ use common; }}");
            children.push(word(format!("child{index}")));
        }

        let mut codebase = scan(&source);
        populate_full(&mut codebase);

        for child in children {
            super::update(&mut codebase, &std::iter::once(child).collect());
            for parent in ["root", "contract", "common"] {
                let descendants = &codebase.all_class_like_descendants[&word(parent)];
                assert_eq!(descendants.len(), child_count);
                assert_eq!(descendants.capacity(), child_count, "all descendants of {parent}");
            }
            for parent in ["root", "contract"] {
                let descendants = &codebase.direct_classlike_descendants[&word(parent)];
                assert_eq!(descendants.len(), child_count);
                assert_eq!(descendants.capacity(), child_count, "direct descendants of {parent}");
                assert_eq!(codebase.class_likes[&word(parent)].child_class_likes.as_ref(), Some(descendants));
            }
        }
    }
}
