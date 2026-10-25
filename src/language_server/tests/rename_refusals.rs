use serde_json::json;

use super::harness::Harness;
use super::references::{prepare_at, rename_at, rename_messages};
use crate::language_server::ServerConfig;

fn column(source: &str, needle: &str) -> u32 {
    source.find(needle).unwrap() as u32
}

async fn prepare_needle(h: &mut Harness, source: &str, needle: &str) -> Result<serde_json::Value, String> {
    let before = &source[..source.find(needle).unwrap()];
    let line = before.matches('\n').count() as u32;
    let character = before.rsplit('\n').next().unwrap().len() as u32;
    prepare_at(h, "a.php", line, character).await
}

#[tokio::test]
async fn function_name_in_string_is_reported() {
    let source = r"<?php namespace Lib; if (!function_exists('Lib\run')) { function run(): void {} }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let expected = r"the name appears in the string 'Lib\run' at a.php:1, which PHP may resolve at runtime.";
    let prepared = prepare_at(&mut h, "a.php", 0, column(source, "run()")).await;
    assert_eq!(prepared, Err(format!("Cannot rename: {expected}")));
    let renamed = rename_at(&mut h, "a.php", source, "run()", "execute").await;
    assert_eq!(renamed, Err(format!("Cannot rename to \"execute\": {expected}")));
}

#[tokio::test]
async fn local_collision_names_the_binding() {
    let source = "<?php function f(array $commands) { foreach ($commands as $command) {} }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let prepared = prepare_at(&mut h, "a.php", 0, column(source, "$command)") + 1).await.unwrap();
    assert_eq!(prepared["placeholder"], "command");
    assert_eq!(
        rename_at(&mut h, "a.php", source, "$command)", "commands").await,
        Err("Cannot rename to \"commands\": $commands is already defined in the same scope at a.php:1.".into())
    );
}

#[tokio::test]
async fn invalid_names_explain_the_problem() {
    let source =
        "<?php class Box { const LIMIT = 1; public function size(): int { $count = self::LIMIT; return $count; } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, name, reason) in [
        ("$count =", "123bad", "it is not a valid PHP identifier."),
        ("$count =", "not valid", "it is not a valid PHP identifier."),
        ("$count =", "", "the new name is empty."),
        ("$count =", "this", "it is a reserved variable name."),
        ("Box", "self", "it is a reserved type name."),
        ("Box", "function", "it is a reserved PHP keyword."),
        ("LIMIT =", "class", "class is reserved for the ::class constant."),
        ("size", "__size", "method names starting with __ are reserved for magic methods."),
    ] {
        assert_eq!(
            rename_at(&mut h, "a.php", source, needle, name).await,
            Err(format!("Cannot rename to \"{name}\": {reason}")),
            "{needle} -> {name}"
        );
    }
}

#[tokio::test]
async fn namespace_imports_are_refused_with_their_name() {
    let library = r"<?php namespace Psl\Str; function pad(): string { return ''; }";
    let source = r"<?php namespace App; use Psl\Str; Str\pad();";
    let mut h = Harness::start(&[("lib.php", library), ("a.php", source)]).await;
    let expected = r"Psl\Str is a namespace import at a.php:1; namespace renames are not supported.";
    let position = column(source, "Str;");
    assert_eq!(prepare_at(&mut h, "a.php", 0, position).await, Err(format!("Cannot rename: {expected}")));
    assert_eq!(
        rename_at(&mut h, "a.php", source, "Str;", "S").await,
        Err(format!("Cannot rename to \"S\": {expected}"))
    );
}

