use std::cmp::Reverse;

use foldhash::HashMap;
use mago_span::Span;
use mago_word::empty_word;

use crate::metadata::CodebaseMetadata;
use crate::reference::ReferenceSource;
use crate::signature::FileSignature;
use crate::symbol::SymbolIdentifier;

enum Entry {
    Scope(ReferenceSource),
    Closure(SymbolIdentifier),
}

pub(super) fn reference_sources(
    codebase: &CodebaseMetadata,
    include_populated: bool,
) -> HashMap<SymbolIdentifier, ReferenceSource> {
    if codebase.file_signatures.is_empty() {
        return HashMap::default();
    }

    let mut files = HashMap::<_, Vec<_>>::default();
    for (key, metadata) in &codebase.function_likes {
        if !metadata.flags.is_built_in()
            && (include_populated || !metadata.flags.is_populated())
            && (metadata.kind.is_closure() || metadata.kind.is_arrow_function())
        {
            files.entry(metadata.span.file_id).or_default().push((metadata.span, Entry::Closure(*key)));
        }
    }

    let mut sources = HashMap::default();
    for (file_id, mut entries) in files {
        let Some(signature) = codebase.get_file_signature(&file_id) else { continue };
        append_scopes(codebase, signature, &mut entries);
        entries.retain(|(span, _)| span.file_id == file_id);
        entries
            .sort_unstable_by_key(|(span, entry)| (span.start, Reverse(span.end), matches!(entry, Entry::Closure(_))));

        let mut scopes: Vec<(Span, ReferenceSource)> = Vec::new();
        for (span, entry) in entries {
            while scopes.last().is_some_and(|(outer, _)| outer.end < span.end) {
                scopes.pop();
            }
            match entry {
                Entry::Scope(source) => scopes.push((span, source)),
                Entry::Closure(key) => {
                    let source = scopes.last().map(|(_, source)| *source).or_else(|| {
                        (!signature.file_name.is_empty()).then_some(ReferenceSource::File(false, signature.file_name))
                    });
                    if let Some(source) = source {
                        sources.insert(key, source);
                    }
                }
            }
        }
    }
    sources
}

fn append_scopes(codebase: &CodebaseMetadata, signature: &FileSignature, entries: &mut Vec<(Span, Entry)>) {
    for node in &signature.ast_nodes {
        if node.is_function {
            if let Some(function) = codebase.function_likes.get(&(empty_word(), node.name)) {
                entries.push((function.span, Entry::Scope(ReferenceSource::Symbol(false, node.name))));
            }
        } else if let Some(class) = codebase.class_likes.get(&node.name) {
            entries.push((class.span, Entry::Scope(ReferenceSource::Symbol(false, node.name))));
            for child in &node.children {
                if child.is_function
                    && let Some(method) = codebase.function_likes.get(&(node.name, child.name))
                {
                    entries.push((
                        method.span,
                        Entry::Scope(ReferenceSource::ClassLikeMember(false, node.name, child.name)),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use foldhash::HashSet;
    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_php_version::PHPVersion;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;
    use mago_word::word;

    use super::*;
    use crate::populator::populate_codebase;
    use crate::populator::populate_codebase_after_body_edit;
    use crate::reference::{ReferenceOrigin, SymbolReferences};
    use crate::scanner::scan_program;
    use crate::signature_builder::build_file_signature;

    fn scan(name: &'static [u8], source: &'static [u8]) -> CodebaseMetadata {
        let file = File::ephemeral(name.into(), source.into());
        let arena = LocalArena::new();
        let program = parse_file(&arena, &file);
        assert!(!program.has_errors());
        let names = NameResolver::new(&arena).resolve(program);
        let mut codebase = scan_program(&arena, &file, program, &names, PHPVersion::PHP85);
        codebase.set_file_signature(file.id, build_file_signature(&file, program, &names));
        codebase
    }

    #[test]
    fn closures_use_nearest_named_scope_and_removable_file_references() {
        let source = b"<?php class Contact {}
            $callback = fn (Contact $global): int => 0;
            function outer() { return fn (Contact $outer) => fn (Contact $nested): int => 0; }
            function container() { function inner() { return fn (Contact $inner): int => 0; } }
            class Handler { public function run() { return fn (Contact $method): int => 0; } }";
        let mut codebase = scan(b"callbacks.php", source);
        let sources = reference_sources(&codebase, true);
        assert_eq!(sources.len(), 5);
        for (key, source) in sources {
            let expected = match codebase.function_likes[&key].parameters[0].name.0.as_bytes() {
                b"$global" => ReferenceSource::File(false, word("callbacks.php")),
                b"$outer" | b"$nested" => ReferenceSource::Symbol(false, word("outer")),
                b"$inner" => ReferenceSource::Symbol(false, word("inner")),
                b"$method" => ReferenceSource::ClassLikeMember(false, word("handler"), word("run")),
                name => panic!("unexpected parameter: {name:?}"),
            };
            assert_eq!(source, expected);
        }
        let mut references = SymbolReferences::new();
        populate_codebase(&mut codebase, &mut references, WordSet::default(), HashSet::default());
        let target = (word("contact"), empty_word());
        assert!(references.has_file_reference_to_symbol(target));
        references.remove_references_from_files(&std::iter::once(word("callbacks.php")).collect());
        references.for_each_reference(|origin, _, _| assert!(!matches!(origin, ReferenceOrigin::File(_))));
    }

    #[test]
    fn body_edits_populate_fresh_closures_without_repopulating_unchanged_signatures() {
        let mut codebase = scan(
            b"kept.php",
            b"<?php class Contact {}
            $global = fn (Contact $value): int => 0;
            function kept() { return fn (Contact $value): int => 0; }",
        );
        codebase.extend(scan(b"edited.php", b"<?php function edited() { return fn (Contact $value): int => 0; }"));
        populate_codebase(&mut codebase, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());

        let fresh = scan(b"edited.php", b"<?php function edited() { return fn (Contact $value): int => 1; }");
        let dirty = fresh.function_likes.keys().copied().collect();
        for key in fresh.function_likes.keys() {
            codebase.function_likes.remove(key);
        }
        codebase.extend(fresh);
        assert_eq!(reference_sources(&codebase, false).len(), 1);

        let mut references = SymbolReferences::new();
        populate_codebase_after_body_edit(
            &mut codebase,
            &mut references,
            std::iter::once(word("contact")).collect(),
            dirty,
        );
        assert!(codebase.function_likes.values().all(|metadata| metadata.flags.is_populated()));
        let mut count = 0;
        references.for_each_reference(|origin, target, _| {
            assert_eq!(origin, ReferenceOrigin::Symbol((word("edited"), empty_word())));
            assert_eq!(target, (word("contact"), empty_word()));
            count += 1;
        });
        assert!(count > 0);
    }
}
