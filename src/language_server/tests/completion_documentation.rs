use serde_json::{Value, json};

use mago_database::file::File;

use crate::language_server::ServerConfig;
use crate::language_server::position::position_at_offset;

use super::harness::Harness;

async fn complete(h: &mut Harness, source: &str, prefix: &str, label: &str) -> Value {
    let result = completion_items(h, source, prefix).await;
    result["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == label)
        .unwrap_or_else(|| panic!("missing {label}: {result}"))
        .clone()
}

async fn completion_items(h: &mut Harness, source: &str, prefix: &str) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let offset = source.rfind(prefix).unwrap() + prefix.len();
    let position = position_at_offset(&file, offset as u32);
    h.at("textDocument/completion", "a.php", position.line, position.character).await
}

#[tokio::test]
async fn completion_reads_builtin_function_method_and_class_docs() {
    let source = "<?php strlen('x'); DateTimeImmutable::createFromFormat('Y', '2026'); new ArrayObject();";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{"completion":{"completionItem":{"documentationFormat":["markdown"]}}}}),
    )
    .await;
    for (prefix, label, expected) in [
        ("str", "strlen", "**Returns** `int<0, max>`"),
        ("DateTimeImmutable::createFromF", "createFromFormat", "@psalm-mutation-free"),
        ("new ArrayObj", "ArrayObject", "@template K of array-key"),
    ] {
        let item = complete(&mut h, source, prefix, label).await;
        assert_eq!(item["documentation"]["kind"], "markdown", "{item}");
        assert!(item["documentation"]["value"].as_str().unwrap().contains(expected), "{item}");
    }
}

#[tokio::test]
async fn completion_uses_declaration_docs_and_its_own_markup_preference() {
    let declarations = r"<?php
namespace Lib;
/** A **widget**. */
#[\AllowDynamicProperties]
class Widget {
    /** Reads a value.
     * @param string $value The **input**.
     */
    public function read(string $value): string { return $value; }
}
trait Render {
    /** Renders a value. */
    public function render(string $value): string { return $value; }
}
class Child extends Widget { use Render { render as renamed; } }
/** Describes **values**.
 * @param string $value The **input**.
 * @return string The result.
 */
function describe(string $value): string { return $value; }
/** Enum docs must not attach to generated methods. */
enum State { case Ready; }
";
    let source = r"<?php
namespace App;
use Lib\Widget as Alias;
use function Lib\describe as describeAlias;
describeAlias('x');
new Alias();
\Lib\describe('x');
new \Lib\Widget();
function test(\Lib\Child $child) { $child->read('x'); $child->renamed('x'); }
\Lib\State::cases();
";
    for (formats, kind, parameter) in [
        (json!(["markdown", "plaintext"]), "markdown", "**Parameter** `$value` `string` The **input**."),
        (json!(["plaintext", "markdown"]), "plaintext", "Parameter $value string The **input**."),
        (json!([]), "plaintext", "Parameter $value string The **input**."),
    ] {
        let mut h = Harness::start_with(
            &[("a.php", source), ("declarations.php", declarations)],
            ServerConfig::default(),
            json!({"textDocument":{
                "hover":{"contentFormat":["markdown"]},
                "completion":{"completionItem":{"documentationFormat":formats}}
            }}),
        )
        .await;
        for (prefix, label, expected) in [
            ("describeAl", "describeAlias", "Describes **values**."),
            ("new Al", "Alias", "A **widget**."),
            (r"\Lib\descr", "describe", parameter),
            (r"new \Lib\Wid", "Widget", "A **widget**."),
            ("$child->re", "renamed", "Renders a value."),
            ("$child->rea", "read", parameter),
        ] {
            let item = complete(&mut h, source, prefix, label).await;
            assert_eq!(item["documentation"]["kind"], kind, "{item}");
            assert!(item["documentation"]["value"].as_str().unwrap().contains(expected), "{item}");
        }
        let generated = complete(&mut h, source, "State::ca", "cases").await;
        assert!(generated["documentation"].is_null(), "{generated}");

        h.open("declarations.php", declarations).await;
        let updated = declarations.replace("Describes **values**.", "Updated **values**.");
        h.change("declarations.php", &updated, 2).await;
        let item = complete(&mut h, source, "describeAl", "describeAlias").await;
        let document = item["documentation"]["value"].as_str().unwrap();
        assert!(document.contains("Updated **values**."), "{item}");
        assert!(!document.contains("Describes **values**."), "{item}");
    }
}

