use serde_json::Value;
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_database::file::File;

use crate::language_server::position::{offset_at_position, position_at_offset};

use super::harness::Harness;

const DECLARATIONS: &str = "<?php namespace Toolkit;
class ContextConcrete {}
abstract class ContextAbstract {}
final class ContextFinal {}
interface ContextContract {}
trait ContextTrait {}
enum ContextEnum { case One; }
function ContextFunction(): void {}
const ContextConstant = 1;
";

async fn complete(h: &mut Harness, marked: &str, version: i32) -> (String, Vec<Value>) {
    let offset = marked.rfind('|').unwrap();
    let mut source = marked.to_owned();
    source.remove(offset);
    h.change("a.php", &source, version).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    (source, result["items"].as_array().unwrap().clone())
}

fn apply(source: &str, item: &Value) -> String {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
    let mut result = source.to_owned();
    result.replace_range(
        offset_at_position(&file, edit.range.start) as usize..offset_at_position(&file, edit.range.end) as usize,
        &edit.new_text,
    );
    result
}

fn has(items: &[Value], label: &str) -> bool {
    items.iter().any(|item| item["label"] == label)
}

#[tokio::test]
async fn function_import_completion_keeps_import_names_and_omits_call_syntax() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, qualifier) in [(2, "\\Toolkit"), (3, "Toolkit")] {
        let marked = format!("<?php namespace App; use function {qualifier}\\ContextF|unction;");
        let (source, items) = complete(&mut h, &marked, version).await;
        let item = items.iter().find(|item| item["label"] == "ContextFunction");
        assert!(item.is_some(), "{marked}: {items:?}");
        let item = item.unwrap();
        assert_eq!(item["textEdit"]["newText"], "ContextFunction", "{marked}");
        assert_eq!(apply(&source, item), marked.replace('|', ""));
        assert!(!has(&items, "ContextFinal"), "a function import must not offer a class: {items:?}");
    }
}

#[tokio::test]
async fn import_completion_handles_groups_lists_aliases_and_unfinished_names() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let cases = [
        (
            "<?php namespace App; use Elsewhere as Toolkit; use function Toolkit\\ContextF|unction;",
            "ContextFunction",
            3,
        ),
        ("<?php namespace App { use function Toolkit\\{ContextF|unction}; }", "ContextFunction", 3),
        ("<?php namespace App; use Toolkit\\{ContextConcrete, function ContextF|unction};", "ContextFunction", 3),
        ("<?php namespace App; use function Toolkit\\Other, Toolkit\\ContextF|unction;", "ContextFunction", 3),
        ("<?php namespace App; use const Toolkit\\ContextC|onstant;", "ContextConstant", 21),
        (
            "<?php namespace App; use Toolkit\\{function ContextFunction, const ContextC|onstant};",
            "ContextConstant",
            21,
        ),
        ("<?php namespace App; use Toolkit\\ContextC|oncrete;", "ContextConcrete", 7),
        ("<?php namespace App; use Toolkit\\{ContextConcrete as Other, ContextC|ontract};", "ContextContract", 8),
    ];
    for (index, (marked, label, kind)) in cases.iter().enumerate() {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        let item = items.iter().find(|item| item["label"] == *label);
        assert!(item.is_some(), "{marked}: {items:?}");
        let item = item.unwrap();
        assert_eq!(item["kind"], *kind, "{marked}");
        assert_eq!(item["textEdit"]["newText"], *label, "{marked}");
        assert_eq!(apply(&source, item), marked.replace('|', ""));
        if *kind == 3 || *kind == 21 {
            assert!(items.iter().all(|item| item["kind"] == *kind), "{marked}: {items:?}");
        } else {
            assert!(!has(&items, "ContextFunction") && !has(&items, "ContextConstant"), "{marked}: {items:?}");
        }
    }
    for (index, marked) in [
        "<?php namespace App; use function Toolkit\\ContextF|",
        "<?php namespace App; use function Toolkit\\|",
        "<?php namespace App; use Toolkit\\{function |",
    ]
    .iter()
    .enumerate()
    {
        let (source, items) = complete(&mut h, marked, index as i32 + 20).await;
        let item = items.iter().find(|item| item["label"] == "ContextFunction");
        assert!(item.is_some(), "{marked}: {items:?}");
        let expected = if marked.ends_with("ContextF|") {
            marked.replace("ContextF|", "ContextFunction")
        } else {
            marked.replace('|', "ContextFunction")
        };
        assert_eq!(apply(&source, item.unwrap()), expected);
    }
    let (_, items) = complete(&mut h, "<?php use function Toolkit\\ContextFunction as ContextF|;", 30).await;
    assert!(items.is_empty(), "an alias name must not suggest import targets: {items:?}");
}

