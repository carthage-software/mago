use indoc::indoc;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::statement::StatementKind;
use mago_inference::flow::Flow;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::harness::*;

test_inference! {
    name = folds_an_enum_declaration_without_error,
    code = "<?php enum Status { case Active; } $x = 1;",
    expect = |ir| {
        assert!(
            ir.statements.iter().any(|statement| matches!(statement.kind, StatementKind::Item(_))),
            "the enum declaration survives inference as an item",
        );
        assert!(get_last_statement(ir).meta.reachable, "code after an enum declaration is reachable");
    }
}

test_inference! {
    name = infers_constant_and_backed_case_values,
    code = indoc! {"
        <?php
        enum Suit: string {
            case Hearts = 'H';
            const COUNT = 4;
        }
    "},
    expect = |ir| {
        let Some(members) = item_members(ir) else { panic!("expected an enum") };
        let mut case_value = None;
        let mut constant_value = None;
        for member in members {
            match member.kind {
                MemberItemKind::EnumCase(case) => case_value = case.value.map(|value| value.meta.to_string()),
                MemberItemKind::Constant(constant) => constant_value = Some(constant.value.meta.to_string()),
                _ => {}
            }
        }
        assert_eq!(case_value.as_deref(), Some("string('H')"), "the backed case value is inferred");
        assert_eq!(constant_value.as_deref(), Some("int(4)"), "the constant value is inferred");
    }
}

test_inference! {
    name = binds_this_to_the_enum_inside_a_method,
    code = indoc! {"
        <?php
        enum Status {
            case Active;
            public function me() { return $this; }
        }
    "},
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"me") else { panic!("expected a return value in me()") };
        assert_eq!(value.meta.to_string(), "enum(Status)", "$this is an instance of the enclosing enum");
    }
}

test_inference! {
    name = resolves_self_constant_inside_a_method,
    code = indoc! {"
        <?php
        enum Status {
            case Active;
            const MAX = 5;
            public function limit(): int { return self::MAX; }
        }
    "},
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"limit") else { panic!("expected a return value in limit()") };
        assert_eq!(value.meta.to_string(), "int(5)", "self::MAX resolves to the constant");
    }
}

test_inference! {
    name = resolves_this_method_call_inside_a_method,
    code = indoc! {"
        <?php
        enum Status {
            case Active;
            public function size(): int { return 0; }
            public function twice(): int { return $this->size(); }
        }
    "},
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"twice") else { panic!("expected a return value in twice()") };
        assert_eq!(value.meta.to_string(), "int", "$this->size() resolves through the enclosing enum");
    }
}

test_inference! {
    name = folds_an_interface_declaration_without_error,
    code = "<?php interface HasName { public function name(): string; } $x = 1;",
    expect = |ir| {
        assert!(
            ir.statements.iter().any(|statement| matches!(statement.kind, StatementKind::Item(_))),
            "the interface declaration survives inference as an item",
        );
        assert!(get_last_statement(ir).meta.reachable, "code after an interface declaration is reachable");
    }
}

test_inference! {
    name = infers_interface_constant_value,
    code = "<?php interface Config { const VERSION = 2; }",
    expect = |ir| {
        let Some(value) = member_constant(ir, b"VERSION") else { panic!("expected a VERSION constant") };
        assert_eq!(value, "int(2)", "the interface constant value is inferred");
    }
}