#[tokio::test]
async fn vendor_declarations_are_named() {
    let vendor = tempfile::tempdir().unwrap();
    std::fs::write(
        vendor.path().join("library.php"),
        "<?php class VendorThing {} const VENDOR_TOKEN = 1;\nclass Bag { public function getString(): string { return ''; } }",
    )
    .unwrap();
    let mut config = ServerConfig::default();
    config.configuration.source.includes = vec![vendor.path().to_str().unwrap().to_owned()];
    let source =
        "<?php new VendorThing(); echo VENDOR_TOKEN; function f(Bag $bag): string { return $bag->getString(); }";
    let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
    for (name, line) in [("VendorThing", 1), ("VENDOR_TOKEN", 1), ("getString", 2)] {
        let refusal = prepare_at(&mut h, "a.php", 0, column(source, name)).await.unwrap_err();
        assert!(refusal.starts_with(&format!("Cannot rename: {name} is declared outside the project sources")));
        assert!(refusal.ends_with(&format!("library.php:{line}.")), "{refusal}");
        let renamed = rename_at(&mut h, "a.php", source, name, "Renamed").await.unwrap_err();
        assert!(renamed.ends_with(&format!("library.php:{line}.")), "{renamed}");
    }
}

#[tokio::test]
async fn existing_class_is_named() {
    let source = "<?php class DeleteController {}";
    let mut h = Harness::start(&[("a.php", source), ("b.php", "<?php\nclass IndexController {}")]).await;
    assert_eq!(
        rename_at(&mut h, "a.php", source, "DeleteController", "IndexController").await,
        Err("Cannot rename to \"IndexController\": IndexController already exists at b.php:2.".into())
    );
}

#[tokio::test]
async fn builtin_magic_dynamic_and_incomplete_hierarchy_are_named() {
    let source = "<?php strlen('x'); class Text { public function __toString(): string { return ''; } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(
        prepare_at(&mut h, "a.php", 0, column(source, "strlen")).await,
        Err("Cannot rename: strlen is declared in PHP's builtin stubs.".into())
    );
    assert_eq!(
        prepare_at(&mut h, "a.php", 0, column(source, "__toString")).await,
        Err("Cannot rename: __toString is a magic method.".into())
    );

    let source = "<?php class Runner { function run(): void {} } function call(Runner $runner, string $method): void { $runner->$method(); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let refusal = rename_at(&mut h, "a.php", source, "run()", "execute").await.unwrap_err();
    assert!(refusal.contains("$method at a.php:1"), "{refusal}");

    let source = "<?php class Child extends Missing { function run(): void {} }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(
        prepare_at(&mut h, "a.php", 0, column(source, "run")).await,
        Err("Cannot rename: Child at a.php:1 depends on Missing, which is not declared.".into())
    );
}

#[tokio::test]
async fn nothing_at_cursor_is_null_for_prepare_and_error_for_rename() {
    let source = "<?php\n\n$value = 1;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert!(prepare_at(&mut h, "a.php", 1, 0).await.unwrap().is_null());
    let params = json!({
        "textDocument": { "uri": h.url("a.php") }, "position": { "line": 1, "character": 0 }, "newName": "item"
    });
    assert_eq!(
        super::references::rename_or_refusal(&mut h, params).await,
        Err("Cannot rename to \"item\": no renameable symbol at the cursor.".into())
    );
    assert!(rename_messages(&mut h).await.is_empty());
}

#[tokio::test]
async fn php_provided_enum_methods_and_their_parameters_are_builtin() {
    let source =
        "<?php enum Status: string { case Draft = 'draft'; } enum Size: int { case Small = 1; } enum Flag { case On; }
interface HasCases { public static function cases(): array; } enum Kind implements HasCases { case A; }
function f(): void { Status::cases(); Size::from(1); Status::tryFrom(value: 'draft'); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, expected) in [
        ("cases();", "cases is declared in PHP's builtin stubs."),
        ("from(1)", "from is declared in PHP's builtin stubs."),
        ("value:", "value is declared in PHP's builtin stubs."),
        ("cases(): array", "PHP generates this method for the enum Kind at a.php:2."),
    ] {
        for _ in 0..3 {
            assert_eq!(prepare_needle(&mut h, source, needle).await, Err(format!("Cannot rename: {expected}")));
            assert_eq!(
                rename_at(&mut h, "a.php", source, needle, "renamed").await,
                Err(format!("Cannot rename to \"renamed\": {expected}")),
                "{needle}"
            );
        }
    }
}

