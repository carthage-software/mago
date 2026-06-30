use mago_inference::InferenceError;

use crate::harness::Test;

#[test]
fn function_declaration_is_reported_unsupported() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php function f(): int { return 1; }");

    assert!(
        matches!(result, Err(InferenceError::Unsupported { construct: "function declarations", .. })),
        "expected an unsupported error, got {result:?}",
    );
}

#[test]
fn method_calls_on_an_unknown_receiver_are_mixed_not_an_error() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php $service->handle();");

    assert!(
        result.is_ok(),
        "method calls are supported; an unknown receiver infers as mixed, not an error: {result:?}"
    );
}

#[test]
fn an_unsupported_construct_aborts_inference_rather_than_typing_as_mixed() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php function f(): int { return 1; } $a = 1; $a;");

    assert!(result.is_err(), "inference must not fabricate a type for an unsupported construct");
}

#[test]
fn supported_code_infers_without_error() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php $a = 1; $a;");

    assert!(result.is_ok(), "expected inference to succeed, got {result:?}");
}
