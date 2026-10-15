//! Extract literal values into private constants without moving existing declarations.

use foldhash::{HashMap, HashSet};
use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_span::{HasSpan, Span};
use mago_syntax::cst::*;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::{CodeActionItem, CodeActionKind, Range, Server, TextReplacement};

use super::refactor_edits::apply_ordered_edits;

pub(super) fn extract_constant(server: &Server, file_id: FileId, start: u32, end: u32) -> Option<CodeActionItem> {
    let file = server.database().get(&file_id).ok()?;
    if file.file_type != FileType::Host || start >= end || end > file.size || server.php_version < PHPVersion::PHP71 {
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
    let target = find_target(Node::Program(program), selected)?;
    if program.trivia.iter().any(|trivia| {
        trivia.kind.is_docblock()
            && target.last_member_end <= trivia.start_offset()
            && trivia.end_offset() <= target.close.start.offset
    }) {
        return None;
    }
    let names = NameResolver::new(&arena).resolve(program);
    let class = server.codebase().get_class_like(names.resolve(&target.name.start)?)?;
    if class.span != target.owner.span() {
        return None;
    }
    let mut used = HashSet::default();
    let mut visited = HashSet::default();
    let mut pending = vec![class];
    while let Some(class) = pending.pop() {
        if !visited.insert(class.name) {
            continue;
        }
        if class.has_incomplete_hierarchy() {
            return None;
        }
        used.extend(
            class
                .constants
                .keys()
                .chain(class.trait_constant_ids.keys())
                .chain(class.enum_cases.keys())
                .map(|name| name.as_bytes()),
        );
        pending.extend(
            class
                .all_parent_classes
                .iter()
                .chain(&class.all_parent_interfaces)
                .chain(&class.used_traits)
                .filter_map(|name| server.codebase().get_class_like_by_name(*name)),
        );
    }
    let mut nodes = vec![Node::Program(program)];
    while let Some(node) = nodes.pop() {
        if matches!(node, Node::MagicConstant(MagicConstant::Line(_))) {
            return None;
        }
        if let Node::ClassConstantAccess(access) = node
            && let ClassLikeConstantSelector::Identifier(name) = &access.constant
        {
            used.insert(name.value);
        }
        node.visit_children(|child| nodes.push(child));
    }
    let mut name = "EXTRACTED".to_owned();
    let mut suffix = 2;
    while used.contains(name.as_bytes()) {
        name = format!("EXTRACTED_{suffix}");
        suffix += 1;
    }
    let close = target.close.start.offset as usize;
    let close_line = line_start(source, close);
    let prefix = &source[close_line..close];
    let own_line = prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t'));
    let value = &source[start as usize..end as usize];
    let declaration = if server.php_version >= PHPVersion::PHP83 {
        format!("private const {} {name} = {value};", target.value_type)
    } else {
        format!("private const {name} = {value};")
    };
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else if source.contains('\r') {
        "\r"
    } else {
        "\n"
    };
    let owner = target.owner.start_offset() as usize;
    let owner_line = &source[line_start(source, owner)..owner];
    let indent_len = owner_line.bytes().take_while(|byte| matches!(byte, b' ' | b'\t')).count();
    let class_indent = if own_line { prefix } else { &owner_line[..indent_len] };
    let method = target.method.start_offset() as usize;
    let method_prefix = &source[line_start(source, method)..method];
    let indent = if method_prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
        method_prefix.to_owned()
    } else {
        let unit = if server.formatter.use_tabs { "\t".to_owned() } else { " ".repeat(server.formatter.tab_width) };
        format!("{class_indent}{unit}")
    };
    let (offset, insertion_end, new_text) = if own_line {
        let before = source[..close_line].strip_suffix(newline).unwrap_or("");
        let previous_line = before.rsplit(['\r', '\n']).next().unwrap_or("");
        let separator = if previous_line.trim().is_empty() { "" } else { newline };
        (close_line, close_line, format!("{separator}{indent}{declaration}{newline}"))
    } else {
        let offset = source[..close].trim_end_matches([' ', '\t']).len();
        (offset, close, format!("{newline}{newline}{indent}{declaration}{newline}{class_indent}"))
    };
    let replacement = format!("self::{name}");
    let edits = vec![
        TextReplacement { range: selected, new_text: replacement.clone() },
        TextReplacement { range: Range::new(offset as u32, insertion_end as u32), new_text },
    ];
    let updated = apply_ordered_edits(source, &edits)?;
    let changed = File::ephemeral(file.name.clone(), updated.into_bytes().into());
    let changed_arena = LocalArena::new();
    if parse_file_borrowed_with_settings(&changed_arena, &changed, settings).has_errors() {
        return None;
    }
    Some(CodeActionItem {
        title: format!("Extract to {replacement}"),
        kind: CodeActionKind::RefactorExtract,
        edits: HashMap::from_iter([(file_id, edits)]),
        diagnostic: None,
    })
}

