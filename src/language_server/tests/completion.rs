use serde_json::Value;

use mago_database::file::File;

use crate::language_server::position::{offset_at_position, position_at_offset};

use super::harness::Harness;

async fn complete(marked: &str) -> Vec<Value> {
    let offset = marked.rfind('|').unwrap();
    let mut source = marked.to_owned();
    source.remove(offset);
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    result["items"].as_array().unwrap().clone()
}

fn labels(items: &[Value]) -> Vec<&str> {
    items.iter().map(|item| item["label"].as_str().unwrap()).collect()
}

#[tokio::test]
async fn completion_includes_prelude_symbols_and_members() {
    for (source, label, replacement, kind) in [
        ("<?php strle|;", "strlen", "strlen($1)", 3),
        ("<?php namespace App; strle|;", "strlen", "strlen($1)", 3),
        ("<?php namespace App; \\strle|;", "strlen", "strlen($1)", 3),
        ("<?php new DateTimeImmu|;", "DateTimeImmutable", "DateTimeImmutable", 7),
        ("<?php namespace App; new DateTimeImmu|;", "DateTimeImmutable", "\\DateTimeImmutable", 7),
        ("<?php namespace App; new \\DateTimeImmu|;", "DateTimeImmutable", "DateTimeImmutable", 7),
        ("<?php PHP_VERSION_I|;", "PHP_VERSION_ID", "PHP_VERSION_ID", 21),
        ("<?php namespace App; PHP_VERSION_I|;", "PHP_VERSION_ID", "PHP_VERSION_ID", 21),
        ("<?php namespace App; \\PHP_VERSION_I|;", "PHP_VERSION_ID", "PHP_VERSION_ID", 21),
        ("<?php DateTimeImmutable::createFromF|;", "createFromFormat", "createFromFormat($1)", 2),
    ] {
        let items = complete(source).await;
        let matches: Vec<_> = items.iter().filter(|item| item["label"] == label).collect();
        assert_eq!(matches.len(), 1, "{source}: {items:?}");
        assert_eq!(matches[0]["textEdit"]["newText"], replacement, "{source}");
        assert_eq!(matches[0]["kind"], kind, "{source}");
    }
}

#[tokio::test]
async fn completion_skips_comments_string_text_and_html() {
    for source in [
        "<?php class Widget {} // Wid|\n",
        "<?php class Widget {} # Wid|\n",
        "<?php class Widget {} /* Wid| */",
        "<?php class Widget {} /** Wid| */",
        "<?php class Widget {} 'Wid|';",
        "<?php class Widget {} \"Wid|\";",
        "<?php class Widget {} 'Wid|",
        "<?php class Widget {} \"Wid|",
        "<?php class Widget {} $value = <<<TXT\nWid|\nTXT;",
        "<?php class Widget {} $value = <<<'TXT'\nWid|\nTXT;",
        "<?php class Widget {} ?> <div>Wid|</div>",
    ] {
        assert!(complete(source).await.is_empty(), "{source}");
    }
    for source in ["<?php class Widget {} /* Wid */ new Wid|;", "<?php class Widget {} ?> <div></div> <?php new Wid|;"]
    {
        assert!(labels(&complete(source).await).contains(&"Widget"), "{source}");
    }
    assert!(labels(&complete("<?php $value = 1; echo \"$val|\";").await).contains(&"$value"));
}

const TYPES: &str = "<?php class Leaf { public int $value; public function CamelCase(): void {} } class Box { public Leaf $leaf; public function make(): Leaf { return new Leaf(); } } ";