#[tokio::test]
async fn completion_reads_property_constant_and_enum_case_docs() {
    let declarations = r"<?php
namespace Lib;
/** The **limit**. */ const LIMIT = 1, BACKUP = 2;
class Base {
    /** The **title**. */ public string $title = '', $subtitle = '';
    /** The **total**. */ public static int $total = 0, $extra = 0;
    /** Retry **count**. */ public const RETRIES = 3, ATTEMPTS = 4;
}
class Child extends Base {}
enum State { /** Ready **now**. */ case Ready; }
";
    let source = r"<?php
namespace App;
use const Lib\LIMIT as MaxValue;
use const Lib\BACKUP as Reserve;
MaxValue;
\Lib\BACKUP;
LIMIT;
\Lib\Child::RETRIES;
\Lib\Child::ATTEMPTS;
\Lib\Child::$total;
\Lib\Child::$extra;
\Lib\State::Ready;
function show(\Lib\Child $item): void { echo $item->title, $item->subtitle; }
";
    let mut h = Harness::start_with(
        &[("a.php", source), ("declarations.php", declarations)],
        ServerConfig::default(),
        json!({"textDocument":{"completion":{"completionItem":{"documentationFormat":["markdown"]}}}}),
    )
    .await;
    for (prefix, label, expected) in [
        ("MaxV", "MaxValue", "The **limit**."),
        (r"\Lib\BA", "BACKUP", "The **limit**."),
        ("LIM", "LIMIT", "The **limit**."),
        (r"use const Lib\BA", "BACKUP", "The **limit**."),
        ("Child::RE", "RETRIES", "Retry **count**."),
        ("Child::AT", "ATTEMPTS", "Retry **count**."),
        ("Child::$to", "$total", "The **total**."),
        ("Child::$ex", "$extra", "The **total**."),
        ("State::Re", "Ready", "Ready **now**."),
        ("$item->ti", "title", "The **title**."),
        ("$item->su", "subtitle", "The **title**."),
    ] {
        let item = complete(&mut h, source, prefix, label).await;
        assert_eq!(item["documentation"], json!({"kind":"markdown", "value":expected}), "{item}");
    }
}

#[tokio::test]
async fn named_argument_completion_reads_the_bound_parameter_description() {
    let declarations = r"<?php
namespace Lib;
/** @param int $value Requested **count**. */ function take(int $value): void {}
class Base {
    /** @param int $value Constructor **count**. */ public function __construct(public int $value) {}
    /** @param int $value Method **count**. */ public function read(int $value): void {}
}
class Child extends Base {}
#[\Attribute] class Mark extends Base {}
class Other { /** @param int $value Other **count**. */ public function read(int $value): void {} }
";
    let mut h = Harness::start_with(
        &[("a.php", "<?php"), ("declarations.php", declarations)],
        ServerConfig::default(),
        json!({"textDocument":{"completion":{"completionItem":{"documentationFormat":["markdown"]}}}}),
    )
    .await;
    h.open("a.php", "<?php").await;
    for (index, (source, prefix, expected)) in [
        (r"<?php \Lib\take(va);", "take(va", Some("Requested **count**.")),
        (r"<?php \Lib\take();", "take(", Some("Requested **count**.")),
        (r"<?php \Lib\take(value: 1);", "take(va", Some("Requested **count**.")),
        (r"<?php $partial = \Lib\take(value: ?);", "take(va", Some("Requested **count**.")),
        (r"<?php $partial = \Lib\take(value: ?); $partial(value: 1);", "$partial(va", Some("Requested **count**.")),
        (r"<?php new \Lib\Child(value: 1);", "Child(va", Some("Constructor **count**.")),
        (r"<?php #[\Lib\Mark(value: 1)] class Tagged {}", "Mark(va", Some("Constructor **count**.")),
        (
            r"<?php function run(\Lib\Child $item): void { $item->read(value: 1); }",
            "read(va",
            Some("Method **count**."),
        ),
        (r"<?php function run(\Lib\Base|\Lib\Other $item): void { $item->read(value: 1); }", "read(va", None),
    ]
    .into_iter()
    .enumerate()
    {
        h.change("a.php", source, index as i32 + 2).await;
        let item = complete(&mut h, source, prefix, "value:").await;
        if let Some(expected) = expected {
            assert_eq!(item["documentation"], json!({"kind":"markdown", "value":expected}), "{source}: {item}");
        } else {
            let text = item["documentation"]["value"].as_str().unwrap();
            for expected in
                [r"Lib\Base::read", r"Lib\Other::read", "int $value", "Method **count**.", "Other **count**."]
            {
                assert!(text.contains(expected), "{source}: {item}");
            }
        }
    }
}