#[tokio::test]
async fn vendor_family_refusals_name_the_overridden_declaration() {
    let vendor = tempfile::tempdir().unwrap();
    std::fs::write(
        vendor.path().join("z_base.php"),
        "<?php enum VendorStatus { case On; } abstract class VendorCommand { public function execute(int $input): int { return $input; } }",
    )
    .unwrap();
    for index in 0..12 {
        std::fs::write(
            vendor.path().join(format!("a_sibling_{index}.php")),
            format!(
                "<?php class VendorSibling{index} extends VendorCommand {{ public function execute(int $input): int {{ return $input; }} }}"
            ),
        )
        .unwrap();
    }
    let source =
        "<?php class LocalCommand extends VendorCommand { public function execute(int $input): int { return $input; } }
function run(VendorCommand $command): void { $command->execute(input: 1); VendorStatus::cases(); }";
    for _ in 0..3 {
        let mut config = ServerConfig::default();
        config.configuration.source.includes = vec![vendor.path().to_str().unwrap().to_owned()];
        let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
        for (needle, snippet) in [("execute(int", "execute"), ("$input)", "$input"), ("input: 1", "$input")] {
            for _ in 0..2 {
                let refusal = prepare_needle(&mut h, source, needle).await.unwrap_err();
                assert!(
                    refusal
                        .starts_with(&format!("Cannot rename: {snippet} is declared outside the project sources, at ")),
                    "{refusal}"
                );
                assert!(refusal.ends_with("z_base.php:1."), "{needle}: {refusal}");
                let renamed = rename_at(&mut h, "a.php", source, needle, "renamed").await.unwrap_err();
                assert!(renamed.ends_with("z_base.php:1."), "{needle}: {renamed}");
            }
        }
        assert_eq!(
            rename_at(&mut h, "a.php", source, "cases()", "all").await,
            Err("Cannot rename to \"all\": cases is declared in PHP's builtin stubs.".into())
        );
    }
}

#[tokio::test]
async fn named_arguments_of_builtin_methods_are_builtin() {
    let source = "<?php $date = new DateTimeImmutable(); $date->modify(modifier: '+1 day'); (new DateTimeImmutable())->format(format: 'Y');";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, name) in [("modifier:", "$modifier"), ("format:", "$format")] {
        let expected = format!("{name} is declared in PHP's builtin stubs.");
        assert_eq!(
            prepare_at(&mut h, "a.php", 0, column(source, needle)).await,
            Err(format!("Cannot rename: {expected}"))
        );
        assert_eq!(
            rename_at(&mut h, "a.php", source, needle, "value").await,
            Err(format!("Cannot rename to \"value\": {expected}"))
        );
    }
}

#[tokio::test]
async fn runtime_declared_constants_and_class_aliases_are_refused_at_prepare() {
    let source = "<?php define('ZZ_PROBE_A', 1); echo ZZ_PROBE_A; define('de', 2); echo de; class ZzProbeTarget {} class_alias(ZzProbeTarget::class, 'ZzProbeAlias'); new ZzProbeAlias();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, call) in [
        ("ZZ_PROBE_A;", "define('ZZ_PROBE_A', 1)"),
        ("de;", "define('de', 2)"),
        ("ZzProbeAlias()", "class_alias(ZzProbeTarget::class, 'ZzProbeAlias')"),
    ] {
        let expected = format!("the name is declared at runtime by {call} at a.php:1.");
        assert_eq!(
            prepare_at(&mut h, "a.php", 0, column(source, needle)).await,
            Err(format!("Cannot rename: {expected}"))
        );
        assert_eq!(
            rename_at(&mut h, "a.php", source, needle, "ZzRenamed").await,
            Err(format!("Cannot rename to \"ZzRenamed\": {expected}"))
        );
    }
}

#[tokio::test]
async fn variable_used_as_a_dynamic_method_name_renames_from_every_occurrence() {
    let source = "<?php class Target { function run(): void {} } function f(): void { $name = 'run'; (new Target())->$name(); echo $name; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$name =", "$name()", "$name;"] {
        let renamed = rename_at(&mut h, "a.php", source, needle, "method").await.unwrap();
        assert_eq!(
            super::references::apply(source, &renamed["changes"][h.url("a.php")]),
            source.replace("$name", "$method"),
            "{needle}"
        );
    }
}

