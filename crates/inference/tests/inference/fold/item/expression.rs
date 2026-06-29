use indoc::indoc;
use mago_hir::ir::argument::PartialArgument;
use mago_hir::ir::item::expression::ItemExpressionKind;

use crate::harness::*;

test_inference! {
    name = closure_attribute_arguments_are_inferred,
    code = "<?php $a = #[Attr(1 + 2)] function () {};",
    expect = |ir| {
        let ExpressionKind::Assignment(assignment) = expression_of(get_last_statement(ir)).kind else {
            panic!("expected an assignment")
        };
        let ExpressionKind::Item(item) = assignment.right.kind else { panic!("expected an item expression") };
        let ItemExpressionKind::Closure(closure) = item.kind else { panic!("expected a closure") };
        let Some(attribute) = closure.attributes.first() else { panic!("expected an attribute") };
        let Some(arguments) = attribute.arguments.as_ref() else { panic!("expected attribute arguments") };
        let Some(PartialArgument::Value(argument)) = arguments.items.first() else { panic!("expected a value argument") };

        assert_eq!(argument.meta.to_string(), "int(3)", "the attribute argument is inferred, not mixed");
    }
}

test_inference! {
    name = closure_body_is_inferred_with_parameters_bound,
    code = "<?php $a = fn(string $x) => $x;",
    expect = |ir| {
        let ExpressionKind::Assignment(assignment) = expression_of(get_last_statement(ir)).kind else {
            panic!("expected an assignment")
        };
        let ExpressionKind::Item(item) = assignment.right.kind else { panic!("expected an item expression") };
        let ItemExpressionKind::ArrowFunction(arrow) = item.kind else { panic!("expected an arrow function") };
        assert_eq!(arrow.expression.meta.to_string(), "string", "the body reads the bound parameter type");
    }
}

test_inference! {
    name = parameter_default_value_is_inferred,
    code = "<?php $a = fn($x = 1 + 2) => $x;",
    expect = |ir| {
        let ExpressionKind::Assignment(assignment) = expression_of(get_last_statement(ir)).kind else {
            panic!("expected an assignment")
        };
        let ExpressionKind::Item(item) = assignment.right.kind else { panic!("expected an item expression") };
        let ItemExpressionKind::ArrowFunction(arrow) = item.kind else { panic!("expected an arrow function") };
        let Some(parameter) = arrow.parameters.items.first() else { panic!("expected a parameter") };
        let Some(default) = parameter.default_value else { panic!("expected a default value") };
        assert_eq!(default.meta.to_string(), "int(3)", "the default value expression is inferred");
    }
}

test_inference! {
    name = closure_value_carries_its_signature,
    cases = {
        "<?php $a = fn(string $x): int => (int) $x; $a;" => "(closure(string): int)",
        "<?php function () {};" => "(closure(): null)",
    }
}

test_inference! {
    name = calls_a_closure_value,
    cases = { "<?php $a = fn(string $x): int => (int) $x; $b = $a('123'); $b;" => "int" }
}

test_inference! {
    name = calls_a_closure_directly,
    cases = { "<?php (fn(string $x): int => (int) $x)('123');" => "int" }
}

test_inference! {
    name = closure_return_comes_from_docblock,
    cases = {
        concat!("<?php $f = ", "/** @return string */ ", "function () { return 1; }; ", "$f();") => "string",
    }
}

test_inference! {
    name = closure_return_is_inferred_from_body,
    cases = {
        "<?php $f = function () { return 1; }; $f();" => "int(1)",
        "<?php $f = function (string $s) { return $s; }; $f('x');" => "string",
    }
}

test_inference! {
    name = instantiates_generic_closure,
    cases = {
        indoc! {"
            <?php
            $id = /**
             * @template T
             * @param T $x
             * @return T
             */
            function ($x) { return $x; };
            $id(5);
        "} => "int(5)"
    }
}

test_inference! {
    name = instantiates_generic_arrow_function,
    cases = {
        indoc! {"
            <?php
            $identity = /**
             * @template T
             * @param T $x
             * @return T
             */
            fn ($x) => $x;
            $identity('hi');
        "} => "string('hi')"
    }
}