#[tokio::test]
async fn union_member_completion_keeps_each_declaration_and_its_docs() {
    let source = r"<?php
namespace Domain;
/** @property-read string $value */
class AMagic {
    /** Sends an integer.
     * @param int $count The **count**.
     */
    public function send(int $count = 1): int { return $count; }
}
class ZNative {
    /** Native **value**. */ public int $value = 1;
    /** Sends text.
     * @param string $text The **text**.
     */
    public function send(string $text = 'x'): string { return $text; }
}
class InheritedA extends ZNative {}
class InheritedB extends ZNative {}
function inspect(AMagic|ZNative $record, InheritedA|InheritedB $inherited): void {
    echo $record->value;
    $record->send();
    echo $inherited->value;
    $inherited->send();
}
";
    for format in ["markdown", "plaintext"] {
        let mut h = Harness::start_with(
            &[("a.php", source)],
            ServerConfig::default(),
            json!({"textDocument":{"completion":{"completionItem":{
                "documentationFormat":[format], "snippetSupport":true
            }}}}),
        )
        .await;
        for (prefix, label, declarations, descriptions) in [
            ("$record->va", "value", ["string $value", "int $value"], vec!["Native **value**."]),
            (
                "$record->se",
                "send",
                ["send(int $count = 1): int", "send(string $text = 'x'): string"],
                vec!["Sends an integer.", "The **count**.", "Sends text.", "The **text**."],
            ),
        ] {
            let response = completion_items(&mut h, source, prefix).await;
            let matching: Vec<_> =
                response["items"].as_array().unwrap().iter().filter(|item| item["label"] == label).collect();
            assert_eq!(matching.len(), 1, "{response}");
            let item = matching[0];
            assert_eq!(item["textEdit"]["newText"], label, "{item}");
            assert_eq!(item["documentation"]["kind"], format, "{item}");
            let detail = item["detail"].as_str().unwrap();
            let document = item["documentation"]["value"].as_str().unwrap();
            for (owner, declaration) in [r"Domain\AMagic", r"Domain\ZNative"].into_iter().zip(declarations) {
                assert!(detail.contains(owner) && detail.contains(declaration), "{item}");
                assert!(document.contains(owner) && document.contains(declaration), "{item}");
            }
            for description in descriptions {
                assert!(document.contains(description), "{item}");
            }
            assert_eq!(document.contains("```php\n"), format == "markdown", "{item}");
        }
        // Both receivers inherit the same declaration; keep its original concise detail and docs.
        let field = complete(&mut h, source, "$inherited->va", "value").await;
        assert_eq!(field["detail"], "int", "{field}");
        assert_eq!(field["documentation"], json!({"kind":format,"value":"Native **value**."}));
        let method = complete(&mut h, source, "$inherited->se", "send").await;
        assert_eq!(method["detail"], "send(string $text = 'x'): string", "{method}");
        let document = method["documentation"]["value"].as_str().unwrap();
        assert_eq!(document.matches("Sends text.").count(), 1, "{method}");
        assert!(!document.contains("Domain\\ZNative"), "{method}");
    }
}

