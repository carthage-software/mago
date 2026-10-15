//! Extract scalar return expressions without changing local state.

use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};
use mago_names::resolver::NameResolver;
use mago_span::HasSpan;
use mago_syntax::cst::*;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::{CodeActionItem, CodeActionKind, Range, Server, TextReplacement};

use super::refactor_edits::apply_ordered_edits;

pub(super) fn extract_method(server: &Server, file_id: FileId, start: u32, end: u32) -> Option<CodeActionItem> {
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
    let (class, method, statement) = find_target(Node::Program(program), None, None, selected)?;
    let MethodBody::Concrete(body) = &method.body else { return None };
    // A helper must not lose facts established by branches outside the selection.
    if body.statements.len() != 1
        || !matches!(body.statements.first(), Some(Statement::Return(_)))
        || method.ampersand.is_some()
        || method.name.value.starts_with(b"__")
    {
        return None;
    }
    let names = NameResolver::new(&arena).resolve(program);
    let metadata = server.codebase().get_class_like(names.resolve(&class.name.span.start)?)?;
    if metadata.span != class.span()
        || metadata.has_incomplete_hierarchy()
        || [b"__call".as_slice(), b"__callStatic"]
            .iter()
            .any(|name| server.codebase().get_declaring_method(metadata.name.as_bytes(), name).is_some())
    {
        return None;
    }
    let method_metadata = server.codebase().get_method(metadata.name.as_bytes(), method.name.value)?;
    if method_metadata.return_type_metadata.as_ref().map(|ty| &ty.type_union.types)
        != method_metadata.return_type_declaration_metadata.as_ref().map(|ty| &ty.type_union.types)
    {
        return None;
    }
    let return_hint = match &method.return_type_hint {
        Some(hint) if scalar_hint(&hint.hint) || matches!(hint.hint, Hint::Mixed(_)) => {
            &source[hint.start_offset() as usize..hint.end_offset() as usize]
        }
        Some(_) => return None,
        None => "",
    };
    let mut preceding = statement.start_offset() as usize;
    for trivia in program.trivia.iter().rev().filter(|trivia| trivia.end_offset() <= statement.start_offset()) {
        if !source.as_bytes()[trivia.end_offset() as usize..preceding].iter().all(u8::is_ascii_whitespace) {
            break;
        }
        if trivia.kind.is_docblock() {
            return None;
        }
        preceding = trivia.start_offset() as usize;
    }
    if program
        .trivia
        .iter()
        .any(|trivia| trivia.kind.is_comment() && trivia.start_offset() < end && start < trivia.end_offset())
    {
        return None;
    }
    let parameters: HashSet<_> = method
        .parameter_list
        .parameters
        .iter()
        .filter(|parameter| {
            parameter.ampersand.is_none()
                && parameter.ellipsis.is_none()
                && parameter.hint.as_ref().is_some_and(scalar_hint)
        })
        .map(|parameter| parameter.variable.name)
        .collect();
    if any(Node::Program(program), &mut |node| {
        matches!(node, Node::MagicConstant(MagicConstant::Line(_)))
            || matches!(node, Node::Declare(declare) if declare.items.iter().any(|item| item.name.value.eq_ignore_ascii_case(b"ticks")))
    }) || any(Node::MethodBody(&method.body), &mut |node| match node {
        Node::Expression(expression) => !scalar_node(expression, &parameters),
        Node::Global(_)
        | Node::Static(_)
        | Node::Unset(_)
        | Node::Foreach(_)
        | Node::Try(_)
        | Node::Function(_)
        | Node::Class(_)
        | Node::AnonymousClass(_)
        | Node::Trait(_)
        | Node::Interface(_)
        | Node::Enum(_)
        | Node::Goto(_)
        | Node::HaltCompiler(_)
        | Node::Inline(_) => true,
        _ => false,
    }) {
        return None;
    }
    let (arguments, declarations) = extracted_parameters(source, statement.value?, method, method_metadata)?;
    let name = fresh_method_name(server.codebase(), class, metadata)?;
    let expression = &source[start as usize..end as usize];
    let stub_source = format!(
        "<?php class MagoStub {{ private {}function {name}({declarations}){return_hint} {{ return {expression}; }} }}",
        if method.is_static() { "static " } else { "" }
    );
    let stub_file = File::ephemeral(b"extracted-method.php".as_slice().into(), stub_source.into_bytes().into());
    let stub = super::refactor_stub::format_body(server, &arena, &stub_file, settings)?;
    let mut action = super::refactor_implement::insert_methods(
        server,
        &file,
        program,
        Node::Class(class),
        &[stub],
        &format!("Extract method {name}"),
    )?;
    action.kind = CodeActionKind::RefactorExtract;
    let edits = action.edits.get_mut(&file_id)?;
    edits.push(TextReplacement {
        range: selected,
        new_text: format!("{}{name}({arguments})", if method.is_static() { "self::" } else { "$this->" }),
    });
    edits.sort_unstable_by_key(|edit| edit.range.start);
    let changed = apply_ordered_edits(source, edits)?;
    let changed_file = File::ephemeral(file.name.clone(), changed.into_bytes().into());
    if parse_file_borrowed_with_settings(&arena, &changed_file, settings).has_errors() {
        return None;
    }
    Some(action)
}