#[tokio::test]
async fn completion_resolves_calls_grouping_chains_and_nullsafe_receivers() {
    for expression in ["$box->make()->|", "($box->leaf)->|", "$box?->leaf?->|", "(new Leaf())->|"] {
        let items = complete(&format!("{TYPES} function test(Box $box) {{ {expression} }}")).await;
        let labels = labels(&items);
        assert!(labels.contains(&"value") && labels.contains(&"CamelCase"), "{expression}: {labels:?}");
        assert!(!labels.contains(&"leaf") && !labels.contains(&"make"));
    }
    let items = complete(&format!("{TYPES} function test(Box $box) {{ $box->make()->Ca|melCase(); }}")).await;
    let method = items.iter().find(|item| item["label"] == "CamelCase").unwrap();
    assert_eq!(method["textEdit"]["newText"], "CamelCase");
    assert_eq!(
        method["textEdit"]["range"]["end"]["character"].as_u64().unwrap()
            - method["textEdit"]["range"]["start"]["character"].as_u64().unwrap(),
        9
    );
    let fresh = complete(&format!("{TYPES} function test(Box $box) {{ $box->make()->Ca| }}")).await;
    assert_eq!(fresh.iter().find(|item| item["label"] == "CamelCase").unwrap()["textEdit"]["newText"], "CamelCase($1)");
}

#[tokio::test]
async fn completion_filters_visibility_and_resolves_class_keywords() {
    let declarations = "<?php class Base { public static int $publicStatic; protected static int $protectedStatic; private static int $privateStatic; public int $publicField; protected int $protectedField; private int $privateField; public const PUBLIC_TAG = 1; protected const PROTECTED_TAG = 2; private const PRIVATE_TAG = 3; public function publicMethod() {} protected function protectedMethod() {} private function privateMethod() {} public static function staticMethod() {} } ";
    for receiver in ["self", "parent", "static"] {
        let items =
            complete(&format!("{declarations} class Child extends Base {{ function test() {{ {receiver}::| }} }}"))
                .await;
        let labels = labels(&items);
        for name in ["$publicStatic", "$protectedStatic", "PUBLIC_TAG", "PROTECTED_TAG", "staticMethod"] {
            assert!(labels.contains(&name), "{receiver}: {labels:?}");
        }
        for name in ["$privateStatic", "PRIVATE_TAG", "publicField", "privateMethod"] {
            assert!(!labels.contains(&name), "{receiver}: {labels:?}");
        }
    }
    for (suffix, allowed, denied) in [
        (
            "function test(Base $base) { $base->| }",
            vec!["publicField", "publicMethod"],
            vec!["protectedField", "privateField", "protectedMethod", "privateMethod", "staticMethod"],
        ),
        (
            "class Child extends Base { function test() { $this->| } }",
            vec!["publicField", "protectedField", "publicMethod", "protectedMethod"],
            vec!["privateField", "privateMethod", "staticMethod"],
        ),
        (
            "Base::|",
            vec!["$publicStatic", "PUBLIC_TAG", "staticMethod"],
            vec!["$protectedStatic", "$privateStatic", "PROTECTED_TAG", "PRIVATE_TAG"],
        ),
    ] {
        let items = complete(&format!("{declarations}{suffix}")).await;
        let labels = labels(&items);
        for name in allowed {
            assert!(labels.contains(&name), "{labels:?}");
        }
        for name in denied {
            assert!(!labels.contains(&name), "{labels:?}");
        }
    }
}

#[tokio::test]
async fn completion_uses_the_class_importing_a_trait_property_for_visibility() {
    let traits = "<?php trait Secrets { private int $secret = 0; protected int $shared = 0; private static int $staticSecret = 0; } trait Nested { use Secrets; } ";
    for (classes, allowed, denied) in [
        ("class Owner { use Nested; function test() { $this->| } }", vec!["secret", "shared"], vec!["$staticSecret"]),
        ("class Owner { use Nested; function test() { self::$|; } }", vec!["$staticSecret"], vec!["secret", "shared"]),
        (
            "class Owner { use Nested; } class Child extends Owner { function test() { $this->| } }",
            vec!["shared"],
            vec!["secret", "$staticSecret"],
        ),
        (
            "class Owner { use Nested; } class Other { use Secrets; function test(Owner $owner) { $owner->| } }",
            vec![],
            vec!["secret", "shared"],
        ),
        (
            "class Owner { use Nested; } class Child extends Owner { use Secrets; function test() { $this->| } }",
            vec!["secret", "shared"],
            vec!["$staticSecret"],
        ),
    ] {
        let items = complete(&format!("{traits}{classes}")).await;
        let labels = labels(&items);
        for name in allowed {
            assert!(labels.contains(&name), "{classes}: {labels:?}");
        }
        for name in denied {
            assert!(!labels.contains(&name), "{classes}: {labels:?}");
        }
    }
}

