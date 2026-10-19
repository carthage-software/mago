use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_span::{HasSpan, Span};
use mago_syntax::cst::{
    Access, Call, ClassLikeConstantSelector, ClassLikeMember, Expression, ForBody, ForeachBody, IfBody, MethodBody,
    Program, Statement, Terminator, Variable, WhileBody,
};
use mago_syntax::error::{Expected, ParseError};
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;

#[test]
fn editor_recovery_keeps_nested_bodies_and_reports_missing_delimiters() {
    let source = b"<?php class C { function work($value) { take([1, 2], $value  ";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_slice().into());
    let arena = LocalArena::new();
    let strict = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
    assert!(!strict.statements.iter().any(|statement| matches!(statement, Statement::Class(_))));
    assert!(!strict.errors.is_empty());
    let recovered = parse_file_borrowed_with_settings(
        &arena,
        &file,
        ParserSettings { recover_incomplete: true, ..Default::default() },
    );
    let Some(class) = recovered.statements.iter().find_map(|statement| match statement {
        Statement::Class(class) => Some(class),
        _ => None,
    }) else {
        panic!("missing class")
    };
    let ClassLikeMember::Method(method) = &class.members[0] else { panic!("missing method") };
    let MethodBody::Concrete(body) = &method.body else { panic!("missing body") };
    assert_eq!(body.statements.len(), 1);
    assert_eq!(class.end_offset(), file.size);
    assert!(recovered.errors.len() >= 3);
}

#[test]
fn editor_recovery_handles_each_truncated_prefix_without_changing_valid_trees() {
    for source in [
        "<?php namespace App { class C { function work($value) { return take([1, 2], ($value)); } } }",
        "<?php $f = fn($x) => [$x, 1]; $g = function($y) use ($f) { return $f($y); };",
        "<?php interface I { function f(): void; } trait T { function f(): void {} } enum E { case A; }",
        "<?php function f($a) { if ($a->b) return 1; elseif ($a) {} else {} while ($a->c) {} do {} while ($a); switch ($a) {} }",
        "<?php function f($a) { foreach ($a as $k => $v) $a(); for ($i = 0; $i < $a; $i++) {} if ($a): elseif ($a): endif; }",
        include_str!("fixtures/issue_2589_complete.php"),
    ] {
        let settings = ParserSettings { recover_incomplete: true, ..Default::default() };
        for end in 0..=source.len() {
            let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes()[..end].into());
            let arena = LocalArena::new();
            let recovered = parse_file_borrowed_with_settings(&arena, &file, settings);
            assert!(recovered.statements.iter().all(|statement| statement.end_offset() <= file.size));
            if end == source.len() {
                let strict = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
                assert!(recovered.errors.is_empty());
                assert_eq!(format!("{strict:?}"), format!("{recovered:?}"));
            }
        }
    }
}

fn assert_missing_static_member(source: &'static str) {
    for source in [source, source.trim_end()] {
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
        let arena = LocalArena::new();
        let strict = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
        assert!(!strict.errors.is_empty(), "{source}");
        assert!(!strict.statements.iter().any(|statement| matches!(statement, Statement::Expression(_))), "{source}");

        let recovered = parse_file_borrowed_with_settings(
            &arena,
            &file,
            ParserSettings { recover_incomplete: true, ..Default::default() },
        );
        assert!(!recovered.errors.is_empty(), "{source}");
        assert!(
            recovered
                .errors
                .iter()
                .all(|error| !matches!(error, ParseError::UnexpectedEndOfFile(Expected::OneOf([]), ..))),
            "{source}: {:?}",
            recovered.errors
        );

        let Some(Statement::Expression(statement)) = recovered.statements.last() else {
            panic!("{source}: missing expression statement")
        };
        assert!(matches!(statement.terminator, Terminator::Missing(_)), "{source}");

        let expression = match statement.expression {
            Expression::Assignment(assignment) => assignment.rhs,
            Expression::Call(Call::Function(call)) => {
                assert_eq!(call.argument_list.arguments.len(), 1, "{source}");
                call.argument_list.arguments[0].value()
            }
            expression => expression,
        };
        let Expression::Access(Access::ClassConstant(access)) = expression else {
            panic!("{source}: missing static access: {expression:?}")
        };
        let Expression::Identifier(class) = access.class else { panic!("{source}: missing class name") };
        assert_eq!(class.value(), b"Foo", "{source}");
        let ClassLikeConstantSelector::Missing(span) = access.constant else {
            panic!("{source}: expected missing member")
        };
        assert_eq!(span.length(), 0, "{source}");
        assert!(access.end_offset() <= file.size, "{source}");
    }
}

