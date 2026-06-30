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

test_inference! {
    name = an_anonymous_class_is_an_instance_of_itself,
    code = "<?php $a = new class {}; $a;",
    expect = |ir| {
        let meta = get_last_expression(ir).meta.to_string();
        assert!(meta.starts_with("{anonymous-class"), "the value is an instance of the anonymous class, got {meta}");
    }
}

test_inference! {
    name = resolves_a_method_on_an_anonymous_class,
    cases = {
        "<?php $a = new class { public function f(): int { return 1; } }; $a->f();" => "int",
    }
}

test_inference! {
    name = resolves_an_inherited_method_on_an_anonymous_class,
    cases = {
        indoc! {"
            <?php
            class Base {
                public function b(): int { return 0; }
            }
            $a = new class extends Base {};
            $a->b();
        "} => "int",
    }
}

test_inference! {
    name = resolves_an_interface_method_on_an_anonymous_class,
    cases = {
        indoc! {"
            <?php
            interface I {
                public function j(): string;
            }
            $a = new class implements I {
                public function j(): string { return 'x'; }
            };
            $a->j();
        "} => "string",
    }
}

test_inference! {
    name = binds_this_in_an_anonymous_class_constructor,
    cases = {
        indoc! {"
            <?php
            $a = new class(1) {
                public int $x;
                public function __construct(int $x) { $this->x = $x; }
            };
            $a->x;
        "} => "int",
    }
}

test_inference! {
    name = anonymous_class_constructor_arguments_are_inferred,
    code = "<?php $a = new class(1 + 2) {};",
    expect = |ir| {
        let ExpressionKind::Assignment(assignment) = expression_of(get_last_statement(ir)).kind else {
            panic!("expected an assignment")
        };
        let ExpressionKind::Item(item) = assignment.right.kind else { panic!("expected an item expression") };
        let ItemExpressionKind::AnonymousClass(class) = item.kind else { panic!("expected an anonymous class") };
        let Some(arguments) = class.arguments.as_ref() else { panic!("expected constructor arguments") };
        let Some(PartialArgument::Value(argument)) = arguments.items.first() else {
            panic!("expected a value argument")
        };
        assert_eq!(argument.meta.to_string(), "int(3)", "the constructor argument is inferred, not mixed");
    }
}