test_inference! {
    name = resolves_self_constant_in_an_interface_constant,
    code = indoc! {"
        <?php
        interface Config {
            const A = 1;
            const B = self::A;
        }
    "},
    expect = |ir| {
        let Some(value) = member_constant(ir, b"B") else { panic!("expected a B constant") };
        assert_eq!(value, "int(1)", "self::A resolves to the earlier constant");
    }
}

test_inference! {
    name = folds_a_trait_declaration_without_error,
    code = indoc! {"
        <?php
        trait Greets {
            public function hello(): string { return 'hi'; }
        }
        $x = 1;
    "},
    expect = |ir| {
        assert!(
            ir.statements.iter().any(|statement| matches!(statement.kind, StatementKind::Item(_))),
            "the trait declaration survives inference as an item",
        );
        assert!(get_last_statement(ir).meta.reachable, "code after a trait declaration is reachable");
    }
}

test_inference! {
    name = resolves_this_method_call_inside_a_trait_method,
    code = indoc! {"
        <?php
        trait Math {
            public function one(): int { return 1; }
            public function again(): int { return $this->one(); }
        }
    "},
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"again") else { panic!("expected a return value in again()") };
        assert_eq!(value.meta.to_string(), "int", "$this->one() resolves through the enclosing trait");
    }
}

test_inference! {
    name = binds_this_as_static_inside_a_trait_method,
    code = indoc! {"
        <?php
        trait Identity {
            public function me() { return $this; }
        }
    "},
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"me") else { panic!("expected a return value in me()") };
        assert_eq!(value.meta.to_string(), "$this(Identity)", "$this is the $this/static self of the trait");
    }
}

test_inference! {
    name = folds_a_class_declaration_without_error,
    code = indoc! {"
        <?php
        class Box {
            public int $size = 0;
            public function size(): int { return $this->size; }
        }
        $x = 1;
    "},
    expect = |ir| {
        assert!(
            ir.statements.iter().any(|statement| matches!(statement.kind, StatementKind::Item(_))),
            "the class declaration survives inference as an item",
        );
        assert!(get_last_statement(ir).meta.reachable, "code after a class declaration is reachable");
    }
}

test_inference! {
    name = infers_property_default_value,
    code = "<?php class Box { public int $size = 7; }",
    expect = |ir| {
        let Some(members) = item_members(ir) else { panic!("expected a class") };
        let default = members.iter().find_map(|member| match member.kind {
            MemberItemKind::Property(property) => property.default_value.map(|value| value.meta.to_string()),
            _ => None,
        });
        assert_eq!(default.as_deref(), Some("int(7)"), "the property default value is inferred");
    }
}

test_inference! {
    name = reads_this_property_inside_a_class_method,
    code = indoc! {"
        <?php
        class Box {
            public int $size = 0;
            public function size(): int { return $this->size; }
        }
    "},
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"size") else { panic!("expected a return value in size()") };
        assert_eq!(value.meta.to_string(), "int", "$this->size resolves to the declared property type");
    }
}