#[test]
fn editor_recovery_keeps_static_access_at_eof() {
    assert_missing_static_member(include_str!("fixtures/issue_2589_static_access.php"));
}

#[test]
fn editor_recovery_keeps_static_access_argument_at_eof() {
    assert_missing_static_member(include_str!("fixtures/issue_2589_static_argument.php"));
}

#[test]
fn editor_recovery_keeps_static_property_at_eof() {
    assert_missing_static_member(include_str!("fixtures/issue_2589_static_property.php"));
}

#[test]
fn editor_recovery_keeps_static_property_argument_at_eof() {
    assert_missing_static_member(include_str!("fixtures/issue_2589_static_property_argument.php"));
}

fn function_body<'arena>(program: &'arena Program<'arena>) -> Option<&'arena [Statement<'arena>]> {
    program.statements.iter().find_map(|statement| match statement {
        Statement::Function(function) => Some(function.body.statements.as_slice()),
        _ => None,
    })
}

fn control_head<'arena>(statement: &'arena Statement<'arena>) -> Option<(&'arena Expression<'arena>, Span)> {
    match statement {
        Statement::If(r#if) => Some(match &r#if.body {
            IfBody::Statement(body) => {
                body.else_if_clauses.last().map_or((r#if.condition, r#if.right_parenthesis), |clause| {
                    (clause.condition, clause.right_parenthesis)
                })
            }
            IfBody::ColonDelimited(body) => {
                body.else_if_clauses.last().map_or((r#if.condition, r#if.right_parenthesis), |clause| {
                    (clause.condition, clause.right_parenthesis)
                })
            }
        }),
        Statement::While(r#while) => Some((r#while.condition, r#while.right_parenthesis)),
        Statement::DoWhile(do_while) => Some((do_while.condition, do_while.right_parenthesis)),
        Statement::Switch(switch) => Some((switch.expression, switch.right_parenthesis)),
        _ => None,
    }
}

fn control_tail(statement: &Statement<'_>) -> Option<Span> {
    match statement {
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(body) => {
                Some(body.else_if_clauses.last().map_or(body.statement, |clause| clause.statement).span())
            }
            IfBody::ColonDelimited(_) => None,
        },
        Statement::While(r#while) => match r#while.body {
            WhileBody::Statement(body) => Some(body.span()),
            WhileBody::ColonDelimited(_) => None,
        },
        Statement::DoWhile(do_while) => Some(do_while.terminator.span()),
        Statement::Switch(switch) => Some(switch.body.span()),
        _ => None,
    }
}

