use indoc::indoc;
use mago_hir::ir::item::expression::ItemExpressionKind;

use crate::harness::*;

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
    name = closure_value_carries_its_signature,
    cases = {
        "<?php $a = fn(string $x): int => (int) $x; $a;" => "(closure(string): int)",
        "<?php function () {};" => "(closure(): mixed)",
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
    name = closure_without_declared_return_is_mixed,
    cases = { "<?php $f = function () { return 1; }; $f();" => "mixed" }
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