#[tokio::test]
async fn completion_includes_magic_properties_without_replacing_visible_native_properties() {
    let source = r"<?php
/**
 * @property-read string $title
 * @property-write int $count
 * @property bool $enabled
 * @property string $visible
 * @property string $hidden
 * @property string $shared
 */
class Base {
    public int $visible = 1;
    private int $hidden = 1;
    public static int $shared = 1;
    public function __get(string $name): mixed { return null; }
    public function __set(string $name, mixed $value): void {}
    public function inside(): int { return $this->hidden; }
}
class Child extends Base {}
/** @property-read float $title */ class Overridden extends Base {}
function inspect(Base $base, Child $child, Overridden $overridden): void {
    $base->title;
    $child->title;
    $overridden->title;
    Base::$shared;
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let inherited = [
        ("title", "string"),
        ("count", "int"),
        ("enabled", "bool"),
        ("visible", "int"),
        ("hidden", "string"),
        ("shared", "string"),
    ];
    for (prefix, expected) in [
        ("$base->", inherited.as_slice()),
        ("$child->", inherited.as_slice()),
        ("$overridden->ti", &[("title", "float")]),
        ("$this->hi", &[("hidden", "int")]),
        ("Base::$", &[("$shared", "int")]),
    ] {
        let offset = source.find(prefix).unwrap() + prefix.len();
        let position = position_at_offset(&file, offset as u32);
        let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
        let properties: Vec<_> = result["items"].as_array().unwrap().iter().filter(|item| item["kind"] == 5).collect();
        assert_eq!(properties.len(), expected.len(), "{prefix}: {result}");
        for (label, detail) in expected {
            let item = properties.iter().find(|item| item["label"] == *label).unwrap();
            assert_eq!(item["detail"], *detail, "{prefix}: {item}");
            assert_eq!(item["textEdit"]["newText"], *label, "{prefix}: {item}");
        }
    }
}

#[tokio::test]
async fn completion_checks_inherited_constant_visibility_at_its_declaration() {
    let base = "<?php class Base { protected const VALUE = 1; private const SECRET = 2; } ";
    for (classes, expected) in [
        ("class Right extends Base {} class Left extends Base { function test() { Right::VAL|; } }", true),
        ("class Right extends Base {} function test() { Right::VAL|; }", false),
        (
            "class Right extends Base { protected const VALUE = 3; } class Left extends Base { function test() { Right::VAL|; } }",
            false,
        ),
    ] {
        let items = complete(&format!("{base}{classes}")).await;
        assert_eq!(labels(&items).contains(&"VALUE"), expected, "{classes}: {items:?}");
    }
    for (import, expected) in [("", true), ("use Values;", false)] {
        let source = format!(
            "<?php trait Values {{ protected const VALUE = 1; }} class Base {{ use Values; }} class Right extends Base {{ {import} }} class Left extends Base {{ function test() {{ Right::VAL|; }} }}"
        );
        let items = complete(&source).await;
        assert_eq!(labels(&items).contains(&"VALUE"), expected, "{source}: {items:?}");
    }
}

#[tokio::test]
async fn static_property_completion_replaces_one_dollar_and_full_suffix() {
    let source = "<?php class Box { public static int $property; public static function probe() {} } Box::$pr|operty;";
    let items = complete(source).await;
    assert_eq!(labels(&items), ["$property"]);
    let edit = &items[0]["textEdit"];
    let start = edit["range"]["start"]["character"].as_u64().unwrap() as usize;
    let end = edit["range"]["end"]["character"].as_u64().unwrap() as usize;
    let mut applied = source.replace('|', "");
    applied.replace_range(start..end, edit["newText"].as_str().unwrap());
    assert!(applied.ends_with("Box::$property;"), "{applied}");
}

#[tokio::test]
async fn member_completion_replaces_selectors_starting_at_the_cursor() {
    let declarations = "<?php class Box { public int $field = 0; public static int $property = 0; public function send(string $message, int $priority = 0): void {} public static function create(string $message): void {} } ";
    for (suffix, label) in [
        ("function test(Box $box) { $box->|send(message: 'hello', priority: 2); }", "send"),
        ("function test(Box $box) { $box?->|send /* keep */ ('hello'); }", "send"),
        ("function test(Box $box) { $box->|send(...); }", "send"),
        ("function test(Box $box) { $box->|field; }", "field"),
        ("Box::|create('hello');", "create"),
        ("Box::|$property;", "$property"),
    ] {
        let source = format!("{declarations}{suffix}");
        let items = complete(&source).await;
        let item = items.iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("{source}: {items:?}"));
        assert_eq!(item["textEdit"]["newText"], label, "{source}: {item}");
        assert_eq!(apply(&source, item), source.replace('|', ""), "{source}");
    }
}

