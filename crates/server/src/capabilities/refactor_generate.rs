//! Generate a private method for an exact call in its owning class.

use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};
use mago_names::resolver::NameResolver;
use mago_span::HasSpan;
use mago_syntax::cst::*;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::{CodeActionItem, Range, Server};

mod parameter_types;

pub(super) fn generate_method(server: &Server, file_id: FileId, start: u32, end: u32) -> Option<CodeActionItem> {
    let file = server.database().get(&file_id).ok()?;
    if file.file_type != FileType::Host || start > end || end > file.size {
        return None;
    }
    std::str::from_utf8(&file.contents).ok()?;
    let arena = LocalArena::new();
    let mut settings = server.parser_settings();
    settings.recover_incomplete = false;
    let program = parse_file_borrowed_with_settings(&arena, &file, settings);
    if program.has_errors() {
        return None;
    }
    let target = find_target(Node::Program(program), None, None, Range::new(start, end))?;
    let names = NameResolver::new(&arena).resolve(program);
    let class = server.codebase().get_class_like(names.resolve(&target.class.name.span.start)?)?;
    let name = target.name.value;
    if class.span != target.class.span()
        || !class.kind.is_class()
        || class.has_incomplete_hierarchy()
        || magic_name(name)
        || blocked(server, class, name)
        || name.eq_ignore_ascii_case(class.original_name.as_bytes().rsplit(|byte| *byte == b'\\').next()?)
    {
        return None;
    }
    if server.codebase().all_class_like_descendants.get(&class.name).into_iter().flatten().any(|descendant| {
        server.codebase().get_class_like_by_name(*descendant).is_none_or(|class| blocked(server, class, name))
    }) {
        return None;
    }
    let editor = server.editor_file(file_id);
    let parameters = parameter_names(target.arguments)?
        .into_iter()
        .zip(target.arguments.arguments.iter())
        .map(|(name, argument)| match parameter_types::hint(server, editor.as_deref(), argument.value()) {
            Some(hint) => format!("{hint} ${name}"),
            None => format!("${name}"),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let name = std::str::from_utf8(name).ok()?;
    let source = format!(
        "<?php class MagoStub {{ private {}function {name}({parameters}) {{ throw new \\LogicException('Not implemented.'); }} }}",
        if target.is_static { "static " } else { "" }
    );
    let stub_file = File::ephemeral(b"generated-method.php".as_slice().into(), source.into_bytes().into());
    let stub = super::refactor_stub::format_body(server, &arena, &stub_file, settings)?;
    super::refactor_implement::insert_methods(
        server,
        &file,
        program,
        Node::Class(target.class),
        &[stub],
        &format!("Generate method {name}"),
    )
}

fn blocked(server: &Server, class: &ClassLikeMetadata, name: &[u8]) -> bool {
    class.has_incomplete_hierarchy()
        || !class.mixins.is_empty()
        || server.codebase().get_declaring_method(class.name.as_bytes(), name).is_some()
        || [b"__call".as_slice(), b"__callStatic".as_slice()]
            .iter()
            .any(|magic| server.codebase().get_declaring_method(class.name.as_bytes(), magic).is_some())
        || class
            .pseudo_methods
            .iter()
            .chain(&class.static_pseudo_methods)
            .any(|method| method.as_bytes().eq_ignore_ascii_case(name))
        || class.all_parent_classes.iter().any(|parent| {
            [name, b"__call", b"__callStatic"]
                .iter()
                .any(|method| server.codebase().get_method(parent.as_bytes(), method).is_some())
        })
}

fn magic_name(name: &[u8]) -> bool {
    [
        "__construct",
        "__destruct",
        "__clone",
        "__call",
        "__callStatic",
        "__get",
        "__set",
        "__isset",
        "__unset",
        "__sleep",
        "__wakeup",
        "__serialize",
        "__unserialize",
        "__toString",
        "__invoke",
        "__set_state",
        "__debugInfo",
    ]
    .iter()
    .any(|magic| name.eq_ignore_ascii_case(magic.as_bytes()))
}

struct Target<'ast, 'arena> {
    class: &'ast Class<'arena>,
    name: &'ast LocalIdentifier<'arena>,
    arguments: &'ast ArgumentList<'arena>,
    is_static: bool,
}

fn find_target<'ast, 'arena>(
    node: Node<'ast, 'arena>,
    mut class: Option<&'ast Class<'arena>>,
    mut method: Option<&'ast Method<'arena>>,
    selected: Range,
) -> Option<Target<'ast, 'arena>> {
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
        _ => {}
    }
    let call = match node {
        Node::MethodCall(call) if matches!(call.object, Expression::Variable(Variable::Direct(variable)) if variable.name == b"$this") => {
            Some((&call.method, &call.argument_list, false))
        }
        Node::StaticMethodCall(call) if matches!(call.class, Expression::Self_(_)) => {
            Some((&call.method, &call.argument_list, true))
        }
        _ => None,
    };
    if let Some((ClassLikeMemberSelector::Identifier(name), arguments, is_static)) = call
        && name.start_offset() <= selected.start
        && selected.start < name.end_offset()
        && selected.end <= name.end_offset()
    {
        let method = method?;
        if method.is_abstract() || method.body.start_offset() > selected.start || (!is_static && method.is_static()) {
            return None;
        }
        return Some(Target { class: class?, name, arguments, is_static });
    }
    let mut target = None;
    node.visit_children(|child| {
        if target.is_none() {
            target = find_target(child, class, method, selected);
        }
    });
    target
}

fn parameter_names(arguments: &ArgumentList<'_>) -> Option<Vec<String>> {
    let mut used = HashSet::default();
    for argument in &arguments.arguments {
        if let Argument::Named(argument) = argument
            && (matches!(argument.name.value, b"this" | b"GLOBALS") || !used.insert(argument.name.value.to_vec()))
        {
            return None;
        }
    }
    let mut named = false;
    let mut suffix = 1;
    arguments
        .arguments
        .iter()
        .map(|argument| match argument {
            Argument::Named(argument) => {
                named = true;
                Some(std::str::from_utf8(argument.name.value).ok()?.to_owned())
            }
            Argument::Positional(argument) => {
                if named || argument.ellipsis.is_some() {
                    return None;
                }
                if let Expression::Variable(Variable::Direct(variable)) = argument.value.unparenthesized() {
                    let name = &variable.name[1..];
                    if !crate::variables::is_builtin_variable(name) && used.insert(name.to_vec()) {
                        return Some(std::str::from_utf8(name).ok()?.to_owned());
                    }
                }
                loop {
                    let name = format!("arg{suffix}");
                    suffix += 1;
                    if used.insert(name.as_bytes().to_vec()) {
                        return Some(name);
                    }
                }
            }
        })
        .collect()
}
