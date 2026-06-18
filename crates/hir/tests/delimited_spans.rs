use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::r#type::TypeKind;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_span::Span;
use mago_syntax::parser::parse_file;

const CODE: &str = "<?php
strlen();
implode(', ', $parts);
new Foo;
new Bar();
";

fn slice_of(code: &str, span: Span) -> &str {
    let bytes = &code.as_bytes()[span.start.offset as usize..span.end.offset as usize];

    match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => panic!("the span must cover valid UTF-8, got {error}"),
    }
}

fn source_slice(span: Span) -> &'static str {
    slice_of(CODE, span)
}

#[test]
fn argument_lists_carry_their_delimiter_spans() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Owned(CODE.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let mut argument_list_texts = Vec::new();
    let mut instantiation_argument_list_texts = Vec::new();

    for statement in ir.statements {
        let StatementKind::Expression(expression) = &statement.kind else {
            continue;
        };

        match &expression.kind {
            ExpressionKind::Call(call) => argument_list_texts.push(source_slice(call.arguments.span)),
            ExpressionKind::Instantiation(instantiation) => instantiation_argument_list_texts
                .push(instantiation.arguments.map(|arguments| source_slice(arguments.span))),
            other => panic!("unexpected fixture expression: {other:?}"),
        }
    }

    assert_eq!(
        argument_list_texts,
        vec!["()", "(', ', $parts)"],
        "call argument lists must span their parentheses, delimiters included",
    );
    assert_eq!(
        instantiation_argument_list_texts,
        vec![None, Some("()")],
        "`new Foo` must lower without an argument list, `new Bar()` with an empty one",
    );
}

const TYPES: &str = "<?php
/**
 * @param ?bool $flag
 */
function f(?string $a, int|null $b, bool $flag): void {}
";

#[test]
fn type_hints_carry_their_own_spans() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"types.php"), Cow::Owned(TYPES.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let mut functions = ir.statements.iter().filter_map(|statement| {
        let StatementKind::Item(item) = &statement.kind else {
            return None;
        };
        let ItemStatementKind::Function(function) = &item.kind else {
            return None;
        };

        Some(*function)
    });

    let Some(function) = functions.next() else {
        panic!("the fixture must lower to a function");
    };

    let mut parameters = function.parameters.iter();

    // `?string` lowers to a nullable wrapping `string`: the `?` lives in the outer
    // span, the inner type spans just `string`. It is no longer expanded to `string|null`.
    let Some(first) = parameters.next() else {
        panic!("the fixture must have a first parameter");
    };
    let Some(first_type) = first.r#type else {
        panic!("the first parameter must carry a native type hint");
    };
    let TypeKind::Nullable(inner) = first_type.kind else {
        panic!("`?string` must lower to a nullable, got {:?}", first_type.kind);
    };
    assert_eq!(
        slice_of(TYPES, first_type.span),
        "?string",
        "the nullable must span the leading `?` and its inner type"
    );
    assert_eq!(slice_of(TYPES, inner.span), "string", "the nullable's inner type must span `string`");

    // `int|null` is an explicit union and stays one; each member carries its own span.
    let Some(second) = parameters.next() else {
        panic!("the fixture must have a second parameter");
    };
    let Some(second_type) = second.r#type else {
        panic!("the second parameter must carry a native type hint");
    };
    let TypeKind::Union(members) = second_type.kind else {
        panic!("`int|null` must lower to a union, got {:?}", second_type.kind);
    };
    let member_texts = members.iter().map(|member| slice_of(TYPES, member.span)).collect::<Vec<_>>();
    assert_eq!(member_texts, vec!["int", "null"], "explicit union members must carry their own spans");

    let Some(annotation) = function.annotation else {
        panic!("the fixture docblock must lower to an annotation");
    };
    let [parameter_annotation] = annotation.parameters else {
        panic!("the `@param` tag must lower to a parameter annotation");
    };
    let Some(r#type) = parameter_annotation.r#type else {
        panic!("the `@param` tag must carry a type");
    };
    let TypeAnnotationKind::Union(members) = r#type.kind else {
        panic!("the `?bool` annotation must lower to a union, got {:?}", r#type.kind);
    };

    let member_texts = members.iter().map(|member| slice_of(TYPES, member.span)).collect::<Vec<_>>();
    assert_eq!(
        member_texts,
        vec!["bool", "?"],
        "phpdoc union members must carry their own spans, with `?` spanning the implied null",
    );
}
