use serde_json::json;

use super::harness::Harness;
use super::references::{apply, prepare_at, rename_at};

#[tokio::test]
async fn method_rename_updates_interface_and_override_family_across_files() {
    let base =
        "<?php interface Contract { function run(): void; } class Base implements Contract { function run(): void {} }";
    let child = "<?php class Child extends Base { function run(): void {} } function invoke(Contract $object): void { $object->run(); } (new Child())->run();";
    let other = "<?php class Other { function run(): void {} } (new Other())->run();";
    let mut h = Harness::start(&[("base.php", base), ("child.php", child), ("other.php", other)]).await;
    for (path, source, needle) in [("base.php", base, "run"), ("child.php", child, "run();")] {
        let edit = rename_at(&mut h, path, source, needle, "execute").await.unwrap();
        assert_eq!(edit["changes"].as_object().unwrap().len(), 2);
        assert_eq!(apply(base, &edit["changes"][h.url("base.php")]), base.replace("run", "execute"));
        assert_eq!(apply(child, &edit["changes"][h.url("child.php")]), child.replace("run", "execute"));
    }
}

#[tokio::test]
async fn property_constant_and_case_rename_preserve_syntax_and_inherited_uses() {
    let base = "<?php class Base { public int $value = 1; public static int $count = 1; public const TOKEN = 1; } enum Status { case On; case Off; }";
    let child = "<?php class Child extends Base { public int $value = 2; public const TOKEN = 2; } $object = new Child(); echo $object->value, Child::$count, Child::TOKEN; $status = Status::On;";
    let mut h = Harness::start(&[("base.php", base), ("child.php", child)]).await;
    for (needle, name, old) in [
        ("$value", "amount", "value"),
        ("$count", "total", "count"),
        ("TOKEN", "CODE", "TOKEN"),
        ("On;", "Active", "On"),
    ] {
        let edit = rename_at(&mut h, "base.php", base, needle, name).await.unwrap();
        assert_eq!(apply(base, &edit["changes"][h.url("base.php")]), base.replace(old, name));
        assert_eq!(apply(child, &edit["changes"][h.url("child.php")]), child.replace(old, name));
    }
    let prepare = h
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": h.url("base.php") },
                "position": { "line": 0, "character": base.find("$count").unwrap() + 2 }
            }),
        )
        .await;
    assert_eq!(prepare["placeholder"], "count");
    assert_eq!(prepare["range"]["start"]["character"], base.find("$count").unwrap() + 1);
}

#[tokio::test]
async fn private_member_rename_leaves_same_name_child_member_alone() {
    let source = "<?php class Base { private function run(): void {} function invoke(): void { $this->run(); } } class Child extends Base { function run(): void {} } (new Child())->run();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "run", "execute").await.unwrap();
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replacen("run", "execute", 2));
}

