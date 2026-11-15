#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::borrow::Cow;

use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::ttype::TType;
use mago_database::file::File;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;
use mago_word::WordSet;

fn scan(name: &'static [u8], file_type: FileType, code: &'static str) -> CodebaseMetadata {
    let file = File::new(Cow::Borrowed(name), file_type, None, Cow::Borrowed(code.as_bytes()));
    let arena = LocalArena::new();
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "parse failed: {:?}", program.errors);
    let resolved_names = NameResolver::new(&arena).resolve(program);

    scan_program(&arena, &file, program, &resolved_names, PHPVersion::LATEST)
}

fn populate(codebase: &mut CodebaseMetadata) {
    populate_codebase(codebase, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());
}

#[test]
fn full_population_resolves_template_arguments_of_vendored_classes() {
    let mut codebase = scan(
        b"builtin.php",
        FileType::Builtin,
        "<?php
        /**
         * @template TKey
         * @template-covariant TValue
         */
        interface Traversable {}

        /**
         * @template TKey
         * @template-covariant TValue
         *
         * @extends Traversable<TKey, TValue>
         */
        interface IteratorAggregate extends Traversable {}",
    );
    populate(&mut codebase);

    codebase.extend(scan(
        b"vendor/Node.php",
        FileType::Vendored,
        "<?php
        /** @implements IteratorAggregate<int|string, Node> */
        class Node implements IteratorAggregate {}",
    ));
    populate(&mut codebase);

    let node = codebase.get_class_like(b"Node").unwrap();
    for ancestor in [b"iteratoraggregate".as_slice(), b"traversable"] {
        let parameters =
            node.template_extended_parameters.iter().find(|(name, _)| name.as_bytes() == ancestor).unwrap().1;
        let value = parameters.get_index(1).unwrap().1;

        assert_eq!(value.get_id().as_str(), Some("Node"), "TValue of {}", String::from_utf8_lossy(ancestor));
    }
}
