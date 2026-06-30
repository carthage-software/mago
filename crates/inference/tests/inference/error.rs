use mago_inference::InferenceError;

use crate::harness::Test;

#[test]
fn parameter_property_hooks_outside_a_constructor_are_reported_unsupported() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php function f($x { get => 1; }) {}");

    assert!(
        matches!(result, Err(InferenceError::Unsupported { construct: "parameter property hooks", .. })),
        "expected an unsupported error, got {result:?}",
    );
}

#[test]
fn an_unsupported_construct_aborts_inference_rather_than_typing_as_mixed() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php function f($x { get => 1; }) {} $y = 1; $y;");

    assert!(result.is_err(), "inference must not fabricate a type for an unsupported construct");
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
fn a_dynamically_named_variable_variable_write_is_not_an_error() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php $$$a = 1; $b = 2; $b;");

    assert!(result.is_ok(), "a dynamically-named variable-variable write is untracked, not an error: {result:?}");
}

#[test]
fn an_invalid_assignment_target_is_not_an_error() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php class C { const F = 1; } C::F = 2;");

    assert!(result.is_ok(), "an invalid assignment target is never-typed, not an error: {result:?}");
}

#[test]
fn supported_code_infers_without_error() {
    let test = Test::new();
    let result = test.try_infer("<?php", "<?php $a = 1; $a;");

    assert!(result.is_ok(), "expected inference to succeed, got {result:?}");
}
