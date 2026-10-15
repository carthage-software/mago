use super::harness::Harness;

async fn hover(h: &mut Harness, source: &str, marker: &str) -> String {
    let offset = source.find(marker).unwrap();
    let prefix = &source[..offset];
    let result = h
        .at(
            "textDocument/hover",
            "a.php",
            prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
            prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32 + 1,
        )
        .await;
    result["contents"]["value"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn hover_type_spelling_preserves_apostrophes_in_literal_strings() {
    let source = r#"<?php
class User {}
/** @param "a'user" $value */
function sample(string $value): void {}
"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(hover(&mut h, source, "$value):").await, "string('a'user') $value");
    let function = hover(&mut h, source, "sample(").await;
    assert!(function.starts_with("function sample(string('a'user') $value): void"), "{function}");
}

#[tokio::test]
async fn hover_type_spelling_preserves_apostrophes_in_array_keys() {
    let source = r#"<?php
class User {}
/** @param array{"a'user": int} $value */
function sample(array $value): void {}
"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(hover(&mut h, source, "$value):").await, "array{'a'user': int} $value");
    let function = hover(&mut h, source, "sample(").await;
    assert!(function.starts_with("function sample(array{'a'user': int} $value): void"), "{function}");
}

#[tokio::test]
async fn hover_type_spelling_still_shortens_real_object_names() {
    let source = r"<?php
namespace Domain;
class Widget {}
function sample(Widget $value): void {}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(hover(&mut h, source, "$value):").await, "Widget $value");
    assert_eq!(hover(&mut h, source, "sample(").await, "Domain\n\nfunction sample(Widget $value): void");
}

#[tokio::test]
async fn hover_enum_types_use_php_names_in_parameters_returns_and_variables() {
    let source = r"<?php
namespace Domain;
enum State { case Open; }
function sample(State $value): State { return $value; }
$state = State::Open;
echo $state->name;
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(hover(&mut h, source, "$value):").await, "State $value");
    assert_eq!(hover(&mut h, source, "sample(").await, "Domain\n\nfunction sample(State $value): State");
    assert_eq!(hover(&mut h, source, "$state->").await, "State::Open $state");
}