struct Target<'ast, 'arena> {
    owner: Node<'ast, 'arena>,
    name: Span,
    close: Span,
    last_member_end: u32,
    method: &'ast Method<'arena>,
    value_type: &'static str,
}

fn find_target<'ast, 'arena>(node: Node<'ast, 'arena>, selected: Range) -> Option<Target<'ast, 'arena>> {
    if node.start_offset() > selected.start
        || node.end_offset() < selected.end
        || matches!(node, Node::Trait(_) | Node::AnonymousClass(_))
    {
        return None;
    }
    let owner = match node {
        Node::Class(class) => Some((class.name.span, class.right_brace, &class.members)),
        Node::Enum(enumeration) => Some((enumeration.name.span, enumeration.right_brace, &enumeration.members)),
        _ => None,
    };
    if let Some((name, close, members)) = owner {
        for member in members {
            if let ClassLikeMember::Method(method) = member
                && let MethodBody::Concrete(body) = &method.body
                && let Some(value_type) = selected_expression(Node::Block(body), selected)
            {
                return Some(Target {
                    owner: node,
                    name,
                    close,
                    last_member_end: members.last()?.end_offset(),
                    method,
                    value_type,
                });
            }
        }
    }
    let mut found = None;
    node.visit_children(|child| {
        if found.is_none() {
            found = find_target(child, selected);
        }
    });
    found
}

fn selected_expression(node: Node<'_, '_>, selected: Range) -> Option<&'static str> {
    if node.start_offset() > selected.start
        || node.end_offset() < selected.end
        || matches!(
            node,
            Node::DeclareItem(_)
                | Node::Break(_)
                | Node::Continue(_)
                | Node::Function(_)
                | Node::Closure(_)
                | Node::ArrowFunction(_)
                | Node::Class(_)
                | Node::Enum(_)
                | Node::Interface(_)
                | Node::Trait(_)
                | Node::AnonymousClass(_)
        )
    {
        return None;
    }
    if let Node::Expression(expression) = node
        && Range::new(node.start_offset(), node.end_offset()) == selected
    {
        return constant_type(expression);
    }
    let mut found = None;
    node.visit_children(|child| {
        if found.is_none() {
            found = selected_expression(child, selected);
        }
    });
    found
}

fn constant_type(expression: &Expression<'_>) -> Option<&'static str> {
    match expression.unparenthesized() {
        Expression::Literal(literal) => Some(super::refactor_literal::native_hint(literal).unwrap_or("int|float")),
        Expression::UnaryPrefix(unary)
            if matches!(unary.operator, UnaryPrefixOperator::Plus(_) | UnaryPrefixOperator::Negation(_))
                && matches!(
                    unary.operand.unparenthesized(),
                    Expression::Literal(Literal::Integer(_) | Literal::Float(_))
                ) =>
        {
            constant_type(unary.operand)
        }
        Expression::Array(array) => array.elements.iter().all(constant_element).then_some("array"),
        Expression::LegacyArray(array) => array.elements.iter().all(constant_element).then_some("array"),
        _ => None,
    }
}

fn constant_element(element: &ArrayElement<'_>) -> bool {
    match element {
        ArrayElement::Value(element) => constant_type(element.value).is_some(),
        ArrayElement::KeyValue(element) => {
            let key = element.key.unparenthesized();
            (matches!(key, Expression::Literal(Literal::Integer(_) | Literal::String(_)))
                || matches!(key, Expression::UnaryPrefix(unary) if matches!(unary.operand.unparenthesized(), Expression::Literal(Literal::Integer(_))) && constant_type(key).is_some()))
                && constant_type(element.value).is_some()
        }
        _ => false,
    }
}

fn line_start(source: &str, offset: usize) -> usize {
    source.as_bytes()[..offset].iter().rposition(|byte| matches!(byte, b'\r' | b'\n')).map_or(0, |i| i + 1)
}
