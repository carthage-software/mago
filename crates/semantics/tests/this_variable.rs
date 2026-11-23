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
fn this_variable_binding_is_rejected() {
    for source in [
        "function f() { try {} catch (Exception $this) {} }",
        "function f() { try {} catch (TypeError|ValueError $this) {} }",
        "function f() { global $this; }",
        "function f() { global $a, $this; }",
        "function f() { static $this; }",
        "function f() { static $a = 1, $this = 2; }",
        "class A { function f() { unset($this); } }",
        "function f() { $a = 1; unset($a, $this); }",
        "function f($this) {}",
        "function f(...$this) {}",
        "class A { function f(int $a, $this) {} }",
        "interface I { function f($this); }",
        "class A { function __construct(public $this) {} }",
        "$f = function ($this) {};",
        "$f = fn($this) => 1;",
        "class A { function f() { return function () use ($this) {}; } }",
        "class A { function f() { $a = 1; return function () use ($a, &$this) {}; } }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}")), 1, "{source}");
    }
}

#[test]
fn this_variable_use_is_accepted() {
    for source in [
        "class A { public $x; function f() { $this->x = 1; unset($this->x); return $this; } }",
        "class A { function f() { return function () { return $this; }; } }",
        "function f($thisValue) { global $thisVar; static $self = null; return $thisValue; }",
        "function f($THIS) { global $This; static $tHis; unset($THIS); try {} catch (Exception $THIS) {} return function () use ($THIS) {}; }",
        "function f() { try {} catch (Exception) {} }",
        "function f() { $name = 'this'; global $$name; unset($a['this']); }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}")), 0, "{source}");
    }
}
