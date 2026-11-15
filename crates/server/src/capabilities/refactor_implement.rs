//! Insert inherited method stubs without changing existing code.

use foldhash::HashMap;
use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};
use mago_names::resolver::NameResolver;
use mago_span::HasSpan;
use mago_syntax::cst::{Node, Program};
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::{CodeActionItem, CodeActionKind, Range, Server, TextReplacement};

pub(super) fn implement_methods(server: &Server, file_id: FileId, start: u32, end: u32) -> Option<CodeActionItem> {
    let file = server.database().get(&file_id).ok()?;
    if file.file_type != FileType::Host || start > end || end > file.size {
        return None;
    }
    std::str::from_utf8(&file.contents).ok()?;
    let arena = LocalArena::new();
    let program = parse_file_borrowed_with_settings(&arena, &file, server.parser_settings());
    if program.has_errors() {
        return None;
    }
    let names = NameResolver::new(&arena).resolve(program);
    let mut nodes = vec![Node::Program(program)];
    while let Some(node) = nodes.pop() {
        let selected = match node {
            Node::Class(class) => Some((class.name.span, class.left_brace)),
            Node::Enum(enumeration) => Some((enumeration.name.span, enumeration.left_brace)),
            _ => None,
        };
        if let Some((name, left)) = selected
            && node.start_offset() <= start
            && end <= left.start.offset
        {
            let name = names.resolve(&name.start)?;
            let metadata = server.codebase().get_class_like(name)?;
            if metadata.span != node.span() {
                return None;
            }
            let stubs = server.get_method_stubs(name)?;
            return insert_methods(server, &file, program, node, &stubs, "Implement inherited methods");
        }
        node.visit_children(|child| nodes.push(child));
    }
    None
}

pub(super) fn insert_methods(
    server: &Server,
    file: &File,
    program: &Program<'_>,
    class_node: Node<'_, '_>,
    stubs: &[String],
    title: &str,
) -> Option<CodeActionItem> {
    if stubs.is_empty() || program.has_errors() || file.file_type != FileType::Host {
        return None;
    }
    let (left, right, members) = match class_node {
        Node::Class(class) => (class.left_brace, class.right_brace, &class.members),
        Node::Enum(enumeration) => (enumeration.left_brace, enumeration.right_brace, &enumeration.members),
        _ => return None,
    };
    let source = std::str::from_utf8(&file.contents).ok()?;
    let last_member_end = members.last().map_or(left.end.offset, HasSpan::end_offset);
    // An orphan PHPDoc could become the new method's documentation.
    if program.trivia.iter().any(|trivia| {
        trivia.kind.is_docblock()
            && last_member_end <= trivia.span.start.offset
            && trivia.span.end.offset <= right.start.offset
    }) {
        return None;
    }
    let newline = if source.contains("\r\n") { "\r\n" } else { "\n" };
    let header_start = class_node.start_offset() as usize;
    let header_line = source.as_bytes()[..header_start].iter().rposition(|byte| *byte == b'\n').map_or(0, |i| i + 1);
    let indent_len =
        source.as_bytes()[header_line..header_start].iter().take_while(|byte| matches!(byte, b' ' | b'\t')).count();
    let indent = &source[header_line..header_line + indent_len];
    let close = right.start.offset as usize;
    let close_line = source.as_bytes()[..close].iter().rposition(|byte| *byte == b'\n').map_or(0, |i| i + 1);
    let own_line = source.as_bytes()[close_line..close].iter().all(|byte| matches!(byte, b' ' | b'\t'));
    let offset = if own_line { close_line } else { close };
    let mut new_text = String::new();
    if !own_line || (!members.is_empty() && !source[..offset].ends_with(&format!("{newline}{newline}"))) {
        new_text.push_str(newline);
    }
    for (index, stub) in stubs.iter().enumerate() {
        if index != 0 {
            new_text.push_str(newline);
        }
        for line in stub.lines() {
            if !line.is_empty() {
                new_text.push_str(indent);
            }
            new_text.push_str(line);
            new_text.push_str(newline);
        }
    }
    if !own_line {
        new_text.push_str(indent);
    }
    let updated = format!("{}{}{}", &source[..offset], new_text, &source[offset..]);
    let changed = File::ephemeral(b"method-stubs.php".as_slice().into(), updated.into_bytes().into());
    let arena = LocalArena::new();
    if parse_file_borrowed_with_settings(&arena, &changed, server.parser_settings()).has_errors() {
        return None;
    }
    let edit = TextReplacement { range: Range::new(offset as u32, offset as u32), new_text };
    Some(CodeActionItem {
        title: title.to_owned(),
        kind: CodeActionKind::QuickFix,
        edits: HashMap::from_iter([(file.id, vec![edit])]),
        diagnostic: None,
    })
}