#[tokio::test]
async fn union_receiver_reaching_an_unrelated_method_is_named() {
    let source = "<?php class Left { function go(): void {} } class Right { function go(): void {} } function f(Left|Right $x): void { $x->go(); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(
        rename_at(&mut h, "a.php", source, "go()", "run").await,
        Err("Cannot rename to \"run\": go at a.php:1 may also refer to a member outside this rename.".into())
    );
}

#[tokio::test]
async fn incomplete_hierarchy_names_direct_ancestors_as_written() {
    let source = "<?php namespace App;
class Child extends \\Lib\\MissingParent implements MissingB, MissingA { function run(): void {} }
class Sibling implements MissingB, MissingA { function walk(): void {} }
interface Contract extends MissingContract {}
class Implementation implements Contract { function work(): void {} }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, expected) in [
        ("run()", "Child at a.php:2 depends on \\Lib\\MissingParent"),
        ("walk()", "Sibling at a.php:3 depends on MissingB"),
        ("work()", "Implementation at a.php:5 depends on app\\missingcontract"),
    ] {
        assert_eq!(
            prepare_needle(&mut h, source, needle).await,
            Err(format!("Cannot rename: {expected}, which is not declared.")),
            "{needle}"
        );
    }
}

fn vendor_config(vendor: &tempfile::TempDir) -> ServerConfig {
    let mut config = ServerConfig::default();
    config.configuration.source.includes = vec![vendor.path().to_str().unwrap().to_owned()];
    config
}

#[tokio::test]
async fn vendor_families_are_refused_before_their_vendor_classes_are_checked() {
    let vendor = tempfile::tempdir().unwrap();
    std::fs::write(
        vendor.path().join("d_base.php"),
        "<?php class VendorBase { public function go(int $speed): void {} }
class VendorTaker extends VendorBase { public function go(int $speed): void {} public function move(): void {} }",
    )
    .unwrap();
    std::fs::write(
        vendor.path().join("b_broken.php"),
        "<?php class VendorBroken extends VendorBase implements MissingFirst { public function go(int $speed): void {} }
class VendorBroken2 extends VendorBase implements MissingSecond { public function go(int $speed): void {} }",
    )
    .unwrap();
    let source = "<?php class LocalRunner extends VendorBase { public function go(int $speed): void {} }
function run(LocalRunner $runner): void { $runner->go(speed: 1); }";
    for _ in 0..2 {
        let mut h = Harness::start_with(&[("a.php", source)], vendor_config(&vendor), json!({})).await;
        for (needle, snippet) in [("go(int", "go"), ("$speed)", "$speed"), ("speed: 1", "$speed")] {
            for _ in 0..3 {
                let expected = format!("{snippet} is declared outside the project sources, at ");
                let refusal = prepare_needle(&mut h, source, needle).await.unwrap_err();
                assert!(refusal.starts_with(&format!("Cannot rename: {expected}")), "{needle}: {refusal}");
                assert!(refusal.ends_with("d_base.php:1."), "{needle}: {refusal}");
                for name in ["renamed", "move"] {
                    let renamed = rename_at(&mut h, "a.php", source, needle, name).await.unwrap_err();
                    assert!(renamed.starts_with(&format!("Cannot rename to \"{name}\": {expected}")), "{renamed}");
                    assert!(renamed.ends_with("d_base.php:1."), "{needle} -> {name}: {renamed}");
                }
            }
        }
    }
}