#[tokio::test]
async fn constructor_completion_offers_only_instantiable_classes() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, marked) in [(2, "<?php new \\Toolkit\\Con|textConcrete();"), (3, "<?php new \\Toolkit\\Con|")] {
        let (_, items) = complete(&mut h, marked, version).await;
        assert!(has(&items, "ContextConcrete") && has(&items, "ContextFinal"), "{marked}: {items:?}");
        for excluded in ["ContextAbstract", "ContextContract", "ContextTrait", "ContextEnum", "ContextFunction"] {
            assert!(!has(&items, excluded), "{marked} offered {excluded}: {items:?}");
        }
    }
}

#[tokio::test]
async fn class_like_completion_obeys_interface_trait_and_parameter_contexts() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let cases: &[(&str, &[&str], &[&str])] = &[
        (
            "<?php class Subject implements \\Toolkit\\Con|textContract {}",
            &["ContextContract"],
            &["ContextConcrete", "ContextAbstract", "ContextFinal", "ContextTrait", "ContextEnum", "ContextFunction"],
        ),
        (
            "<?php interface Subject extends \\Toolkit\\Con|textContract {}",
            &["ContextContract"],
            &["ContextConcrete", "ContextAbstract", "ContextFinal", "ContextTrait", "ContextEnum", "ContextFunction"],
        ),
        (
            "<?php class Subject { use \\Toolkit\\Con|textTrait; }",
            &["ContextTrait"],
            &["ContextConcrete", "ContextContract", "ContextEnum", "ContextFunction"],
        ),
        (
            "<?php function accept(\\Toolkit\\Con|textConcrete $value): void {}",
            &["ContextConcrete", "ContextContract", "ContextEnum"],
            &["ContextTrait", "ContextFunction"],
        ),
    ];
    for (index, (marked, required, excluded)) in cases.iter().enumerate() {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        for label in *required {
            assert!(has(&items, label), "{marked} missed {label}: {items:?}");
        }
        for label in *excluded {
            assert!(!has(&items, label), "{marked} offered {label}: {items:?}");
        }
        let label = if marked.contains("textContract") {
            "ContextContract"
        } else if marked.contains("textTrait") {
            "ContextTrait"
        } else {
            "ContextConcrete"
        };
        let item = items.iter().find(|item| item["label"] == label).unwrap();
        assert_eq!(item["textEdit"]["newText"], label);
        assert_eq!(apply(&source, item), marked.replace('|', ""));
    }
}