#[tokio::test]
async fn locals_stay_in_scope_and_do_not_leak_from_nested_functions() {
    let cases = [
        (
            "<?php $global = 1; function first($other) {} function second($local) { $| }",
            vec!["$local"],
            vec!["$global", "$other"],
        ),
        ("<?php function outer($local) { $fn = function($inner) {}; $| }", vec!["$local", "$fn"], vec!["$inner"]),
        (
            "<?php function outer($captured, $hidden) { $fn = function($local) use ($captured) { $| }; }",
            vec!["$local", "$captured"],
            vec!["$hidden"],
        ),
        ("<?php function outer($captured) { $fn = fn($local) => $|; }", vec!["$local", "$captured"], vec![]),
        ("<?php function outer($captured) { $fn = fn(int $local): int => $|; }", vec!["$local", "$captured"], vec![]),
        ("<?php function outer($local) { $fn = fn($hidden) => $hidden; $| }", vec!["$local", "$fn"], vec!["$hidden"]),
    ];
    for (source, allowed, denied) in cases {
        let items = complete(source).await;
        let labels = labels(&items);
        for name in allowed {
            assert!(labels.contains(&name), "{source}: {labels:?}");
        }
        for name in denied {
            assert!(!labels.contains(&name), "{source}: {labels:?}");
        }
    }
}

#[tokio::test]
async fn typed_shadowing_and_scalar_receivers_never_reuse_an_outer_type() {
    for body in ["$box = 1; $box->|", "$box->missing()->|", "(1)->|", "$fn = function(int $box) { $box->| }; "] {
        let items = complete(&format!("{TYPES} function test(Box $box) {{ {body} }}")).await;
        assert!(items.is_empty(), "{body}: {items:?}");
    }
}

fn apply(marked: &str, item: &Value) -> String {
    let mut source = marked.to_owned();
    source.remove(source.rfind('|').unwrap());
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let range: tower_lsp_server::gen_lsp_types::Range =
        serde_json::from_value(item["textEdit"]["range"].clone()).unwrap();
    source.replace_range(
        offset_at_position(&file, range.start) as usize..offset_at_position(&file, range.end) as usize,
        item["textEdit"]["newText"].as_str().unwrap(),
    );
    source
}

#[tokio::test]
async fn variable_completion_replaces_the_whole_token() {
    for (source, label) in [
        ("<?php function run(object $command): void { $co|mmand; }", "$command"),
        ("<?php function run(string $title): void { echo \"$ti|tle\"; }", "$title"),
        ("<?php /* 😀 */ function run(string $名前): void { echo $名|前; }", "$名前"),
        ("<?php function run(string $title) { return fn () => $ti|tle; }", "$title"),
        ("<?php echo $_GE|T;", "$_GET"),
        ("<?php class C { function run() { return $th|is; } }", "$this"),
    ] {
        let items = complete(source).await;
        let matches: Vec<_> = items.iter().filter(|item| item["label"] == label).collect();
        assert_eq!(matches.len(), 1, "{source}: {items:?}");
        assert_eq!(apply(source, matches[0]), source.replace('|', ""), "{source}");
    }
}