#[tokio::test]
async fn project_family_refusals_follow_source_order() {
    let source = "<?php interface Runner { function run(): void; }
class RunnerOne extends MissingOne implements Runner { function run(): void {} }
class RunnerTwo extends MissingTwo implements Runner { function run(): void {} }
class RunnerThree extends MissingThree implements Runner { function run(): void {} }
interface Walker { function walk(): void; }
class WalkerOne implements Walker { function walk(): void {} function stroll(): void {} }
class WalkerTwo implements Walker { function walk(): void {} function stroll(): void {} }
class WalkerThree implements Walker { function walk(): void {} function stroll(): void {} }
interface Mover { function move(): void; }
class MoverOne implements Mover { function move(): void {} function shift(): void {} }
class MoverTwo extends MissingMover implements Mover { function move(): void {} }";
    for _ in 0..2 {
        let mut h = Harness::start(&[("a.php", source)]).await;
        for _ in 0..4 {
            let runner = "RunnerOne at a.php:2 depends on MissingOne, which is not declared.";
            assert_eq!(prepare_needle(&mut h, source, "run(): void;").await, Err(format!("Cannot rename: {runner}")));
            assert_eq!(
                rename_at(&mut h, "a.php", source, "run(): void;", "execute").await,
                Err(format!("Cannot rename to \"execute\": {runner}"))
            );
            assert_eq!(
                rename_at(&mut h, "a.php", source, "walk(): void;", "stroll").await,
                Err("Cannot rename to \"stroll\": stroll already exists at a.php:6.".into())
            );
            let mover = "MoverTwo at a.php:11 depends on MissingMover, which is not declared.";
            assert_eq!(prepare_needle(&mut h, source, "move(): void;").await, Err(format!("Cannot rename: {mover}")));
            assert_eq!(
                rename_at(&mut h, "a.php", source, "move(): void;", "shift").await,
                Err(format!("Cannot rename to \"shift\": {mover}"))
            );
        }
    }
}

#[tokio::test]
async fn hook_and_anonymous_class_parameters_rename() {
    let source = "<?php final class Person { public int $age { set(int $value) { $this->age = $value; } } }
function counter(): int { $counter = new class { public function bump(int $step): int { return $step + 1; } }; return $counter->bump(step: 2); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value)", "$value;"] {
        assert_eq!(prepare_needle(&mut h, source, needle).await.unwrap()["placeholder"], "value", "{needle}");
        let edit = rename_at(&mut h, "a.php", source, needle, "next").await.unwrap();
        assert_eq!(
            super::references::apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("$value", "$next"),
            "{needle}"
        );
    }
    for needle in ["$step)", "$step +", "step: 2"] {
        assert_eq!(prepare_needle(&mut h, source, needle).await.unwrap()["placeholder"], "step", "{needle}");
        let edit = rename_at(&mut h, "a.php", source, needle, "amount").await.unwrap();
        assert_eq!(
            super::references::apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("$step", "$amount").replace("step:", "amount:"),
            "{needle}"
        );
    }
}

#[tokio::test]
async fn project_methods_that_php_generates_for_an_enum_name_the_enum() {
    let source = "<?php enum Unrelated: int { case One = 1; }
interface WithCases { public static function cases(): array; }
enum Plain implements WithCases { case X; }
interface Fromable { public static function from(int|string $value): static; }
final class Parsed implements Fromable { public static function from(int|string $value): static { return new static(); } }
enum Level: int implements Fromable { case Low = 1; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, name, enumeration) in
        [("cases(): array", "all", "Plain at a.php:3"), ("$value): static {", "input", "Level at a.php:6")]
    {
        for _ in 0..3 {
            let expected = format!("PHP generates this method for the enum {enumeration}.");
            assert_eq!(prepare_needle(&mut h, source, needle).await, Err(format!("Cannot rename: {expected}")));
            assert_eq!(
                rename_at(&mut h, "a.php", source, needle, name).await,
                Err(format!("Cannot rename to \"{name}\": {expected}")),
                "{needle}"
            );
        }
    }
}

#[tokio::test]
async fn vendor_refusals_prefer_declarations_that_project_traits_and_interfaces_inherit() {
    let vendor = tempfile::tempdir().unwrap();
    std::fs::write(
        vendor.path().join("z_base.php"),
        "<?php namespace Lib; trait RunTrait { public function run(int $speed): void {} } abstract class Base { use RunTrait; }",
    )
    .unwrap();
    std::fs::write(
        vendor.path().join("a_sibling.php"),
        "<?php namespace Lib; class ASibling extends Base { public function run(int $speed): void {} }",
    )
    .unwrap();
    for source in [
        "<?php trait LocalRun { public function run(int $speed): void {} } final class LocalUser extends \\Lib\\Base { use LocalRun; }",
        "<?php interface LocalRunner { public function run(int $speed): void; } final class LocalImpl extends \\Lib\\Base implements LocalRunner {}",
    ] {
        let mut h = Harness::start_with(&[("a.php", source)], vendor_config(&vendor), json!({})).await;
        for (needle, snippet) in [("run(int", "run"), ("$speed)", "$speed")] {
            let expected = format!("{snippet} is declared outside the project sources, at ");
            let refusal = prepare_needle(&mut h, source, needle).await.unwrap_err();
            assert!(refusal.starts_with(&format!("Cannot rename: {expected}")), "{source}: {refusal}");
            assert!(refusal.ends_with("z_base.php:1."), "{source}: {refusal}");
            let renamed = rename_at(&mut h, "a.php", source, needle, "renamed").await.unwrap_err();
            assert!(renamed.ends_with("z_base.php:1."), "{source}: {renamed}");
        }
    }
}

