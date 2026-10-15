use super::harness::Harness;
use crate::language_server::ServerConfig;
use serde_json::json;

#[tokio::test]
async fn hover_on_member_declarations_uses_exact_names_and_signatures() {
    let source = "<?php class Café { public static int $count = 1; protected const LIMIT = 10; public function greet(string $name): string { return $name; } public function __construct(public string $title) {} } enum Status { case Ready; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (name, expected) in [
        ("$count", "Café\n\npublic static int $count"),
        ("LIMIT", "Café\n\nprotected const LIMIT"),
        ("greet", "Café\n\npublic function greet(string $name): string"),
        ("$title", "Café\n\npublic string $title"),
        ("Ready", "Status\n\ncase Ready"),
    ] {
        let byte = source.find(name).unwrap();
        let column = source[..byte].encode_utf16().count() as u32;
        let hover = h.at("textDocument/hover", "a.php", 0, column + 1).await;
        assert!(hover["contents"]["value"].as_str().is_some_and(|text| text.contains(expected)), "{name}: {hover}");
        assert_eq!(hover["range"]["start"]["character"], column);
        assert_eq!(hover["range"]["end"]["character"], column + name.encode_utf16().count() as u32);
    }
    // Synthetic enum methods span the enum body but must not claim its whitespace.
    let byte = source.find("{ case").unwrap() + 1;
    let column = source[..byte].encode_utf16().count() as u32;
    assert!(h.at("textDocument/hover", "a.php", 0, column).await.is_null());
}

#[tokio::test]
async fn markdown_hover_separates_namespace_and_preserves_declared_names() {
    let source = r"<?php
namespace App\Domain;
interface Zoo {}
interface Alpha {}
abstract class ParentType {}
final class Invoice extends ParentType implements Zoo, Alpha {}
readonly class Value {}
interface Child extends Zoo, Alpha {}
";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{"hover":{"contentFormat":["markdown", "plaintext"]}}}),
    )
    .await;
    for (line, name, declaration) in [
        (4, "ParentType", "abstract class ParentType"),
        (5, "Invoice", "final class Invoice extends ParentType implements Alpha, Zoo"),
        (6, "Value", "readonly class Value"),
        (7, "Child", "interface Child extends Alpha, Zoo"),
    ] {
        let column = source.lines().nth(line).unwrap().find(name).unwrap() as u32;
        let hover = h.at("textDocument/hover", "a.php", line as u32, column).await;
        assert_eq!(hover["contents"]["kind"], "markdown");
        assert_eq!(hover["contents"]["value"], format!("`App\\Domain`\n\n```php\n{declaration}\n```"));
        assert_eq!(hover["range"]["start"]["character"], column);
        assert_eq!(hover["range"]["end"]["character"], column + name.len() as u32);
    }
}

#[tokio::test]
async fn hover_respects_plaintext_preference_and_keeps_member_context() {
    let source = "<?php\nnamespace App;\nclass Item {}\nclass Service { final public static function Find(Item $item): Item { return $item; } }\nService::find(new Item());\n";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{"hover":{"contentFormat":["plaintext", "markdown"]}}}),
    )
    .await;
    let hover = h.at("textDocument/hover", "a.php", 4, 11).await;
    assert_eq!(hover["contents"]["kind"], "plaintext");
    assert_eq!(hover["contents"]["value"], "App\\Service\n\nfinal public static function Find(Item $item): Item");
}