#[tokio::test]
async fn type_completion_keeps_aliases_qualifiers_and_declaration_intent() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let cases = [
        (
            "<?php namespace App; use Toolkit as Types; function accept(?Types\\Con|textEnum $value): void {}",
            "ContextEnum",
        ),
        (
            "<?php use Toolkit\\ContextContract as ContractAlias; function accept(ContractA|lias $value): void {}",
            "ContractAlias",
        ),
        (
            "<?php function accept(\\Toolkit\\ContextConcrete|\\Toolkit\\Con|textContract $value): void {}",
            "ContextContract",
        ),
        ("<?php function make(): null|(\\Toolkit\\Con|textContract&Countable) {}", "ContextContract"),
        ("<?php $make = function () use ($value): \\Toolkit\\Con|textConcrete {};", "ContextConcrete"),
        ("<?php class Subject { public readonly \\Toolkit\\Con|textEnum $value; }", "ContextEnum"),
        ("<?php class Subject extends \\Toolkit\\Con|textAbstract {}", "ContextAbstract"),
        ("<?php class Subject implements Countable, \\Toolkit\\Con|textContract {}", "ContextContract"),
        ("<?php interface Subject extends Countable, \\Toolkit\\Con|textContract {}", "ContextContract"),
        ("<?php trait Subject { use OtherTrait, \\Toolkit\\Con|textTrait; }", "ContextTrait"),
    ];
    for (index, (marked, label)) in cases.iter().enumerate() {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        let item = items.iter().find(|item| item["label"] == *label);
        assert!(item.is_some(), "{marked}: {items:?}");
        assert_eq!(apply(&source, item.unwrap()), source);
        assert!(!has(&items, "ContextFunction") && !has(&items, "ContextConstant"), "{marked}: {items:?}");
        if marked.contains("extends ") && marked.contains("class ") {
            assert!(!has(&items, "ContextFinal") && !has(&items, "ContextContract"), "{marked}: {items:?}");
        }
    }
    for (index, marked) in [
        "<?php function accept(\\Toolkit\\Con|",
        "<?php function accept($first, ?\\Toolkit\\Con|",
        "<?php function make(): \\Toolkit\\Con|",
    ]
    .iter()
    .enumerate()
    {
        let (source, items) = complete(&mut h, marked, index as i32 + 20).await;
        let item = items.iter().find(|item| item["label"] == "ContextConcrete");
        assert!(item.is_some(), "{marked}: {items:?}");
        assert_eq!(apply(&source, item.unwrap()), source.strip_suffix("Con").unwrap().to_owned() + "ContextConcrete");
        assert!(!has(&items, "ContextTrait") && !has(&items, "ContextFunction"), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn type_completion_does_not_capture_default_values_or_call_arguments() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, marked) in [
        "<?php function accept($value = \\Toolkit\\ContextF|unction()) {}",
        "<?php function accept($value = foo(\\Toolkit\\ContextF|unction())) {}",
        "<?php consume($value, \\Toolkit\\ContextF|unction());",
        "<?php consume(new \\Toolkit\\ContextConcrete(), \\Toolkit\\ContextF|unction());",
    ]
    .iter()
    .enumerate()
    {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        let item = items.iter().find(|item| item["label"] == "ContextFunction");
        assert!(item.is_some(), "{marked}: {items:?}");
        assert_eq!(apply(&source, item.unwrap()), marked.replace('|', ""));
    }
}

#[tokio::test]
async fn parent_completion_keeps_instance_methods_in_instance_scope() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, modifier, present) in [(2, "", true), (3, "static ", false)] {
        let marked = format!(
            "<?php class Base {{ protected function boundMethod(): void {{}} }} \
             class Child extends Base {{ {modifier}function work(): void {{ parent::bound|Method(); }} }}"
        );
        let (source, items) = complete(&mut h, &marked, version).await;
        assert_eq!(has(&items, "boundMethod"), present, "{marked}: {items:?}");
        if present {
            let item = items.iter().find(|item| item["label"] == "boundMethod").unwrap();
            assert_eq!(apply(&source, item), marked.replace('|', ""));
        }
    }
}

#[tokio::test]
async fn scoped_method_completion_keeps_visibility_and_static_properties() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let declarations = "<?php class Base {
        protected function boundMethod(): void {}
        private function boundPrivate(): void {}
        public static function boundStatic(): void {}
        protected static int $boundStaticProperty = 1;
        protected int $boundInstanceProperty = 1;
        protected const boundConstant = 1;
    }
    class Unrelated {
        public function boundMethod(): void {}
        public static function boundStatic(): void {}
    }";
    let mut version = 2;
    for modifier in ["", "static "] {
        for receiver in ["parent", "self", "static", "Base", "Child", "Unrelated"] {
            let marked = format!(
                "{declarations} class Child extends Base {{
                private function boundOwn(): void {{}}
                private static function boundOwnStatic(): void {{}}
                {modifier}function work(): void {{ {receiver}::bound|Method(); }}
            }}"
            );
            let (source, items) = complete(&mut h, &marked, version).await;
            version += 1;
            let instance = modifier.is_empty() && receiver != "Unrelated";
            assert_eq!(has(&items, "boundMethod"), instance, "{marked}: {items:?}");
            assert!(has(&items, "boundStatic"), "{marked}: {items:?}");
            assert!(!has(&items, "boundPrivate"), "{marked}: {items:?}");
            let own = matches!(receiver, "self" | "static" | "Child");
            assert_eq!(has(&items, "boundOwn"), own && instance, "{marked}: {items:?}");
            assert_eq!(has(&items, "boundOwnStatic"), own, "{marked}: {items:?}");
            assert_eq!(has(&items, "$boundStaticProperty"), receiver != "Unrelated", "{marked}: {items:?}");
            assert_eq!(has(&items, "boundConstant"), receiver != "Unrelated", "{marked}: {items:?}");
            assert!(
                !has(&items, "boundInstanceProperty") && !has(&items, "$boundInstanceProperty"),
                "{marked}: {items:?}"
            );
            if instance {
                let item = items.iter().find(|item| item["label"] == "boundMethod").unwrap();
                assert_eq!(apply(&source, item), source);
            }
        }
    }
}