#[tokio::test]
async fn member_rename_rejects_collisions_magic_dynamic_and_unresolved_uses() {
    for (source, needle, name) in [
        ("<?php class C { function run(): void {} function taken(): void {} }", "run", "taken"),
        (
            "<?php class C { function run(): void {} } class Child extends C { function taken(): void {} }",
            "run",
            "taken",
        ),
        ("<?php class C { public int $value = 1; public int $taken = 2; }", "$value", "taken"),
        ("<?php enum E { case On; const Taken = 1; }", "On;", "Taken"),
        ("<?php class C { function __toString(): string { return ''; } }", "__toString", "changed"),
        (
            "<?php class C { function run(): void {} } $object = new C(); $method = 'run'; $object->$method();",
            "run",
            "changed",
        ),
        (
            "<?php class C { function run(): void {} } function invoke($unknown): void { $unknown->run(); }",
            "run",
            "changed",
        ),
        (
            "<?php class A { function run(): void {} } class B { function run(): void {} } function invoke(A|B $object): void { $object->run(); }",
            "run",
            "changed",
        ),
        (
            "<?php trait T { function run(): void {} } class C { use T { run as alias; } } (new C())->run();",
            "run",
            "changed",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(rename_at(&mut h, "a.php", source, needle, name).await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn plain_trait_methods_rename_without_touching_unrelated_adaptations() {
    let source = "<?php trait T { function run(): void {} } class C { use T; } trait Other { function run(): void {} } class D { use Other { run as alias; } } (new C())->run(); (new D())->run();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "run", "execute").await.unwrap();
    let expected = source.replacen("run", "execute", 1).replace("(new C())->run()", "(new C())->execute()");
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected);
}

#[tokio::test]
async fn member_rename_does_not_capture_existing_magic_dispatch() {
    for (source, needle) in [
        (
            "<?php class C { function run(): void {} function __call($name, $args): void {} } (new C())->execute();",
            "run",
        ),
        (
            "<?php class C { public int $value = 1; function __get($name): int { return 2; } } echo (new C())->execute;",
            "$value",
        ),
        (
            "<?php class C { public int $value = 1; function __set($name, $value): void {} } $object = new C(); $object->execute = 2;",
            "$value",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(rename_at(&mut h, "a.php", source, needle, "execute").await.is_err(), "{source}");
    }
    let source = "<?php class C { function run(): void {} } class Other { function __call($name, $args): void {} } (new C())->run(); (new Other())->execute();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "run", "execute").await.unwrap();
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace("run", "execute"));
}

#[tokio::test]
async fn method_rename_declines_known_callable_strings_and_arrays() {
    for tail in [
        "$cb = [new C(), 'run'];",
        "$object = new C(); $cb = [$object, 'run'];",
        "function make(C $object): array { return [$object, 'run']; }",
        "$cb = [C::class, 'run'];",
        "$cb = ['C', 'run'];",
        "$cb = 'C::run';",
        "$cb = array(C::class, \"ru\\x6e\");",
        "$cb = [1 => 'run', 0 => C::class];",
        "$cb = ['1' => 'run', '0' => 'C'];",
        "class D extends C { function make(): array { return [parent::class, 'run']; } }",
        "function unknown($object): array { return [$object, 'run']; }",
    ] {
        let source = format!("<?php class C {{ static function run(): void {{}} }} {tail}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "run", "execute").await.is_err(), "{source}");
        let position = source.find("run").unwrap() as u32;
        assert!(prepare_at(&mut h, "a.php", 0, position).await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn method_rename_respects_callable_class_aliases_and_unrelated_receivers() {
    let source = "<?php namespace Lib; class C { static function run(): void {} } class Other { static function run(): void {} } namespace App; use Lib\\C as Alias; use Lib\\Other; $cb = [Alias::class, 'run'];";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert!(rename_at(&mut h, "a.php", source, "run", "execute").await.is_err());

    let source = "<?php namespace Lib; class C { static function run(): void {} } class Other { static function run(): void {} } namespace App; use Lib\\C as Alias; use Lib\\Other; $a = [Other::class, 'run']; $b = 'Lib\\Other::run'; $c = ['Lib\\Other', 'run']; $d = [new Other(), 'run']; Alias::run();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "run", "execute").await.unwrap();
    let expected = source.replacen("run", "execute", 1).replace("Alias::run()", "Alias::execute()");
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected);
}

#[tokio::test]
async fn method_rename_does_not_capture_callable_magic_dispatch() {
    for tail in ["$cb = [C::class, 'execute'];", "$cb = 'C::execute';", "$cb = ['C', 'execute'];"] {
        let source = format!(
            "<?php class C {{ static function run(): void {{}} static function __callStatic($name, $args): void {{}} }} {tail}"
        );
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "run", "execute").await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn member_rename_accepts_legal_keywords_and_non_ascii_names() {
    let source = "<?php class C { function run(): void {} public int $value = 1; const TOKEN = 1; } enum E { case On; } (new C())->run(); echo (new C())->value, C::TOKEN; $e = E::On;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, old, new) in
        [("run", "run", "match"), ("$value", "value", "int"), ("TOKEN", "TOKEN", "list"), ("On;", "On", "Actifé")]
    {
        let edit = rename_at(&mut h, "a.php", source, needle, new).await.unwrap();
        assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace(old, new));
    }
    for (needle, name) in
        [("TOKEN", "class"), ("On;", "ClAsS"), ("run", "__invoke"), ("run", "bad-name"), ("$value", "1value")]
    {
        assert!(rename_at(&mut h, "a.php", source, needle, name).await.is_err(), "{needle} -> {name}");
    }
}

#[tokio::test]
async fn callable_array_keys_and_spreads_do_not_leave_partial_member_renames() {
    for expression in [
        "[0.9 => C::class, 1.8 => 'run']",
        "array(0.0 => C::class, 1.0 => 'run')",
        "[0 => C::class, 1 => 'old', 1 => 'run']",
        "[C::class, 'run', 0 => C::class]",
        "[$key => C::class, 1 => 'run']",
        "[0 => C::class, 1 + 0 => 'run']",
        "[C::class, 'run', ...[]]",
        "[C::class, ...['run']]",
        "[C::class, ...[...['run']]]",
        "[...$items, 'run']",
        "[18446744073709551615 => C::class, 1 => 'run']",
        "[1e400 => C::class, 1 => 'run']",
    ] {
        let source = format!(
            "<?php class C {{ static function run(): void {{}} }} $key = 0; $items = [C::class]; $cb = {expression}; $cb();"
        );
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "run", "execute").await.is_err(), "{expression}");
    }
}

#[tokio::test]
async fn callable_array_guards_respect_last_writes_and_unrelated_receivers() {
    for expression in [
        "[0.9 => Other::class, 1.8 => 'run']",
        "[0 => C::class, 1 => 'run', 0 => Other::class]",
        "[C::class, 'run', 1 => 'other']",
        "[C::class, 'run', ...$items, 1 => 'other']",
    ] {
        let source = format!(
            "<?php class C {{ static function run(): void {{}} static function other(): void {{}} }} class Other {{ static function run(): void {{}} }} $items = []; $cb = {expression}; C::run();"
        );
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", &source, "run", "execute").await.unwrap();
        let expected = source.replacen("run", "execute", 1).replace("C::run()", "C::execute()");
        assert_eq!(apply(&source, &edit["changes"][h.url("a.php")]), expected, "{expression}");
    }
}

#[tokio::test]
async fn callable_rename_checks_runtime_children_trait_users_and_unknown_union_arms() {
    for source in [
        "<?php class Base { static function callback(): array { return [static::class, 'run']; } } class Child extends Base { static function run(): void {} } $cb = Child::callback(); $cb();",
        "<?php class Base {} class Child extends Base { static function run(): void {} } function callback(Base $object): array { return [$object, 'run']; } $cb = callback(new Child()); $cb();",
        "<?php interface Marker {} class C implements Marker { static function run(): void {} } function callback(Marker $object): array { return [$object, 'run']; } $cb = callback(new C()); $cb();",
        "<?php trait T { static function callback(): array { return [static::class, 'run']; } } trait U { use T; } class Base { use U; } class Child extends Base { static function run(): void {} } $cb = Child::callback(); $cb();",
        "<?php trait T { static function callback(): array { return [self::class, 'run']; } } trait U { use T; } class C { use U; static function run(): void {} } $cb = C::callback(); $cb();",
        "<?php class Other { static function run(): void {} } class C { static function run(): void {} } function callback(Other|string $object): array { return [$object, 'run']; } $cb = callback(C::class); $cb();",
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        let needle = if source.contains("class Other") { "run(): void {} } function callback" } else { "run():" };
        assert!(rename_at(&mut h, "a.php", source, needle, "execute").await.is_err(), "{source}");
    }
    let source = "<?php class Base { static function callback(): array { return [static::class, 'execute']; } } class Child extends Base { static function run(): void {} static function __callStatic($name, $args): void {} } $cb = Child::callback(); $cb();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert!(rename_at(&mut h, "a.php", source, "run", "execute").await.is_err());

    let source = "<?php class C { static function run(): void {} } class Other { static function run(): void {} } function callback(?Other $object): array { return [$object, 'run']; } C::run();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "run", "execute").await.unwrap();
    let expected = source.replacen("run", "execute", 1).replace("C::run()", "C::execute()");
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected);
}