#[test]
fn editor_recovery_keeps_control_heads_missing_their_closing_parenthesis() {
    for (source, closed) in [
        ("<?php function f($a) { if ($a->\n return 1; }", false),
        ("<?php function f($a) { if ($a->x }", false),
        ("<?php function f($a) { if ($a->x", false),
        ("<?php function f($a) { if ($a) {} elseif ($a->x }", false),
        ("<?php function f($a) { if ($a): elseif ($a->x: endif; }", false),
        ("<?php function f($a) { if ($a->x) }", true),
        ("<?php function f($a) { if ($a) {} else }", true),
        ("<?php function f($a) { while ($a->\n return 1; }", false),
        ("<?php function f($a) { while ($a->x) }", true),
        ("<?php function f($a) { while ($a->", false),
        ("<?php function f($a) { switch ($a->\n return 1; }", false),
        ("<?php function f($a) { switch ($a->", false),
        ("<?php function f($a) { do {} while ($a->\n}", false),
        ("<?php function f($a) { do {} while ($a->", false),
    ] {
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
        let arena = LocalArena::new();
        let strict = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
        assert!(!strict.errors.is_empty(), "{source}");
        assert!(
            function_body(strict).is_none_or(|body| body.iter().all(|statement| control_head(statement).is_none())),
            "{source}"
        );
        let recovered = parse_file_borrowed_with_settings(
            &arena,
            &file,
            ParserSettings { recover_incomplete: true, ..Default::default() },
        );
        assert!(!recovered.errors.is_empty(), "{source}");
        let body = function_body(recovered).unwrap_or_else(|| panic!("{source}: missing function"));
        let (condition, right_parenthesis) =
            body.iter().find_map(control_head).unwrap_or_else(|| panic!("{source}: missing control statement"));
        let receiver = match condition {
            Expression::Access(Access::Property(access)) => access.object,
            condition => condition,
        };
        let Expression::Variable(Variable::Direct(variable)) = receiver else {
            panic!("{source}: unexpected condition {condition:?}")
        };
        assert_eq!(variable.name, b"$a", "{source}");
        assert_eq!(right_parenthesis.length() == 0, !closed, "{source}");
        assert!(recovered.errors.len() <= strict.errors.len(), "{source}: {:?}", recovered.errors);
        if !closed && let Some(tail) = body.iter().find_map(control_tail) {
            assert_eq!(tail, right_parenthesis, "{source}");
        }
        assert!(body.iter().all(|statement| statement.end_offset() <= file.size), "{source}");
        if let Some(value) = source.rfind("1;").and_then(|index| u32::try_from(index).ok()) {
            assert!(
                body.iter().any(|statement| control_head(statement).is_none() && statement.span().has_offset(value)),
                "{source}"
            );
        }
    }
}