#[tokio::test]
async fn variable_completion_offers_superglobals_without_prior_uses() {
    let globals = ["$GLOBALS", "$_SERVER", "$_GET", "$_POST", "$_FILES", "$_COOKIE", "$_SESSION", "$_REQUEST", "$_ENV"];
    for source in [
        "<?php $|;",
        "<?php function run() { $|; }",
        "<?php class C { static function run() { $|; } }",
        "<?php $run = static fn () => $|;",
    ] {
        let items = complete(source).await;
        for name in globals {
            assert_eq!(items.iter().filter(|item| item["label"] == name).count(), 1, "{source}: {items:?}");
        }
        assert!(!labels(&items).contains(&"$this"), "{source}: {items:?}");
    }
    assert_eq!(labels(&complete("<?php $_GE|;").await), ["$_GET"]);
    assert_eq!(labels(&complete("<?php $GLO|;").await), ["$GLOBALS"]);
    assert!(complete("<?php $_ge|;").await.is_empty());
}

#[tokio::test]
async fn variable_completion_keeps_this_in_its_instance_scope() {
    for (source, expected) in [
        ("<?php class C { function run() { $th|; } }", true),
        ("<?php class C { function run() { $f = function () { $th|; }; } }", true),
        ("<?php class C { function run() { $f = fn () => $th|; } }", true),
        ("<?php class C { function run() { $f = fn () => $|; } }", true),
        ("<?php class C { function run() { $f = static fn () => $|; } }", false),
        ("<?php class C { function run() { $f = static fn () => $th|; } }", false),
        ("<?php class C { function run() { $f = static function () { $th|; }; } }", false),
        ("<?php class C { function run() { function nested() { $th|; } } }", false),
        ("<?php class C { static function run() { $th|; } }", false),
        ("<?php class C { static function run() { $f = fn () => $th|; } }", false),
        ("<?php $object = new class { function run() { $th|; } };", true),
        ("<?php class C { public int $value { get { $th|; return 1; } } }", true),
        ("<?php $f = function () { $th|; };", false),
        ("<?php $th|;", false),
    ] {
        let items = complete(source).await;
        assert_eq!(labels(&items).contains(&"$this"), expected, "{source}: {items:?}");
    }
}

const NAMESPACED: &str = "<?php namespace Lib { class Widget {} function work() {} const FLAG = 1; } namespace App\\Sub { class Widget {} } ";

#[tokio::test]
async fn completion_uses_class_function_and_constant_aliases() {
    for (tail, label, replacement, kind) in
        [("new Par|;", "Parcel", "Parcel", 7), ("ac|;", "act", "act($1)", 3), ("echo Mar|;", "Mark", "Mark", 21)]
    {
        let source = format!(
            "{NAMESPACED} namespace App {{ use Lib\\{{Widget as Parcel, function work as act, const FLAG as Mark}}; {tail} }}"
        );
        let items = complete(&source).await;
        let matches: Vec<_> = items.iter().filter(|item| item["label"] == label).collect();
        assert_eq!(matches.len(), 1, "{items:?}");
        assert_eq!(matches[0]["textEdit"]["newText"], replacement);
        assert_eq!(matches[0]["kind"], kind);
        assert!(matches[0]["additionalTextEdits"].is_null());
    }
    let source =
        format!("{NAMESPACED} namespace One {{ use Lib\\Widget as HiddenAlias; }} namespace Two {{ new HiddenA|; }}");
    assert!(!labels(&complete(&source).await).contains(&"HiddenAlias"));
    let source = format!("{NAMESPACED} namespace App {{ use Lib\\Widget; new Wid|; }}");
    assert_eq!(
        complete(&source)
            .await
            .iter()
            .filter(|item| item["label"] == "Widget" && item["documentation"]["value"] == "Lib\\Widget")
            .count(),
        1
    );
    let items =
        complete("<?php namespace Lib; class Widget {} namespace App; use Lib\\Widget as Parcel; new Par|").await;
    assert!(labels(&items).contains(&"Parcel"), "{items:?}");
    let source = format!("{NAMESPACED} namespace App {{ use const Lib\\FLAG as Flag; echo FL|; }}");
    let items = complete(&source).await;
    assert_eq!(items.iter().find(|item| item["label"] == "Flag").unwrap()["textEdit"]["newText"], "Flag");
    assert!(
        items.iter().find(|item| item["label"] == "FLAG").unwrap()["textEdit"]["newText"]
            .as_str()
            .unwrap()
            .starts_with('\\')
    );
}