async fn hover_name(h: &mut Harness, source: &str, name: &str) -> serde_json::Value {
    let offset = source.find(name).unwrap();
    let prefix = &source[..offset];
    h.at(
        "textDocument/hover",
        "a.php",
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
    .await
}

#[tokio::test]
async fn hover_docs_keep_paragraphs_tags_and_the_declaring_source() {
    let source = r"<?php
namespace App;
/** Handles **invoices**.
 *
 * Keep each invoice's history.
 */
#[\AllowDynamicProperties]
class ParentType {
    /** The current invoice. */
    #[Unknown([1, 2])]
    public ?string $invoice;
    /** Sends an invoice.
     *
     * @param string $name The name.
     * @return string The result.
     */
    public function send(string $name = 'invoice'): string { return $name; }
    /** Retry count. */
    public const RETRIES = 3;
}
class Child extends ParentType {}
(new Child())->send();
/** The state. */
enum State { /** Ready to send. */ case Ready; }
/** A helper. */
function helper(int $count = 1 + 2): int { return $count; }
/** The limit. */
const LIMIT = 3;
";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{"hover":{"contentFormat":["markdown"]}}}),
    )
    .await;
    for (name, docs) in [
        ("ParentType {", "Handles **invoices**.\n\nKeep each invoice's history."),
        ("$invoice;", "The current invoice."),
        ("RETRIES =", "Retry count."),
        ("Ready;", "Ready to send."),
        ("helper(int", "A helper."),
        ("LIMIT =", "The limit."),
    ] {
        let hover = hover_name(&mut h, source, name).await;
        assert!(hover["contents"]["value"].as_str().unwrap().ends_with(docs), "{name}: {hover}");
    }
    let declaration = hover_name(&mut h, source, "send(string").await;
    let call = hover_name(&mut h, source, "send();").await;
    assert_eq!(declaration["contents"], call["contents"]);
    assert_eq!(
        call["contents"]["value"],
        "`App\\ParentType`\n\n```php\npublic function send(string $name = 'invoice'): string\n```\n\n---\n\nSends an invoice.\n\n**Parameter** `$name` `string` The name.\n\n**Returns** `string` The result."
    );
    assert!(
        hover_name(&mut h, source, "helper(int").await["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("function helper(int $count = 1 + 2): int")
    );
}

#[tokio::test]
async fn hover_docs_follow_buffer_changes_without_borrowing_unattached_comments() {
    let source = "<?php\n/** First docs. */\nclass Target {}\n/** Unattached docs. */\nconst OTHER = '/** Fake docs. */';\nclass Bare {}\nstrlen('x');\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    h.open("a.php", source).await;
    assert_eq!(hover_name(&mut h, source, "Target {}").await["contents"]["value"], "class Target\n\nFirst docs.");
    let changed = source.replace("First docs.", "Fresh docs.");
    h.change("a.php", &changed, 2).await;
    assert_eq!(hover_name(&mut h, &changed, "Target {}").await["contents"]["value"], "class Target\n\nFresh docs.");
    assert_eq!(hover_name(&mut h, &changed, "Bare {}").await["contents"]["value"], "class Bare");
    let builtin = hover_name(&mut h, &changed, "strlen(").await;
    assert!(builtin["contents"]["value"].as_str().unwrap().starts_with("function strlen("));
}

#[tokio::test]
async fn hover_grouped_constants_and_properties_share_their_declaration_docs() {
    let source = r"<?php
/** Global values. */
const GLOBAL_FIRST = [1, [2, 3]], GLOBAL_SECOND = [];
class ParentType {
    /** Class values. */
    public const array CLASS_FIRST = [1, [2, 3]], CLASS_SECOND = [];
    /** Property values. */
    public array $first = [1, [2, 3]], $second = [];
    /** Unset values. */
    public int $third, $fourth;
    public int $bare = 0;
}
class Child extends ParentType {}
echo GLOBAL_SECOND;
echo Child::CLASS_SECOND;
$object = new Child();
echo $object->second;
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, docs) in [
        ("GLOBAL_SECOND =", "Global values."),
        ("GLOBAL_SECOND;", "Global values."),
        ("CLASS_SECOND =", "Class values."),
        ("CLASS_SECOND;", "Class values."),
        ("$second =", "Property values."),
        ("second;", "Property values."),
        ("$fourth;", "Unset values."),
    ] {
        let hover = hover_name(&mut h, source, marker).await;
        assert!(hover["contents"]["value"].as_str().unwrap().ends_with(docs), "{marker}: {hover}");
    }
    let hover = hover_name(&mut h, source, "$bare =").await;
    assert_eq!(hover["contents"]["value"], "ParentType\n\npublic int $bare");
}

#[tokio::test]
async fn hover_defaults_cannot_close_the_php_fence() {
    let source = "<?php function sample(string $text = '\n```\nvalue\n```\n'): string { return $text; }";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        ServerConfig::default(),
        json!({"textDocument":{"hover":{"contentFormat":["markdown"]}}}),
    )
    .await;
    let hover = hover_name(&mut h, source, "sample(").await;
    assert_eq!(
        hover["contents"]["value"],
        "````php\nfunction sample(\n    string $text = '\n```\nvalue\n```\n'\n): string\n````"
    );
}

#[tokio::test]
async fn hover_content_property_keeps_its_write_visibility() {
    let source = "<?php class Article { public private(set) string $title = 'Draft'; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let hover = hover_name(&mut h, source, "$title").await;
    assert!(hover["contents"]["value"].as_str().unwrap().contains("private(set)"), "{hover}");
}

#[tokio::test]
async fn hover_content_inherited_constant_keeps_its_declaring_owner() {
    let source = r"<?php
namespace Library { class ParentType { public const LIMIT = 3; } }
namespace App { class Child extends \Library\ParentType {} echo Child::LIMIT; }
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let hover = hover_name(&mut h, source, "LIMIT;").await;
    let text = hover["contents"]["value"].as_str().unwrap();
    assert!(text.contains("Library\\ParentType") && text.contains("const LIMIT"), "{hover}");
    assert!(!text.contains("App\\Child"), "{hover}");
}

#[tokio::test]
async fn hover_content_constants_keep_types_modifiers_and_values() {
    let source = "<?php const RETRIES = 3; class Limits { final public const int MAXIMUM = 10; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (name, parts) in [("RETRIES", vec!["RETRIES", "3"]), ("MAXIMUM", vec!["final", "int", "MAXIMUM", "10"])] {
        let hover = hover_name(&mut h, source, name).await;
        let text = hover["contents"]["value"].as_str().unwrap();
        assert!(parts.iter().all(|part| text.contains(part)), "{hover}");
    }
}