#[tokio::test]
async fn variables_used_by_name_at_runtime_are_refused() {
    let source = r#"<?php function named(): array { $alpha = 1; return compact("alpha"); }
function proposed(): array { $alpha = 1; return compact('first', ['second']); }
function extracted(array $row): int { $count = 1; extract($row); return $count; }
function listed(): array { $count = 1; return get_defined_vars(); }
function dynamic(string $name): mixed { $count = 1; return $$name; }
function braced(): mixed { $alpha = 1; $beta = 2; return ${'alpha'}; }
function parameter(int $alpha): array { return compact('alpha'); }
function captured(): callable { $alpha = 1; return function () use ($alpha) { return compact('alpha'); }; }
function evaluated(): mixed { $count = 1; return eval('return $count;'); }"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, construct, line) in [
        ("$alpha = 1; return compact(\"", "compact(\"alpha\")", 1),
        ("$count = 1; extract", "extract($row)", 3),
        ("$count = 1; return get", "get_defined_vars()", 4),
        ("$count = 1; return $$", "$$name", 5),
        ("$name)", "$$name", 5),
        ("$alpha = 1; $beta", "${'alpha'}", 6),
        ("$alpha): array", "compact('alpha')", 7),
        ("$alpha = 1; return function", "compact('alpha')", 8),
        ("$count = 1; return eval", "eval('return $count;')", 9),
    ] {
        let expected = format!("{construct} at a.php:{line} uses variables by name at runtime.");
        assert_eq!(prepare_needle(&mut h, source, needle).await, Err(format!("Cannot rename: {expected}")), "{needle}");
        assert_eq!(
            rename_at(&mut h, "a.php", source, needle, "renamed").await,
            Err(format!("Cannot rename to \"renamed\": {expected}")),
            "{needle}"
        );
    }
    let needle = "$alpha = 1; return compact('";
    assert_eq!(prepare_needle(&mut h, source, needle).await.unwrap()["placeholder"], "alpha");
    for name in ["first", "second"] {
        assert_eq!(
            rename_at(&mut h, "a.php", source, needle, name).await,
            Err(format!(
                "Cannot rename to \"{name}\": compact('first', ['second']) at a.php:2 uses variables by name at runtime."
            ))
        );
    }
    let edit = rename_at(&mut h, "a.php", source, needle, "third").await.unwrap();
    let changed = super::references::apply(source, &edit["changes"][h.url("a.php")]);
    assert!(changed.contains("function proposed(): array { $third = 1;"), "{changed}");
    let edit = rename_at(&mut h, "a.php", source, "$beta", "gamma").await.unwrap();
    let changed = super::references::apply(source, &edit["changes"][h.url("a.php")]);
    assert!(changed.contains("$gamma = 2; return ${'alpha'};"), "{changed}");
}

#[tokio::test]
async fn implicit_set_hook_value_is_refused() {
    let source = "<?php final class Person { public string $name { set { $this->name = strtoupper($value); } } public string $nick { set => trim($value, ' '); } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value)", "$value, ' '"] {
        let expected = "$value at a.php:1 is the implicit parameter of a set hook.";
        assert_eq!(prepare_needle(&mut h, source, needle).await, Err(format!("Cannot rename: {expected}")), "{needle}");
        assert_eq!(
            rename_at(&mut h, "a.php", source, needle, "v").await,
            Err(format!("Cannot rename to \"v\": {expected}")),
            "{needle}"
        );
    }
}
