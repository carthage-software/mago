use mago_database::file::File;
use serde_json::Value;

use super::harness::Harness;
use crate::language_server::position::position_at_offset;

async fn at(h: &mut Harness, source: &str, offset: usize, method: &str) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    h.at(method, "a.php", position.line, position.character).await
}

#[tokio::test]
async fn parameter_rendering_preserves_each_features_names_and_type_style() {
    let parameters = "Item &$名, string $suffix = '😀', array $options = [1, /* keep */ 2], int &...$rest";
    let source = format!(
        "<?php namespace Demo; class Item {{}} \
         function &inspect({parameters}): Item {{ return $名; }} \
         class Service {{ final public static function &inspect({parameters}): Item {{ return $名; }} }} \
         $item = new Item(); $count = 1; inspect($item, 'x', [], $count); \
         Service::inspect($item, 'x', [], $count);"
    );
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let displayed = parameters.replace("array $options", "array<array-key, mixed> $options");
    let hover_parameters = "Item &$名,\n    string $suffix = '😀',\n    array<array-key, mixed> $options = [1, /* keep */ 2],\n    int &...$rest";
    for (call, hover_prefix, signature_name) in [
        ("inspect($item", "function &inspect", "Demo\\inspect"),
        ("Service::inspect($item", "final public static function &inspect", "Demo\\Service::inspect"),
    ] {
        let start = source.find(call).unwrap() + call.find("inspect").unwrap();
        let hover = at(&mut h, &source, start, "textDocument/hover").await;
        assert!(
            hover["contents"]["value"]
                .as_str()
                .unwrap()
                .contains(&format!("{hover_prefix}(\n    {hover_parameters}\n): Item")),
            "{hover}",
        );
        let completion = at(&mut h, &source, start + "inspe".len(), "textDocument/completion").await;
        let item = completion["items"].as_array().unwrap().iter().find(|item| item["label"] == "inspect").unwrap();
        assert_eq!(item["detail"], format!("&inspect({displayed}): Item"));

        let signature = at(&mut h, &source, start + "inspect($item, ".len(), "textDocument/signatureHelp").await;
        let info = &signature["signatures"][0];
        assert_eq!(info["label"], format!("function &{signature_name}({displayed}): Item"));
        assert_eq!(signature["activeParameter"], 1);
        let label: Vec<_> = info["label"].as_str().unwrap().encode_utf16().collect();
        let expected = [
            "Item &$名",
            "string $suffix = '😀'",
            "array<array-key, mixed> $options = [1, /* keep */ 2]",
            "int &...$rest",
        ];
        assert_eq!(info["parameters"].as_array().unwrap().len(), expected.len());
        for (parameter, expected) in info["parameters"].as_array().unwrap().iter().zip(expected) {
            let start = parameter["label"][0].as_u64().unwrap() as usize;
            let end = parameter["label"][1].as_u64().unwrap() as usize;
            assert_eq!(String::from_utf16(&label[start..end]).unwrap(), expected);
        }
    }
}

async fn check_comment_before_default(comment: &str) {
    let source =
        format!("<?php function commented(string $value {comment} = '😀'): string {{ return $value; }} commented();");
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let start = source.rfind("commented()").unwrap();
    let hover = at(&mut h, &source, start, "textDocument/hover").await;
    let completion = at(&mut h, &source, start + "comme".len(), "textDocument/completion").await;
    let signature = at(&mut h, &source, start + "commented(".len(), "textDocument/signatureHelp").await;
    let declaration = "function commented(string $value = '😀'): string";
    assert_eq!(hover["contents"]["value"], declaration, "{comment}");
    let item = completion["items"].as_array().unwrap().iter().find(|item| item["label"] == "commented").unwrap();
    assert_eq!(item["detail"], "commented(string $value = '😀'): string", "{comment}");
    let info = &signature["signatures"][0];
    assert_eq!(info["label"], declaration, "{comment}");
    let label: Vec<_> = info["label"].as_str().unwrap().encode_utf16().collect();
    let start = info["parameters"][0]["label"][0].as_u64().unwrap() as usize;
    let end = info["parameters"][0]["label"][1].as_u64().unwrap() as usize;
    assert_eq!(String::from_utf16(&label[start..end]).unwrap(), "string $value = '😀'");
}

