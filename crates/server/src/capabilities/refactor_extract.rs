//! Extract complete statement expressions without moving earlier evaluations.

use foldhash::{HashMap, HashSet};
use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};
use mago_names::ResolvedNames;
use mago_names::resolver::NameResolver;
use mago_span::{HasSpan, Span};
use mago_syntax::cst::*;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::{CodeActionItem, CodeActionKind, Range, Server, TextReplacement};

use super::refactor_edits::apply_ordered_edits;

pub(super) fn extract_local(server: &Server, file_id: FileId, start: u32, end: u32) -> Option<CodeActionItem> {
    let file = server.database().get(&file_id).ok()?;
    if file.file_type != FileType::Host || start >= end || end > file.size {
        return None;
    }
    let source = std::str::from_utf8(&file.contents).ok()?;
    let arena = LocalArena::new();
    let mut settings = server.parser_settings();
    settings.recover_incomplete = false;
    let program = parse_file_borrowed_with_settings(&arena, &file, settings);
    if program.has_errors() {
        return None;
    }
    let selected = Range::new(start, end);
    let target = find_target(Node::Program(program), None, selected)?;
    let mut preceding = target.statement.start.offset as usize;
    for trivia in program.trivia.iter().rev().filter(|trivia| trivia.end_offset() <= target.statement.start.offset) {
        if !source.as_bytes()[trivia.end_offset() as usize..preceding].iter().all(u8::is_ascii_whitespace) {
            break;
        }
        if trivia.kind.is_docblock() {
            return None;
        }
        preceding = trivia.start_offset() as usize;
    }
    let names = NameResolver::new(&arena).resolve(program);
    if any(Node::Program(program), &mut |node| {
        matches!(node, Node::MagicConstant(MagicConstant::Line(_)))
            || matches!(node, Node::Declare(declare) if declare.items.iter().any(|item| item.name.value.eq_ignore_ascii_case(b"ticks")))
    }) || any(target.scope, &mut |node| scope_hazard(node, &names))
        || any(Node::Expression(target.expression), &mut expression_hazard)
    {
        return None;
    }
    let mut used = HashSet::default();
    any(target.scope, &mut |node| {
        if let Node::DirectVariable(variable) = node {
            used.insert(variable.name);
        }
        false
    });
    let mut name = "$extracted".to_owned();
    let mut suffix = 2;
    while used.contains(name.as_bytes()) {
        name = format!("$extracted{suffix}");
        suffix += 1;
    }
    let insertion = target.statement.start.offset as usize;
    let line_start =
        source.as_bytes()[..insertion].iter().rposition(|byte| matches!(byte, b'\r' | b'\n')).map_or(0, |i| i + 1);
    let prefix = &source[line_start..insertion];
    let separator = if prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
        let newline = if source.contains("\r\n") {
            "\r\n"
        } else if source.contains('\n') {
            "\n"
        } else {
            "\r"
        };
        format!("{newline}{prefix}")
    } else {
        " ".to_owned()
    };
    let edits = vec![
        TextReplacement {
            range: Range::new(insertion as u32, insertion as u32),
            new_text: format!("{name} = {};{separator}", &source[start as usize..end as usize]),
        },
        TextReplacement { range: selected, new_text: name.clone() },
    ];
    let changed = apply_ordered_edits(source, &edits)?;
    let changed_file = File::ephemeral(file.name.clone(), changed.into_bytes().into());
    let changed_arena = LocalArena::new();
    if parse_file_borrowed_with_settings(&changed_arena, &changed_file, settings).has_errors() {
        return None;
    }
    Some(CodeActionItem {
        title: format!("Extract to {name}"),
        kind: CodeActionKind::RefactorExtract,
        edits: HashMap::from_iter([(file_id, edits)]),
        diagnostic: None,
    })
}

struct Target<'ast, 'arena> {
    statement: Span,
    expression: &'ast Expression<'arena>,
    scope: Node<'ast, 'arena>,
}

fn find_target<'ast, 'arena>(
    node: Node<'ast, 'arena>,
    scope: Option<Node<'ast, 'arena>>,
    selected: Range,
) -> Option<Target<'ast, 'arena>> {
    if node.start_offset() > selected.start || node.end_offset() < selected.end {
        return None;
    }
    let scope = match node {
        Node::Function(function) => function.ampersand.is_none().then_some(node),
        Node::Method(method) => method.ampersand.is_none().then_some(node),
        Node::Closure(closure) => closure.ampersand.is_none().then_some(node),
        Node::ArrowFunction(_) | Node::Class(_) | Node::AnonymousClass(_) => None,
        _ => scope,
    };
    if let Node::Block(block) = node
        && let Some(scope) = scope
    {
        for statement in &block.statements {
            let expression = match statement {
                Statement::Return(statement) if statement.terminator.is_semicolon() => statement.value,
                Statement::Expression(statement) if statement.terminator.is_semicolon() => match statement.expression {
                    Expression::Assignment(assignment)
                        if assignment.operator.is_assign()
                            && matches!(assignment.lhs, Expression::Variable(Variable::Direct(_)))
                            && scalar(assignment.rhs) =>
                    {
                        Some(assignment.rhs)
                    }
                    _ => None,
                },
                _ => None,
            };
            if let Some(expression) = expression
                && Range::new(expression.start_offset(), expression.end_offset()) == selected
            {
                return Some(Target { statement: statement.span(), expression, scope });
            }
        }
    }
    let mut found = None;
    node.visit_children(|child| {
        if found.is_none() {
            found = find_target(child, scope, selected);
        }
    });
    found
}