fn right_parentheses(statement: &Statement<'_>) -> Vec<Span> {
    match statement {
        Statement::If(r#if) => std::iter::once(r#if.right_parenthesis)
            .chain(match &r#if.body {
                IfBody::Statement(body) => body.else_if_clauses.iter().map(|clause| clause.right_parenthesis).collect(),
                IfBody::ColonDelimited(body) => {
                    body.else_if_clauses.iter().map(|clause| clause.right_parenthesis).collect::<Vec<_>>()
                }
            })
            .collect(),
        Statement::While(r#while) => vec![r#while.right_parenthesis],
        Statement::Switch(switch) => vec![switch.right_parenthesis],
        _ => vec![],
    }
}

#[test]
fn editor_recovery_keeps_existing_bodies_of_control_heads_missing_their_closing_parenthesis() {
    for source in [
        "<?php function f($a, $b) { if ($a {\n return 1; } return 0; }",
        "<?php function f($a, $b) { if ($a > 1 {\n return 1; } elseif ($b) { return 2; } else { return 3; } return 0; }",
        "<?php function f($a, $b) { if ($a) { return 1; } elseif ($b {\n return 2; } else { return 3; } return 0; }",
        "<?php function f($a, $b) { while ($a instanceof B {\n return 1; } return 0; }",
        "<?php function f($a, $b) { switch ($a {\n case 1: return 1; default: return 2; } return 0; }",
        "<?php function f($a, $b) { if ($a:\n return 1; elseif ($b): return 2; else: return 3; endif; return 0; }",
        "<?php function f($a, $b) { if ($a): return 1; elseif ($b:\n return 2; else: return 3; endif; return 0; }",
        "<?php function f($a, $b) { while ($a:\n return 1; endwhile; return 0; }",
    ] {
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
        let arena = LocalArena::new();
        let recovered = parse_file_borrowed_with_settings(
            &arena,
            &file,
            ParserSettings { recover_incomplete: true, ..Default::default() },
        );
        assert_eq!(recovered.errors.len(), 1, "{source}: {:?}", recovered.errors);
        let body = function_body(recovered).unwrap_or_else(|| panic!("{source}: missing function"));
        let [statement, Statement::Return(_)] = body else { panic!("{source}: {body:?}") };
        assert_eq!(right_parentheses(statement).iter().filter(|span| span.length() == 0).count(), 1, "{source}");
        match statement {
            Statement::If(r#if) => {
                if let IfBody::Statement(body) = &r#if.body {
                    assert!(
                        std::iter::once(body.statement)
                            .chain(body.else_if_clauses.iter().map(|clause| clause.statement))
                            .chain(body.else_clause.iter().map(|clause| clause.statement))
                            .all(|statement| matches!(statement, Statement::Block(_))),
                        "{source}"
                    );
                }
            }
            Statement::While(r#while) => assert!(
                matches!(r#while.body, WhileBody::Statement(Statement::Block(_)) | WhileBody::ColonDelimited(_)),
                "{source}"
            ),
            Statement::Switch(switch) => assert_eq!(switch.body.cases().len(), 2, "{source}"),
            statement => panic!("{source}: {statement:?}"),
        }
        for value in ["1;", "2;", "3;"] {
            if let Some(offset) = source.find(value).and_then(|index| u32::try_from(index).ok()) {
                assert!(statement.span().has_offset(offset), "{source}: {value}");
            }
        }
    }
}

#[test]
fn editor_recovery_reports_each_missing_token_once() {
    for source in [
        "<?php function f($a) { if ($a }",
        "<?php function f($a) { if ($a-> }",
        "<?php function f($a) { if ($a) {} elseif ($a }",
        "<?php function f($a) { while ($a }",
        "<?php function f($a) { while ($a) }",
        "<?php function f($a) { switch ($a\n return 1; }",
        "<?php function f($a) { switch ($a }",
        "<?php function f($a) { do {} while ($a\n return 1; }",
        "<?php function f($a) { do {} while ($a }",
        "<?php function f($a) { foreach ($a as $b) }",
        "<?php if ($a",
        "<?php if ($a)",
        "<?php if (foo($a",
        "<?php while ($a",
        "<?php switch ($a",
        "<?php do {} while ($a",
    ] {
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
        let arena = LocalArena::new();
        let recovered = parse_file_borrowed_with_settings(
            &arena,
            &file,
            ParserSettings { recover_incomplete: true, ..Default::default() },
        );
        assert_eq!(recovered.errors.len(), 1, "{source}: {:?}", recovered.errors);
    }
}

fn loop_body<'arena>(statement: &'arena Statement<'arena>) -> Option<&'arena Statement<'arena>> {
    match statement {
        Statement::Foreach(foreach) => match &foreach.body {
            ForeachBody::Statement(body) => Some(body),
            ForeachBody::ColonDelimited(_) => None,
        },
        Statement::For(r#for) => match &r#for.body {
            ForBody::Statement(body) => Some(body),
            ForBody::ColonDelimited(_) => None,
        },
        _ => None,
    }
}

#[test]
fn editor_recovery_keeps_loops_missing_their_body() {
    for source in [
        "<?php function f($a) { foreach ($a as $b) }",
        "<?php function f($a) { foreach ($a as $k => $b)",
        "<?php function f($a) { for ($i = 0; $i < $a; $i++) }",
        "<?php function f($a) { for (;;)",
    ] {
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
        let arena = LocalArena::new();
        let strict = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
        assert!(!strict.errors.is_empty(), "{source}");
        assert!(function_body(strict).is_none_or(|body| body.iter().all(|statement| loop_body(statement).is_none())));
        let recovered = parse_file_borrowed_with_settings(
            &arena,
            &file,
            ParserSettings { recover_incomplete: true, ..Default::default() },
        );
        assert!(!recovered.errors.is_empty(), "{source}");
        let Some(body) = function_body(recovered).and_then(|body| body.iter().find_map(loop_body)) else {
            panic!("{source}: missing loop")
        };
        assert!(matches!(body, Statement::Noop(span) if span.length() == 0), "{source}: {body:?}");
        assert!(body.end_offset() <= file.size, "{source}");
    }
}
