use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::item::statement::r#enum::Enum;
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
    code = "<?php enum Suit: string { case Hearts = 'H'; const COUNT = 4; }",
    expect = |ir| {
        let Some(enum_node) = enum_of(ir) else { panic!("expected an enum") };
        let mut case_value = None;
        let mut constant_value = None;
        for member in enum_node.members.items {
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
    code = "<?php enum Status { case Active; public function me() { return $this; } }",
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"me") else { panic!("expected a return value in me()") };
        assert_eq!(value.meta.to_string(), "enum(Status)", "$this is an instance of the enclosing enum");
    }
}

test_inference! {
    name = resolves_self_constant_inside_a_method,
    code = "<?php enum Status { case Active; const MAX = 5; public function limit(): int { return self::MAX; } }",
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"limit") else { panic!("expected a return value in limit()") };
        assert_eq!(value.meta.to_string(), "int(5)", "self::MAX resolves to the constant");
    }
}

test_inference! {
    name = resolves_this_method_call_inside_a_method,
    code = "<?php enum Status { case Active; public function size(): int { return 0; } public function twice(): int { return $this->size(); } }",
    expect = |ir| {
        let Some(value) = method_return_value(ir, b"twice") else { panic!("expected a return value in twice()") };
        assert_eq!(value.meta.to_string(), "int", "$this->size() resolves through the enclosing enum");
    }
}

fn enum_of(ir: TypedIr<'_>) -> Option<&Enum<'_, SymbolId, Flow, Type<'_>>> {
    ir.statements.iter().find_map(|statement| match statement.kind {
        StatementKind::Item(item) => match item.kind {
            ItemStatementKind::Enum(node) => Some(node),
            _ => None,
        },
        _ => None,
    })
}

fn method_return_value<'arena>(ir: TypedIr<'arena>, name: &[u8]) -> Option<&'arena TypedExpression<'arena>> {
    for member in enum_of(ir)?.members.items {
        if let MemberItemKind::Method(method) = member.kind
            && method.name.value == name
            && let Some(body) = method.body
        {
            return returned_value(body);
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