#[tokio::test]
async fn scoped_method_completion_follows_closure_and_named_function_boundaries() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let cases = [
        ("", "$fn = function () { self::bound|Method(); };", true, true),
        ("", "$fn = fn() => self::bound|Method();", true, true),
        ("", "$fn = static function () { self::bound|Method(); };", false, true),
        ("", "$fn = static fn() => self::bound|Method();", false, true),
        ("static ", "$fn = function () { self::bound|Method(); };", false, true),
        ("", "function nested() { Subject::bound|Method(); }", false, false),
        ("", "function nested() { $fn = function () { Subject::bound|Method(); }; }", false, false),
        ("", "function nested() { $fn = fn() => Subject::bound|Method(); }", false, false),
    ];
    for (index, (modifier, body, bound, class_scope)) in cases.iter().enumerate() {
        let marked = format!(
            "<?php class Subject {{
            private function boundMethod(): void {{}}
            private static function boundPrivateStatic(): void {{}}
            public static function boundPublicStatic(): void {{}}
            {modifier}function work(): void {{ {body} }}
        }}"
        );
        let (_, items) = complete(&mut h, &marked, index as i32 + 2).await;
        assert_eq!(has(&items, "boundMethod"), *bound, "{marked}: {items:?}");
        assert_eq!(has(&items, "boundPrivateStatic"), *class_scope, "{marked}: {items:?}");
        assert!(has(&items, "boundPublicStatic"), "{marked}: {items:?}");
    }
}

const RECEIVERS: &str = "<?php namespace Demo;
final class Bus { public function dispatch(): void {} }
enum Kind: string {
    case One = 'one';
    const DEFAULT = self::One;
    public static function make(): self { return self::One; }
    public function label(): string { return $this->value; }
}
";

fn controller(body: &str) -> String {
    format!(
        "<?php namespace Demo;
final class Controller {{
    const LIMIT = 1;
    private Bus $bus;
    public function run(Bus $other) {{
        {body}
    }}
}}
"
    )
}