#[tokio::test]
async fn magic_property_docs_follow_the_declaring_class_in_completion_and_hover() {
    let source = r"<?php
/**
 * @property-read string $title Base **title**.
 * @psalm-property-read int $count Psalm **count**.
 * @phpstan-property-write string $input PHPStan **input**.
 * @property string $native Magic description must not replace native docs.
 */
class Base {
    /** Native **field**. */ public int $native = 1;
    /** @param string $promoted Promoted **field**. */
    public function __construct(public string $promoted = '') {}
    public function __get(string $name): mixed { return null; }
    public function __set(string $name, mixed $value): void {}
}
class Child extends Base {}
/** @property-read string $title Child **title**. */
class Overridden extends Base {}
function inspect(Base $base, Child $child, Overridden $overridden): void {
    echo $base->title, $child->title, $overridden->title, $child->count, $child->native, $child->promoted;
    $child->input = 'value';
}
";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{
            "completion":{"completionItem":{"documentationFormat":["markdown"]}},
            "hover":{"contentFormat":["markdown"]}
        }}),
    )
    .await;
    for (prefix, label, expected) in [
        ("$base->ti", "title", "Base **title**."),
        ("$child->ti", "title", "Base **title**."),
        ("$overridden->ti", "title", "Child **title**."),
        ("$child->co", "count", "Psalm **count**."),
        ("$child->in", "input", "PHPStan **input**."),
        ("$child->na", "native", "Native **field**."),
        ("$child->pr", "promoted", "Promoted **field**."),
    ] {
        let item = complete(&mut h, source, prefix, label).await;
        assert_eq!(item["documentation"], json!({"kind":"markdown", "value":expected}), "{item}");
        let position = position_at_offset(&file, (source.find(prefix).unwrap() + prefix.len()) as u32);
        let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
        let text = hover["contents"]["value"].as_str().unwrap();
        assert!(text.contains(expected), "{prefix}: {hover}");
        assert!(!text.contains("Magic description"), "{prefix}: {hover}");
    }
}

#[tokio::test]
async fn callable_docs_distinguish_native_magic_and_generated_methods() {
    let source = r"<?php
/** Class overview must not become method docs.
 * @method string greet(string $name) Greets **one** person.
 * @method static string build() Builds **one** record.
 */
class Greeter {
    public function __call(string $name, array $args): mixed { return ''; }
    public static function __callStatic(string $name, array $args): mixed { return ''; }
    /** Native **greeting**. */ public function native(string $name): string { return $name; }
}
class Child extends Greeter {}
/** @method string greet(string $name) Child **greeting**. */
class Overridden extends Greeter {}
/** @method string greet(string $name) Ignored magic description. */
class Native extends Greeter {
    /** Actual **greeting**. */
    #[\Deprecated]
    public function greet(mixed $name): mixed { return $name; }
}
class NativeChild extends Native {}
/** Class overview must not become method docs.
 * @method string greet(string $name) Ignored magic description.
 */
class Undocumented extends Greeter { public function greet(mixed $name): mixed { return $name; } }
/** Describes status values. */
enum Status: int { case Ready = 1; }
/** A **helper**. */ function helper(string $name): string { return $name; }
function inspect(Greeter $base, Child $child, Overridden $overridden, Native $native, NativeChild $nativeChild, Undocumented $undocumented): void {
    $base->greet('A');
    $child->greet('A');
    $overridden->greet('A');
    $native->greet('A');
    $nativeChild->greet('A');
    $undocumented->greet('A');
    $child->native('A');
    Child::build();
    Status::from(1);
    Status::cases();
    helper('A');
    strlen('A');
}
";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{
            "completion":{"completionItem":{"documentationFormat":["markdown"]}},
            "hover":{"contentFormat":["markdown"]},
            "signatureHelp":{"signatureInformation":{"documentationFormat":["markdown"]}}
        }}),
    )
    .await;
    for (call, label, expected) in [
        ("$base->greet(", "greet", Some("Greets **one** person.")),
        ("$child->greet(", "greet", Some("Greets **one** person.")),
        ("$overridden->greet(", "greet", Some("Child **greeting**.")),
        ("$native->greet(", "greet", Some("Actual **greeting**.")),
        ("$nativeChild->greet(", "greet", Some("Actual **greeting**.")),
        ("$undocumented->greet(", "greet", None),
        ("$child->native(", "native", Some("Native **greeting**.")),
        ("Child::build(", "build", Some("Builds **one** record.")),
        ("Status::from(", "from", None),
        ("Status::cases(", "cases", None),
        ("helper(", "helper", Some("A **helper**.")),
        ("strlen(", "strlen", Some("**Returns** `int<0, max>`")),
    ] {
        let prefix = &call[..call.len() - 1];
        let item = complete(&mut h, source, prefix, label).await;
        let start = source.rfind(call).unwrap();
        let hover_position = position_at_offset(&file, (start + call.len() - 2) as u32);
        let hover = h.at("textDocument/hover", "a.php", hover_position.line, hover_position.character).await;
        let hover_text = hover["contents"]["value"].as_str().unwrap();
        let position = position_at_offset(&file, (start + call.len()) as u32);
        let help = h.at("textDocument/signatureHelp", "a.php", position.line, position.character).await;
        assert_eq!(help["signatures"].as_array().unwrap().len(), 1, "{call}: {help}");
        if matches!(call, "$native->greet(" | "$nativeChild->greet(" | "$undocumented->greet(") {
            // The class tag still supplies the type contract; only docs come from the native declaration.
            for signature in
                [item["detail"].as_str().unwrap(), help["signatures"][0]["label"].as_str().unwrap(), hover_text]
            {
                assert!(signature.contains("greet(string $name): string"), "{call}: {signature}");
            }
        }
        for document in [&item["documentation"], &help["signatures"][0]["documentation"]] {
            if let Some(expected) = expected {
                assert!(document["value"].as_str().is_some_and(|text| text.contains(expected)), "{call}: {document}");
                assert!(hover_text.contains(expected), "{call}: {hover}");
            } else {
                assert!(document.is_null(), "{call}: {document}");
            }
        }
        for unrelated in ["Class overview", "Describes status values", "Ignored magic description"] {
            assert!(!hover_text.contains(unrelated), "{call}: {hover}");
        }
    }
}