fn extracted_parameters(
    source: &str,
    expression: &Expression<'_>,
    method: &Method<'_>,
    method_metadata: &FunctionLikeMetadata,
) -> Option<(String, String)> {
    let mut used = Vec::new();
    any(Node::Expression(expression), &mut |node| {
        if let Node::DirectVariable(variable) = node {
            used.push((variable.start_offset(), variable.name));
        }
        false
    });
    used.sort_unstable_by_key(|(offset, _)| *offset);
    let mut seen = HashSet::default();
    let used = used
        .into_iter()
        .filter_map(|(_, name)| seen.insert(name).then_some(name))
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let arguments = used.join(", ");
    let mut declarations = Vec::with_capacity(used.len());
    for name in used {
        let parameter =
            method.parameter_list.parameters.iter().find(|parameter| parameter.variable.name == name.as_bytes())?;
        let metadata =
            method_metadata.parameters.iter().find(|parameter| parameter.name.0.as_bytes() == name.as_bytes())?;
        let declared = &metadata.type_declaration_metadata.as_ref()?.type_union;
        let hint = parameter.hint.as_ref()?;
        // The helper drops defaults, so its copied hint must admit null on its own.
        if metadata.type_metadata.as_ref().is_none_or(|ty| ty.type_union.types != declared.types)
            || (parameter.default_value.is_some()
                && !hint.contains_null()
                && metadata.default_type.as_ref().is_none_or(|ty| ty.type_union.can_be_null()))
        {
            return None;
        }
        declarations.push(format!("{} {name}", &source[hint.start_offset() as usize..hint.end_offset() as usize]));
    }
    Some((arguments, declarations.join(", ")))
}

fn fresh_method_name(codebase: &CodebaseMetadata, class: &Class<'_>, metadata: &ClassLikeMetadata) -> Option<String> {
    let mut reserved = HashSet::default();
    any(Node::Class(class), &mut |node| {
        if let Node::LocalIdentifier(name) = node {
            reserved.insert(name.value.to_ascii_lowercase());
        }
        false
    });
    let relatives = std::iter::once(metadata.name)
        .chain(metadata.all_parent_classes.iter().copied())
        .chain(codebase.all_class_like_descendants.get(&metadata.name).into_iter().flatten().copied());
    for relative in relatives {
        let relative = codebase.get_class_like_by_name(relative)?;
        reserved.extend(
            relative
                .declaring_method_ids
                .keys()
                .chain(&relative.pseudo_methods)
                .chain(&relative.static_pseudo_methods)
                .map(|name| name.as_bytes().to_ascii_lowercase()),
        );
    }
    let mut name = "extracted".to_owned();
    let mut suffix = 2;
    while reserved.contains(name.as_bytes()) {
        name = format!("extracted{suffix}");
        suffix += 1;
    }
    Some(name)
}

fn scalar_hint(hint: &Hint<'_>) -> bool {
    match hint {
        Hint::Bool(_)
        | Hint::Float(_)
        | Hint::Integer(_)
        | Hint::String(_)
        | Hint::Null(_)
        | Hint::True(_)
        | Hint::False(_) => true,
        Hint::Nullable(hint) => scalar_hint(hint.hint),
        Hint::Parenthesized(hint) => scalar_hint(hint.hint),
        Hint::Union(hint) => scalar_hint(hint.left) && scalar_hint(hint.right),
        _ => false,
    }
}

fn scalar_node(expression: &Expression<'_>, parameters: &HashSet<&[u8]>) -> bool {
    match expression {
        Expression::Literal(Literal::String(string)) => !string.raw.contains(&b'\n') && !string.raw.contains(&b'\r'),
        Expression::Literal(_) => true,
        Expression::Variable(Variable::Direct(variable)) => parameters.contains(variable.name),
        Expression::Parenthesized(_) => true,
        Expression::Binary(binary) => !matches!(binary.operator, BinaryOperator::Instanceof(_)),
        Expression::UnaryPrefix(unary) => matches!(
            unary.operator,
            UnaryPrefixOperator::BoolCast(..)
                | UnaryPrefixOperator::BooleanCast(..)
                | UnaryPrefixOperator::DoubleCast(..)
                | UnaryPrefixOperator::RealCast(..)
                | UnaryPrefixOperator::FloatCast(..)
                | UnaryPrefixOperator::IntCast(..)
                | UnaryPrefixOperator::IntegerCast(..)
                | UnaryPrefixOperator::StringCast(..)
                | UnaryPrefixOperator::BinaryCast(..)
                | UnaryPrefixOperator::Not(_)
                | UnaryPrefixOperator::Plus(_)
                | UnaryPrefixOperator::Negation(_)
                | UnaryPrefixOperator::BitwiseNot(_)
        ),
        _ => false,
    }
}

fn find_target<'ast, 'arena>(
    node: Node<'ast, 'arena>,
    mut class: Option<&'ast Class<'arena>>,
    mut method: Option<&'ast Method<'arena>>,
    selected: Range,
) -> Option<(&'ast Class<'arena>, &'ast Method<'arena>, &'ast Return<'arena>)> {
    if node.start_offset() > selected.start || selected.end > node.end_offset() {
        return None;
    }
    match node {
        Node::Class(current) => {
            class = Some(current);
            method = None;
        }
        Node::Method(current) => method = Some(current),
        Node::AnonymousClass(_)
        | Node::Trait(_)
        | Node::Interface(_)
        | Node::Enum(_)
        | Node::Closure(_)
        | Node::ArrowFunction(_)
        | Node::Function(_) => return None,
        Node::Return(statement)
            if statement.terminator.is_semicolon()
                && statement
                    .value
                    .is_some_and(|value| Range::new(value.start_offset(), value.end_offset()) == selected) =>
        {
            return Some((class?, method?, statement));
        }
        _ => {}
    }
    let mut found = None;
    node.visit_children(|child| {
        if found.is_none() {
            found = find_target(child, class, method, selected);
        }
    });
    found
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
