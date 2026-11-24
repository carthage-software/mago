use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

fn count_semantics(source: &str) -> usize {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(program.errors.is_empty(), "test source did not parse: {:?}", program.errors);

    let names = NameResolver::new(&arena).resolve(program);
    let issues = SemanticsChecker::new(PHPVersion::PHP85).check(&file, program, &names);

    issues.iter().filter(|issue| issue.code.as_deref() == Some("semantics")).count()
}

#[test]
fn empty_list_in_destructuring_is_rejected() {
    for source in [
        "<?php [] = [1];",
        "<?php list() = [1];",
        "<?php list(,) = [1];",
        "<?php [,] = [1];",
        "<?php [, ,] = [1];",
        "<?php [[], $b] = [[1], 2];",
        "<?php [$b, []] = [1, [2]];",
        "<?php [$b, [,]] = [1, [2]];",
        "<?php list($b, list()) = [1, [2]];",
        "<?php ['a' => []] = ['a' => [1]];",
        "<?php $x = [] = [1];",
        "<?php function f(array $a): void { [] = $a; }",
        "<?php foreach ([[1]] as []) {}",
        "<?php foreach ([[1]] as $k => []) {}",
        "<?php foreach ([[1]] as [[]]) {}",
    ] {
        assert_eq!(count_semantics(source), 1, "{source}");
    }
}

#[test]
fn non_empty_list_in_destructuring_is_accepted() {
    for source in [
        "<?php [$a] = [1];",
        "<?php [, $b] = [1, 2];",
        "<?php [$a, ] = [1];",
        "<?php list(, $b) = [1, 2];",
        "<?php [[$a]] = [[1]];",
        "<?php [$a, [, $b]] = [1, [2, 3]];",
        "<?php foreach ([[1]] as [$a]) {}",
        "<?php $x = [];",
        "<?php $f = fn() => [];",
        "<?php var_dump([] == [1]);",
        "<?php function g(array $a = []): array { return []; }",
    ] {
        assert_eq!(count_semantics(source), 0, "{source}");
    }
}