#[tokio::test]
async fn hover_content_enum_keeps_its_backing_type_and_case_value() {
    let source = "<?php enum State: string { case Ready = 'ready'; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let declaration = hover_name(&mut h, source, "State:").await;
    assert!(declaration["contents"]["value"].as_str().unwrap().contains("enum State: string"), "{declaration}");
    let case = hover_name(&mut h, source, "Ready").await;
    assert!(case["contents"]["value"].as_str().unwrap().contains("'ready'"), "{case}");
}

#[tokio::test]
async fn hover_content_class_lists_only_declared_traits() {
    let source =
        "<?php trait Leaf {} trait Wrapper { use Leaf; } class Base { use Wrapper; } class Child extends Base {}";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let base = hover_name(&mut h, source, "Base {").await;
    let text = base["contents"]["value"].as_str().unwrap();
    assert!(text.contains("use Wrapper;") && !text.contains("Leaf"), "{base}");
    let child = hover_name(&mut h, source, "Child extends").await;
    assert!(!child["contents"]["value"].as_str().unwrap().contains("use "), "{child}");
}
#[tokio::test]
async fn hover_content_parameter_keeps_its_declared_type() {
    let source = "<?php function render(string $title): string { return $title; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let hover = hover_name(&mut h, source, "$title)").await;
    let text = hover["contents"]["value"].as_str().unwrap();
    assert!(text.contains("string") && text.contains("$title"), "{hover}");
}

#[tokio::test]
async fn hover_content_this_keeps_its_receiver_type() {
    let source = "<?php class Widget { public function instance(): Widget { return $this; } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let hover = hover_name(&mut h, source, "$this").await;
    assert!(hover["contents"]["value"].as_str().is_some_and(|text| text.contains("Widget")), "{hover}");
}

#[tokio::test]
async fn hover_content_keeps_colliding_type_names_distinct() {
    let source = r"<?php
namespace A { class Result {} }
namespace B { class Result {} }
namespace App {
    use A\Result as First;
    use B\Result as Second;
    function combine(First $left, Second $right): First { return $left; }
    function identity(First $value): First { return $value; }
    class Holder { public First|Second $both; }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let hover = hover_name(&mut h, source, "combine(").await;
    let text = hover["contents"]["value"].as_str().unwrap();
    for part in [r"\A\Result $left", r"\B\Result $right", r"): \A\Result"] {
        assert!(text.contains(part), "{hover}");
    }
    let property = hover_name(&mut h, source, "$both;").await;
    let text = property["contents"]["value"].as_str().unwrap();
    assert!(text.contains(r"\A\Result") && text.contains(r"\B\Result"), "{property}");
    let identity = hover_name(&mut h, source, "identity(").await;
    assert!(
        identity["contents"]["value"].as_str().unwrap().contains("function identity(Result $value): Result"),
        "{identity}"
    );
}

#[tokio::test]
async fn hover_content_keeps_property_modifiers_without_inventing_a_readonly_setter() {
    let source = r"<?php
abstract class Item {
    private readonly string $name;
    final public string $fixed;
    abstract public string $label { get; set; }
    public string $number { final get => 'one'; }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (name, declaration) in [
        ("$name;", "private readonly string $name"),
        ("$fixed;", "final public string $fixed"),
        ("$label {", "abstract public string $label { get; set; }"),
        ("$number {", "public string $number { final get; }"),
    ] {
        let hover = hover_name(&mut h, source, name).await;
        let text = hover["contents"]["value"].as_str().unwrap();
        assert!(text.ends_with(declaration), "{hover}");
    }
}

#[tokio::test]
async fn hover_headers_keep_colliding_parent_interface_and_trait_names_distinct() {
    let source = r"<?php
namespace One { class Item {} interface Contract {} trait Helper {} }
namespace Two { interface Item {} interface Contract {} trait Helper {} }
namespace Three { trait Item {} }
namespace App {
    class Child extends \one\item implements \two\item { use \three\item; }
    interface Both extends \one\contract, \two\contract {}
    class UsesHelpers { use \one\helper, \two\helper; }
    class Item extends \one\item {}
    class Plain extends \one\item {}
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (at, declaration) in [
        ("Child extends", "class Child extends \\One\\Item implements \\Two\\Item {\n    use \\Three\\Item;\n}"),
        ("Both extends", r"interface Both extends \One\Contract, \Two\Contract"),
        ("UsesHelpers {", "class UsesHelpers {\n    use \\One\\Helper, \\Two\\Helper;\n}"),
        ("Item extends", r"class Item extends \One\Item"),
        ("Plain extends", "class Plain extends Item"),
    ] {
        let hover = hover_name(&mut h, source, at).await;
        assert!(hover["contents"]["value"].as_str().unwrap().ends_with(declaration), "{at}: {hover}");
    }
}