#[tokio::test]
async fn qualified_completion_preserves_alias_root_and_relative_prefixes() {
    for (prefix, target) in [
        ("Vendor", "Lib\\Widget"),
        ("\\Lib", "Lib\\Widget"),
        ("Sub", "App\\Sub\\Widget"),
        ("namespace\\Sub", "App\\Sub\\Widget"),
    ] {
        let source = format!("{NAMESPACED} namespace App {{ use Lib as Vendor; new {prefix}\\Wi|dget(); }}");
        let items = complete(&source).await;
        let item = items.iter().find(|item| item["label"] == "Widget").unwrap();
        assert_eq!(item["documentation"]["value"], target);
        assert!(apply(&source, item).ends_with(&format!("new {prefix}\\Widget(); }}")));
        assert!(!labels(&items).contains(&"work"));
    }
    let source = format!("{NAMESPACED} namespace App {{ use Lib as Vendor; Vendor\\wo|rk(); }}");
    let items = complete(&source).await;
    let item = items.iter().find(|item| item["label"] == "work").unwrap();
    assert!(apply(&source, item).ends_with("Vendor\\work(); }"));
    let source = format!("{NAMESPACED} namespace App {{ echo \\Lib\\FL|; }}");
    let items = complete(&source).await;
    assert_eq!(items.iter().find(|item| item["label"] == "FLAG").unwrap()["textEdit"]["newText"], "FLAG");
}

#[tokio::test]
async fn bare_insertions_qualify_shadowed_functions_and_constants() {
    let declarations =
        "<?php namespace { function work() {} const FLAG = 1; } namespace App { function work() {} const FLAG = 2; ";
    for (tail, label, qualified, local) in
        [("wo|rk(); }", "work", "\\work", "work"), ("echo FL|; }", "FLAG", "\\FLAG", "FLAG")]
    {
        let source = format!("{declarations}{tail}");
        let items = complete(&source).await;
        let matches: Vec<_> = items
            .iter()
            .filter(|item| item["label"] == label)
            .map(|item| item["textEdit"]["newText"].as_str().unwrap())
            .collect();
        assert!(matches.contains(&qualified) && matches.contains(&local), "{items:?}");
    }
    let source = format!("{NAMESPACED} namespace App {{ wo|rk(); }}");
    let items = complete(&source).await;
    let item = items.iter().find(|item| item["label"] == "work").unwrap();
    assert!(apply(&source, item).ends_with("\\Lib\\work(); }"));
}

#[tokio::test]
async fn member_completion_keeps_distinct_kinds_and_deduplicates_union_methods() {
    for hint in ["A|B", "A&B"] {
        let source = format!(
            "<?php interface A {{ public function Same(): void; }} interface B {{ public function same(): void; }} function test({hint} $value) {{ $value->sa|; }}"
        );
        let items = complete(&source).await;
        assert_eq!(
            items.iter().filter(|item| item["label"].as_str().unwrap().eq_ignore_ascii_case("same")).count(),
            1,
            "{items:?}"
        );
    }
    let items = complete(
        "<?php class A { public int $same; public function Same(): void {} } function test(A $value) { $value->sa|; }",
    )
    .await;
    assert!(items.iter().any(|item| item["kind"] == 5) && items.iter().any(|item| item["kind"] == 2), "{items:?}");
}