async fn receivers() -> Harness {
    let mut h = Harness::start(&[("lib.php", RECEIVERS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    h
}

#[tokio::test]
async fn member_completion_inside_unclosed_control_heads() {
    let mut h = receivers().await;
    let mut version = 1;
    for (head, label) in [
        ("if ($this->|", "bus"),
        ("if (!$this->|", "bus"),
        ("if ($this->bus->|", "dispatch"),
        ("if ($other->dis|", "dispatch"),
        ("if ($this->bus) {} elseif ($other->|", "dispatch"),
        ("while ($this->bus->|", "dispatch"),
        ("switch ($other->|", "dispatch"),
        ("do {} while ($other->|", "dispatch"),
        ("foreach ($this->|", "bus"),
        ("return match ($this->|", "bus"),
        ("$fn = function () { foreach ($this->|", "bus"),
        ("$next = new Bus();\n        foreach ($next->|", "dispatch"),
        ("$next = $this->bus;\n        $value = match ($next->|", "dispatch"),
    ] {
        for tail in ["\n        return 1;", ""] {
            version += 1;
            let marked = controller(&format!("{head}{tail}"));
            let (_, items) = complete(&mut h, &marked, version).await;
            assert!(has(&items, label), "{marked}: {items:?}");
            assert!(!has(&items, "Controller") && !has(&items, "LIMIT"), "{marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn completion_at_line_end_before_following_statements() {
    let mut h = receivers().await;
    let mut version = 1;
    for (line, label, excluded) in [
        ("$this->|", "bus", "LIMIT"),
        ("$other->|", "dispatch", "bus"),
        ("self::|", "LIMIT", "bus"),
        ("\\Demo\\Kind::|", "One", "label"),
        ("$|", "$other", "Controller"),
        ("\\Demo\\|", "Bus", "dispatch"),
    ] {
        for tail in ["\n        return 1;", "", "\n        return 1;\n    }\n    public function after(): void {"] {
            version += 1;
            let marked = controller(&format!("{line}{tail}"));
            let (source, items) = complete(&mut h, &marked, version).await;
            let item = items.iter().find(|item| item["label"] == label);
            assert!(item.is_some(), "{marked}: {items:?}");
            assert!(!has(&items, excluded), "{marked}: {items:?}");
            let edit: TextEdit = serde_json::from_value(item.unwrap()["textEdit"].clone()).unwrap();
            let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
            let cursor = position_at_offset(&file, marked.rfind('|').unwrap() as u32);
            let start =
                if line == "$|" { position_at_offset(&file, marked.rfind('|').unwrap() as u32 - 1) } else { cursor };
            assert_eq!((edit.range.start, edit.range.end), (start, cursor), "{marked}: {item:?}");
        }
    }
}

#[tokio::test]
async fn static_completion_resolves_expression_receivers() {
    let mut h = receivers().await;
    for (version, marked) in [
        (2, "<?php function f(\\Demo\\Kind $k) { $k::| }"),
        (3, "<?php function f(\\Demo\\Kind $k) { $k::|\n    return 1;\n}"),
    ] {
        let (source, items) = complete(&mut h, marked, version).await;
        for label in ["One", "DEFAULT", "make"] {
            assert!(has(&items, label), "{marked} missed {label}: {items:?}");
        }
        for label in ["label", "Kind", "Bus"] {
            assert!(!has(&items, label), "{marked} offered {label}: {items:?}");
        }
        let item = items.iter().find(|item| item["label"] == "One").unwrap();
        assert_eq!(apply(&source, item), marked.replace('|', "One"));
    }
    let marked = "<?php function f(\\Demo\\Kind $k) { $k::ma| }";
    let (source, items) = complete(&mut h, marked, 4).await;
    let item = items.iter().find(|item| item["label"] == "make");
    assert!(item.is_some(), "{marked}: {items:?}");
    assert!(!has(&items, "One") && !has(&items, "Bus"), "{marked}: {items:?}");
    let item = item.unwrap();
    let text = item["textEdit"]["newText"].as_str().unwrap();
    assert!(text.starts_with("make("), "{item}");
    assert_eq!(apply(&source, item), marked.replace("ma|", text));
    for (index, marked) in [
        "<?php function f() { $unknown::| }",
        "<?php function f() { $unknown::Bu| }",
        "<?php function f() { ::| }",
        "<?php function f() { ->| }",
        "<?php function f() { 'Demo\\Bus'::| }",
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 5).await;
        assert!(items.is_empty(), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn static_completion_resolves_member_fetch_receivers() {
    let mut h = receivers().await;
    let mut version = 1;
    for line in [
        "$this->bus::|",
        "$this->current::|",
        "$other?->current::|",
        "self::PRIMARY::|",
        "\\Demo\\Kind::DEFAULT::|",
        "$this->current::ma|",
    ] {
        for tail in [";", "\n        return 1;"] {
            version += 1;
            let marked = format!(
                "<?php namespace Demo;
final class Station {{
    const PRIMARY = Kind::One;
    private Kind $bus;
    private Kind $current;
    public function run(?Station $other) {{
        {line}{tail}
    }}
}}
"
            );
            let (source, items) = complete(&mut h, &marked, version).await;
            let item = items.iter().find(|item| item["label"] == "make");
            assert!(item.is_some(), "{marked}: {items:?}");
            assert!(
                !has(&items, "dispatch") && !has(&items, "label") && !has(&items, "PRIMARY"),
                "{marked}: {items:?}"
            );
            if line.ends_with("::|") {
                assert!(has(&items, "One") && has(&items, "DEFAULT"), "{marked}: {items:?}");
                let item = items.iter().find(|item| item["label"] == "One").unwrap();
                assert_eq!(apply(&source, item), marked.replacen('|', "One", 1));
            } else {
                assert!(!has(&items, "One"), "{marked}: {items:?}");
            }
        }
    }
}

#[tokio::test]
async fn member_completion_in_dropped_heads_uses_parameters_and_skips_closures() {
    let mut h = receivers().await;
    let mut version = 1;
    for (head, offered) in [
        ("foreach ($other->|", true),
        ("$value = match ($other->|", true),
        ("foreach ($this->bus->dispatch() as $x) {}\n        foreach ($other->|", true),
        ("$next = new Bus();\n        $double = fn (int $v): int => $v * 2;\n        foreach ($next->|", true),
        ("$next = new Bus();\n        $all = array_map(static fn ($v) => $v, []);\n        foreach ($next->|", true),
        (
            "$next = new Bus();\n        $double = function (int $v): int { return $v * 2; };\n        foreach ($next->|",
            true,
        ),
        ("$fn = function () { foreach ($other->|", false),
        ("$next = new Bus();\n        $fn = function () { foreach ($next->|", false),
    ] {
        for tail in ["\n        return 1;", ""] {
            version += 1;
            let marked = controller(&format!("{head}{tail}"));
            let (_, items) = complete(&mut h, &marked, version).await;
            assert_eq!(has(&items, "dispatch"), offered, "{marked}: {items:?}");
            assert!(!has(&items, "Controller") && !has(&items, "LIMIT"), "{marked}: {items:?}");
        }
    }
    for (index, (marked, label, offered)) in [
        (
            "<?php namespace Demo;
final class Worker {
    public function __construct(private Bus $courier) {
        foreach ($courier->|
        return;
    }
}
",
            "dispatch",
            true,
        ),
        (
            "<?php namespace Demo;\nfunction work(Kind $kind) {\n    $value = match ($kind->|\n    return 1;\n}\n",
            "label",
            true,
        ),
        ("<?php namespace Demo;\nfunction work(Kind $kind) {\n    foreach ($kind::|\n    return 1;\n}\n", "make", true),
        (
            "<?php namespace Demo;\nfunction work(Bus ...$all) {\n    foreach ($all->|\n    return 1;\n}\n",
            "dispatch",
            false,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, version + index as i32 + 1).await;
        assert_eq!(has(&items, label), offered, "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn member_completion_in_dropped_heads_skips_static_and_dynamic_property_names() {
    let mut h = receivers().await;
    let mut version = 1;
    for (head, offered) in [
        ("foreach ($count->|", true),
        ("foreach ($count::|", true),
        ("foreach (Station::$count->|", false),
        ("foreach (static::$count->|", false),
        ("foreach (Station::$count?->|", false),
        ("foreach (Station::$count::|", false),
        ("foreach ($this->$count->|", false),
        ("foreach ($other?->$count->|", false),
        ("foreach ($$count->|", false),
        ("foreach ($this->depot::$count->|", false),
    ] {
        for tail in ["\n        return 1;", ""] {
            version += 1;
            let marked = format!(
                "<?php namespace Demo;
final class Station {{
    public static int $count = 0;
    private Station $depot;
    public function run(Kind $count, ?Station $other) {{
        {head}{tail}
    }}
}}
"
            );
            let (_, items) = complete(&mut h, &marked, version).await;
            assert_eq!(has(&items, "label") || has(&items, "make"), offered, "{marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn member_completion_resolves_dropped_property_fetch_receivers() {
    let mut h = receivers().await;
    let mut version = 1;
    for (line, offered, excluded) in [
        ("$this->bus::$|;", &["$count"][..], &["dispatch", "open", "NAME"][..]),
        ("$this->bus::$|", &["$count"][..], &["dispatch", "open", "NAME"][..]),
        ("$x = $this->bus::$|;", &["$count"][..], &["dispatch", "open", "NAME"][..]),
        ("$other->bus::$|;", &["$count"][..], &["dispatch", "open", "NAME"][..]),
        ("foreach ($this->bus::|", &["NAME", "$count", "make"][..], &["dispatch", "open"][..]),
        ("foreach ($other?->bus::|", &["NAME", "$count", "make"][..], &["dispatch", "open"][..]),
        ("$value = match ($this->bus::|", &["NAME", "$count", "make"][..], &["dispatch", "open"][..]),
        ("foreach ($this->bus->|", &["open"][..], &["dispatch", "make", "NAME"][..]),
        ("foreach ($this->next?->bus->|", &["open"][..], &["dispatch", "make", "NAME"][..]),
        ("$fn = function () { foreach ($this->bus->|", &["open"][..], &["dispatch"][..]),
    ] {
        for tail in ["\n        return 1;", ""] {
            version += 1;
            let marked = format!(
                "<?php namespace Demo;
final class Depot {{
    const NAME = 'depot';
    public static int $count = 0;
    public static function make(): self {{ return new self(); }}
    public function open(): void {{}}
}}
final class Station {{
    private Depot $bus;
    private ?Station $next = null;
    public function run(Station $other) {{
        {line}{tail}
    }}
}}
"
            );
            let (_, items) = complete(&mut h, &marked, version).await;
            for label in offered {
                assert!(has(&items, label), "{marked} missed {label}: {items:?}");
            }
            for label in excluded {
                assert!(!has(&items, label), "{marked} offered {label}: {items:?}");
            }
        }
    }
    for (index, marked) in [
        "<?php namespace Demo;\nfinal class Depot { public function open(): void {} }\nfinal class Station {\n    private Depot $bus;\n    public static function run() {\n        foreach ($this->bus->|\n        return 1;\n    }\n}\n",
        "<?php namespace Demo;\nfinal class Depot { public function open(): void {} }\nfinal class Station {\n    private int $bus = 0;\n    public function run() {\n        foreach ($this->bus->|\n        return 1;\n    }\n}\n",
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, version + index as i32 + 1).await;
        assert!(!has(&items, "open"), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn member_completion_in_dropped_heads_expands_parameter_types() {
    let mut h = receivers().await;
    let mut version = 1;
    for (docblock, parameter, label) in [
        ("", "parent $o", "walk"),
        ("/** @param Alias $o */", "$o", "dispatch"),
        ("/** @param null|$this $o */", "$o", "peek"),
        ("/** @param static $o */", "$o", "peek"),
        ("/** @param T $o */", "$o", "dispatch"),
        ("", "self $o", "peek"),
    ] {
        for head in ["$x = $o->|;", "foreach ($o->|\n        return 1;", "$value = match ($o->|\n        return 1;"] {
            version += 1;
            let marked = format!(
                "<?php namespace Demo;
class Node {{ public function walk(): void {{}} }}
/**
 * @psalm-type Alias = Bus
 * @template T of Bus
 */
class Crate extends Node {{
    public function peek(): void {{}}
    {docblock}
    public function run({parameter}) {{
        {head}
    }}
}}
"
            );
            let (_, items) = complete(&mut h, &marked, version).await;
            assert!(has(&items, label), "{marked}: {items:?}");
            assert!(!has(&items, "Crate") && !has(&items, "Node"), "{marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn completion_inside_operator_tokens_is_empty() {
    let mut h = receivers().await;
    for (index, marked) in [
        "<?php function f(\\Demo\\Bus $a) { $a-|>dispatch(); }",
        "<?php function f() { \\Demo\\Kind:|:One; }",
        "<?php function f(?\\Demo\\Bus $a) { $a?-|>dispatch(); }",
        "<?php function f(?\\Demo\\Bus $a) { $a?|->dispatch(); }",
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 2).await;
        assert!(items.is_empty(), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn qualified_completion_replaces_a_trailing_namespace_separator() {
    let mut h = Harness::start(&[
        ("lib.php", "<?php namespace Demo\\Text; function chr(): string { return ''; }"),
        ("a.php", "<?php"),
    ])
    .await;
    h.open("a.php", "<?php").await;
    let marked = "<?php \\Demo\\Te|\\";
    let (source, items) = complete(&mut h, marked, 2).await;
    let item = items.iter().find(|item| item["label"] == "Text\\chr");
    assert!(item.is_some(), "{marked}: {items:?}");
    let item = item.unwrap();
    let text = item["textEdit"]["newText"].as_str().unwrap();
    assert!(text.starts_with("Text\\"), "{item}");
    assert_eq!(apply(&source, item), format!("<?php \\Demo\\{text}"));
}