#[tokio::test]
async fn overridden_callable_parameters_read_exact_native_docs() {
    let declarations = r"<?php
/** Class prose must not become parameter docs.
 * @method string greet(string $name)
 * @method string farewell(string $name)
 * @method string mismatch(string $name)
 * @method string undocumented(string $name)
 */
class Native {
    /** @param mixed $name First **name**. */
    public function greet(mixed $name): mixed { return $name; }
    /** @param mixed $name Last **name**. */
    public function farewell(mixed $name): mixed { return $name; }
    /** @param mixed $other Wrong parameter. */
    public function mismatch(mixed $other): mixed { return $other; }
    public function undocumented(mixed $name): mixed { return $name; }
}
class Child extends Native {}
";
    let source = r"<?php
function inspect(Native $native, Child $child): void {
    $native->greet(name: 'a');
    $native->farewell(name: 'b');
    $child->greet(name: 'c');
    $full = $child->greet(...);
    $full(name: 'd');
    $partial = $child->farewell(name: ?);
    $partial(name: 'e');
    $native->mismatch(name: 'f');
    $native->undocumented(name: 'g');
}
";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut h = Harness::start_with(
        &[("a.php", source), ("declarations.php", declarations)],
        ServerConfig::default(),
        json!({"textDocument":{
            "completion":{"completionItem":{"documentationFormat":["markdown"]}},
            "hover":{"contentFormat":["markdown"]}
        }}),
    )
    .await;
    // Resolve both native overrides in one completion request.
    let menu = completion_items(&mut h, source, "$native->").await;
    for (name, expected) in [("greet", "First **name**."), ("farewell", "Last **name**.")] {
        let item = menu["items"].as_array().unwrap().iter().find(|item| item["label"] == name).unwrap();
        assert!(item["documentation"]["value"].as_str().unwrap().contains(expected), "{item}");
    }
    for (call, expected) in [
        ("$native->greet(name:", Some("First **name**.")),
        ("$native->farewell(name:", Some("Last **name**.")),
        ("$child->greet(name:", Some("First **name**.")),
        ("$full(name:", Some("First **name**.")),
        ("$child->farewell(name:", Some("Last **name**.")),
        ("$partial(name:", Some("Last **name**.")),
        ("$native->mismatch(name:", None),
        ("$native->undocumented(name:", None),
    ] {
        let item = complete(&mut h, source, call.trim_end_matches(':'), "name:").await;
        assert_eq!(item["detail"], "string $name", "{call}: {item}");
        let position = position_at_offset(&file, (source.find(call).unwrap() + call.len() - 2) as u32);
        let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
        let declaration = "```php\nstring $name\n```";
        let expected_hover =
            expected.map_or_else(|| declaration.to_owned(), |text| format!("{declaration}\n\n---\n\n{text}"));
        assert_eq!(hover["contents"]["value"], expected_hover, "{call}: {hover}");
        assert_eq!(
            item["documentation"],
            expected.map_or(Value::Null, |text| json!({"kind":"markdown", "value":text})),
            "{call}: {item}",
        );
    }
}