fn scalar(expression: &Expression<'_>) -> bool {
    match expression.unparenthesized() {
        Expression::Literal(_) => true,
        Expression::Binary(binary) => {
            binary.operator.is_comparison()
                || binary.operator.is_concatenation()
                || ((binary.operator.is_arithmetic() || binary.operator.is_bitwise())
                    && scalar(binary.lhs)
                    && scalar(binary.rhs))
        }
        Expression::UnaryPrefix(unary) => match unary.operator {
            UnaryPrefixOperator::BoolCast(..)
            | UnaryPrefixOperator::BooleanCast(..)
            | UnaryPrefixOperator::DoubleCast(..)
            | UnaryPrefixOperator::RealCast(..)
            | UnaryPrefixOperator::FloatCast(..)
            | UnaryPrefixOperator::IntCast(..)
            | UnaryPrefixOperator::IntegerCast(..)
            | UnaryPrefixOperator::StringCast(..)
            | UnaryPrefixOperator::BinaryCast(..)
            | UnaryPrefixOperator::Not(..) => true,
            UnaryPrefixOperator::Plus(_) | UnaryPrefixOperator::Negation(_) | UnaryPrefixOperator::BitwiseNot(_) => {
                scalar(unary.operand)
            }
            _ => false,
        },
        _ => false,
    }
}

fn scope_hazard(node: Node<'_, '_>, names: &ResolvedNames<'_>) -> bool {
    match node {
        Node::Variable(variable) => !matches!(variable, Variable::Direct(_)),
        Node::DirectVariable(variable) => variable.name == b"$GLOBALS",
        Node::MagicConstant(_) | Node::Yield(_) => true,
        Node::Try(statement) => statement.finally_clause.is_some(),
        Node::UnaryPrefix(unary) => matches!(unary.operator, UnaryPrefixOperator::Reference(_)),
        Node::Construct(construct) => matches!(
            construct,
            Construct::Eval(_)
                | Construct::Include(_)
                | Construct::IncludeOnce(_)
                | Construct::Require(_)
                | Construct::RequireOnce(_)
        ),
        Node::FunctionCall(call) => {
            let Expression::Identifier(identifier) = call.function.unparenthesized() else {
                return true;
            };
            let Some(resolved) = names.resolve(identifier) else { return true };
            let tail = resolved.rsplit(|byte| *byte == b'\\').next().unwrap_or(resolved);
            [
                b"extract".as_slice(),
                b"get_defined_vars",
                b"compact",
                b"parse_str",
                b"mb_parse_str",
                b"assert",
                b"await",
                b"func_get_args",
                b"func_get_arg",
                b"func_num_args",
                b"debug_backtrace",
                b"debug_print_backtrace",
                b"call_user_func",
                b"call_user_func_array",
                b"forward_static_call",
                b"forward_static_call_array",
            ]
            .iter()
            .any(|name| tail.eq_ignore_ascii_case(name))
        }
        _ => false,
    }
}

fn expression_hazard(node: Node<'_, '_>) -> bool {
    match node {
        Node::MethodCall(call) => suspends(&call.method),
        Node::StaticMethodCall(call) => suspends(&call.method),
        Node::Conditional(_)
        | Node::Match(_)
        | Node::Yield(_)
        | Node::Throw(_)
        | Node::Construct(_)
        | Node::NullSafeMethodCall(_)
        | Node::NullSafePropertyAccess(_)
        | Node::Closure(_)
        | Node::ArrowFunction(_)
        | Node::AnonymousClass(_)
        | Node::Assignment(_)
        | Node::Pipe(_) => true,
        Node::Binary(binary) => {
            binary.operator.is_logical() || matches!(binary.operator, BinaryOperator::NullCoalesce(_))
        }
        Node::UnaryPrefix(unary) => matches!(unary.operator, UnaryPrefixOperator::Reference(_)),
        _ => false,
    }
}

fn suspends(selector: &ClassLikeMemberSelector<'_>) -> bool {
    let ClassLikeMemberSelector::Identifier(name) = selector else { return true };
    name.value.eq_ignore_ascii_case(b"await") || name.value.eq_ignore_ascii_case(b"suspend")
}

fn any<'ast, 'arena>(node: Node<'ast, 'arena>, predicate: &mut impl FnMut(Node<'ast, 'arena>) -> bool) -> bool {
    if predicate(node) {
        return true;
    }
    let mut found = false;
    node.visit_children(|child| {
        if !found {
            found = any(child, predicate);
        }
    });
    found
}