#[tokio::test]
async fn parameter_defaults_skip_block_comments_before_equals() {
    check_comment_before_default("/* note = wrong */").await;
}

#[tokio::test]
async fn parameter_defaults_skip_line_comments_before_equals() {
    check_comment_before_default("// note = wrong\n").await;
}

#[tokio::test]
async fn closure_signatures_hide_internal_names_and_keep_bound_class_types() {
    let source = r"<?php namespace MagoEditorQa;
class ParentValue {}
class ChildValue extends ParentValue {
    function run(): void {
        $closure = /** @param parent $value The parent value. */
            function (parent $value, int $count = 2): parent { return $value; };
        $arrow = /** @param self $value The child value. */
            fn(self $value, int $count = 2): self => $value;
        $closure(count: 3, value: new ParentValue());
        $arrow(count: 4, value: new ChildValue());
    }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (call, class, docs) in
        [("$closure(count: ", "ParentValue", "The parent value."), ("$arrow(count: ", "ChildValue", "The child value.")]
    {
        let offset = source.find(call).unwrap() + call.len();
        let signature = at(&mut h, source, offset, "textDocument/signatureHelp").await;
        let info = &signature["signatures"][0];
        assert_eq!(info["label"], format!("function ({class} $value, int $count = 2): {class}"));
        assert!(info["documentation"]["value"].as_str().unwrap().contains(docs), "{info}");
        assert_eq!(signature["activeParameter"], 1);
        assert_eq!(info["activeParameter"], 1);
        let label: Vec<_> = info["label"].as_str().unwrap().encode_utf16().collect();
        for (parameter, expected) in
            info["parameters"].as_array().unwrap().iter().zip([format!("{class} $value"), "int $count = 2".into()])
        {
            let start = parameter["label"][0].as_u64().unwrap() as usize;
            let end = parameter["label"][1].as_u64().unwrap() as usize;
            assert_eq!(String::from_utf16(&label[start..end]).unwrap(), expected);
        }
    }
}

#[tokio::test]
async fn named_signatures_restore_class_casing_and_qualify_colliding_types() {
    let source = r"<?php
namespace LeftSide { class SharedValue {} }
namespace RightSide { class SharedValue {} }
namespace MagoEditorQa {
    class ChildValue {}
    function take(\magoeditorqa\childvalue $value): \magoeditorqa\childvalue { return $value; }
    class Service {
        public static function send(\magoeditorqa\childvalue $value): \magoeditorqa\childvalue { return $value; }
    }
    class Holder {
        public function __construct(public \magoeditorqa\childvalue $value) {}
    }
    function compare(\leftside\sharedvalue $left, \rightside\sharedvalue $right): \leftside\sharedvalue { return $left; }
    take(value: new ChildValue());
    Service::send(value: new ChildValue());
    new Holder(value: new ChildValue());
    compare(left: new \LeftSide\SharedValue(), right: new \RightSide\SharedValue());
}

";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (call, expected) in [
        ("take(value: ", r"function MagoEditorQa\take(ChildValue $value): ChildValue"),
        ("Service::send(value: ", r"function MagoEditorQa\Service::send(ChildValue $value): ChildValue"),
        (
            "compare(left: ",
            r"function MagoEditorQa\compare(\LeftSide\SharedValue $left, \RightSide\SharedValue $right): \LeftSide\SharedValue",
        ),
    ] {
        let offset = source.find(call).unwrap() + call.len();
        let signature = at(&mut h, source, offset, "textDocument/signatureHelp").await;
        assert_eq!(signature["signatures"][0]["label"], expected, "{signature}");
        assert_eq!(signature["activeParameter"], 0);
    }
    for (call, prefix, label, expected) in [
        ("take(value:", "ta", "take", "take(ChildValue $value): ChildValue"),
        ("Service::send(value:", "Service::se", "send", "send(ChildValue $value): ChildValue"),
        ("take(value:", "take(va", "value:", "ChildValue $value"),
        ("Service::send(value:", "Service::send(va", "value:", "ChildValue $value"),
        ("new Holder(value:", "new Holder(va", "value:", "ChildValue $value"),
        (
            "compare(left:",
            "compa",
            "compare",
            r"compare(\LeftSide\SharedValue $left, \RightSide\SharedValue $right): \LeftSide\SharedValue",
        ),
    ] {
        let offset = source.find(call).unwrap() + prefix.len();
        let completion = at(&mut h, source, offset, "textDocument/completion").await;
        let item = completion["items"].as_array().unwrap().iter().find(|item| item["label"] == label).unwrap();
        assert_eq!(item["detail"], expected, "{call}: {item}");
    }
}