test_inference! {
    name = folds_a_hooked_property_without_error,
    code = indoc! {"
        <?php
        class Box {
            public int $size { get => 42; }
        }
        $x = 1;
    "},
    expect = |ir| {
        assert!(
            ir.statements.iter().any(|statement| matches!(statement.kind, StatementKind::Item(_))),
            "the class with a hooked property survives inference",
        );
        assert!(get_last_statement(ir).meta.reachable, "code after the class is reachable");
    }
}

test_inference! {
    name = resolves_a_trait_method_call_through_the_using_class,
    def = indoc! {"
        <?php
        trait Greets {
            public function hello(): string { return 'hi'; }
        }
    "},
    code = indoc! {"
        <?php
        class Greeter {
            use Greets;
            public function greet(): string { return $this->hello(); }
        }
    "},
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"greet") else { panic!("expected a return value in greet()") };
        assert_eq!(value.meta.to_string(), "string", "$this->hello() resolves to the trait method's return type");
    }
}

test_inference! {
    name = resolves_parent_constant_and_method,
    code = indoc! {"
        <?php
        class Base {
            const VERSION = 1;
            public function base(): int { return 0; }
        }

        class Child extends Base {
            public function v(): int { return parent::VERSION; }
            public function b(): int { return parent::base(); }
        }
    "},
    expect = |ir| {
        let Some(constant) = method_return_value(ir, b"v") else { panic!("expected a return value in v()") };
        assert_eq!(constant.meta.to_string(), "int(1)", "parent::VERSION resolves to the parent's constant");

        let Some(method) = method_return_value(ir, b"b") else { panic!("expected a return value in b()") };
        assert_eq!(method.meta.to_string(), "int", "parent::base() resolves to the parent's method");
    }
}

test_inference! {
    name = folds_a_function_declaration_without_error,
    code = "<?php function f(): int { return 1; } $x = 1;",
    expect = |ir| {
        assert!(
            ir.statements.iter().any(|statement| matches!(statement.kind, StatementKind::Item(_))),
            "the function declaration survives inference as an item",
        );
        assert!(get_last_statement(ir).meta.reachable, "code after a function declaration is reachable");
    }
}

test_inference! {
    name = binds_a_native_parameter_type_in_the_body,
    code = indoc! {"
        <?php
        function f(int $a): int {
            $b = $a;
            return $b;
        }
    "},
    expect = |ir| {
        let Some(value) = function_return_value(ir, b"f") else { panic!("expected a return value in f()") };
        assert_eq!(value.meta.to_string(), "int", "the native parameter hint flows through the body");
    }
}

test_inference! {
    name = binds_a_docblock_parameter_type_in_the_body,
    code = indoc! {"
        <?php
        /** @param non-empty-string $a */
        function f($a) {
            return $a;
        }
    "},
    expect = |ir| {
        let Some(value) = function_return_value(ir, b"f") else { panic!("expected a return value in f()") };
        assert_eq!(value.meta.to_string(), "non-empty-string", "the @param hint flows through the body");
    }
}

fn function_return_value<'arena>(ir: TypedIr<'arena>, name: &[u8]) -> Option<&'arena TypedExpression<'arena>> {
    for statement in ir.statements {
        let StatementKind::Item(item) = statement.kind else { continue };
        if let ItemStatementKind::Function(function) = item.kind
            && function.name.value == name
        {
            return returned_value(function.body);
        }
    }

    None
}

/// The members of the first class-like declaration in the program, whatever its
/// kind, so a test can navigate enum/interface/trait members uniformly.
fn item_members(ir: TypedIr<'_>) -> Option<&[mago_hir::ir::item::member::MemberItem<'_, SymbolId, Flow, Type<'_>>]> {
    ir.statements.iter().find_map(|statement| match statement.kind {
        StatementKind::Item(item) => match item.kind {
            ItemStatementKind::Class(node) => Some(node.members.items),
            ItemStatementKind::Enum(node) => Some(node.members.items),
            ItemStatementKind::Interface(node) => Some(node.members.items),
            ItemStatementKind::Trait(node) => Some(node.members.items),
            _ => None,
        },
        _ => None,
    })
}

fn member_constant(ir: TypedIr<'_>, name: &[u8]) -> Option<String> {
    item_members(ir)?.iter().find_map(|member| match member.kind {
        MemberItemKind::Constant(constant) if constant.name.value == name => Some(constant.value.meta.to_string()),
        _ => None,
    })
}

fn method_return_value<'arena>(ir: TypedIr<'arena>, name: &[u8]) -> Option<&'arena TypedExpression<'arena>> {
    for statement in ir.statements {
        let StatementKind::Item(item) = statement.kind else { continue };
        let members = match item.kind {
            ItemStatementKind::Class(node) => node.members.items,
            ItemStatementKind::Enum(node) => node.members.items,
            ItemStatementKind::Interface(node) => node.members.items,
            ItemStatementKind::Trait(node) => node.members.items,
            _ => continue,
        };

        for member in members {
            if let MemberItemKind::Method(method) = member.kind
                && method.name.value == name
                && let Some(body) = method.body
            {
                return returned_value(body);
            }
        }
    }

    None
}

fn returned_value<'arena>(statement: &'arena TypedStatement<'arena>) -> Option<&'arena TypedExpression<'arena>> {
    match statement.kind {
        StatementKind::Return(value) => value,
        StatementKind::Sequence(statements) => statements.iter().find_map(returned_value),
        _ => None,
    }
}