#[tokio::test]
async fn property_completion_restores_declared_class_casing() {
    let source = r"<?php namespace MagoEditorQa;
class ChildValue {}
class Holder {
    public static \magoeditorqa\childvalue $shared;
    public function __construct(public \magoeditorqa\childvalue $value) {}
}
function inspect(Holder $holder): void {
    $holder->value;
    Holder::$shared;
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (prefix, label) in [("$holder->va", "value"), ("Holder::$sha", "$shared")] {
        let offset = source.find(prefix).unwrap() + prefix.len();
        let completion = at(&mut h, source, offset, "textDocument/completion").await;
        let item = completion["items"].as_array().unwrap().iter().find(|item| item["label"] == label).unwrap();
        assert_eq!(item["detail"], "ChildValue", "{prefix}: {item}");
        assert_eq!(item["textEdit"]["newText"], label, "{prefix}: {item}");
    }
}

#[tokio::test]
async fn equal_closure_signatures_keep_distinct_source_docs() {
    let source = r"<?php function run(bool $pick, bool $other): void {
        $first = /** First callback. */ fn(int $value): int => $value;
        $second = /** Second callback. */ fn(int $value): int => $value;
        $third = /** First callback. */ fn(int $value): int => $value;
        $callback = $pick ? $first : ($other ? $second : $third);
        $callback(value: 1);
    }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let offset = source.find("$callback(value: ").unwrap() + "$callback(value: ".len();
    let result = at(&mut h, source, offset, "textDocument/signatureHelp").await;
    let signatures = result["signatures"].as_array().unwrap();
    assert_eq!(signatures.len(), 2, "{result}");
    for signature in signatures {
        assert_eq!(signature["label"], "function (int $value): int");
    }
    for docs in ["First callback.", "Second callback."] {
        assert!(signatures.iter().any(|signature| signature["documentation"]["value"] == docs), "{result}");
    }
}

#[tokio::test]
async fn closure_alternatives_share_type_name_collision_checks() {
    let source = r"<?php
namespace LeftSide { class Item {} }
namespace RightSide { class Item {} }
namespace {
    function run(bool $pick, \LeftSide\Item|\RightSide\Item $value): void {
        $left = function (\LeftSide\Item $value): void {};
        $right = function (\RightSide\Item $value): void {};
        $callback = $pick ? $left : $right;
        $callback(value: $value);
    }
}";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let offset = source.find("$callback(value: ").unwrap() + "$callback(value: ".len();
    let result = at(&mut h, source, offset, "textDocument/signatureHelp").await;
    let labels: Vec<_> =
        result["signatures"].as_array().unwrap().iter().map(|signature| signature["label"].as_str().unwrap()).collect();
    assert_eq!(labels, [r"function (\LeftSide\Item $value): void", r"function (\RightSide\Item $value): void"]);
}
