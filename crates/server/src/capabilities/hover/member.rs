//! Members and constants whose descriptions use declaration source.

use std::fmt::Write;

use mago_analyzer::artifacts::editor::MemberKind;
use mago_bytes::BytesDisplay;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::constant::ConstantMetadata;
use mago_codex::metadata::ttype::TypeMetadata;
use mago_codex::visibility::Visibility;
use mago_database::DatabaseReader;
use mago_span::Span;
use mago_syntax::lexer::Lexer;
use mago_syntax::token::TokenKind;
use mago_syntax_core::input::Input;
use mago_word::Word;

use crate::Server;
use crate::documentation;
use crate::domain::HoverEntry;
use crate::lookup::{DocblockAnchor, is_trivia};

use super::types::{display_union, short_name};

pub(super) fn describe(server: &Server, class: Word, name: Word, kind: MemberKind, magic: bool) -> Option<HoverEntry> {
    let class = server.codebase().get_class_like_by_name(class)?;
    match kind {
        MemberKind::Property => property(server, class, name, magic),
        MemberKind::ClassConstant => class_constant(server, class, name),
        MemberKind::EnumCase => enum_case(server, class, name),
    }
}

fn property(server: &Server, class: &ClassLikeMetadata, name: Word, magic: bool) -> Option<HoverEntry> {
    let codebase = server.codebase();
    let property = if magic {
        codebase.get_declaring_magic_property(class.name.as_bytes(), name.as_bytes())
    } else {
        codebase.get_declaring_property(class.name.as_bytes(), name.as_bytes())
    }?;
    let declaring = if magic { &class.magic_property_ids } else { &class.declaring_property_ids };
    let owner = declaring
        .get(&name)
        .and_then(|name| codebase.get_class_like_by_name(*name))
        .map_or(class.original_name, |class| class.original_name);
    let default_write = if property.flags.is_readonly() {
        property.read_visibility.max(Visibility::Protected)
    } else {
        property.read_visibility
    };
    let setter = if property.write_visibility > default_write {
        format!("{}(set) ", property.write_visibility)
    } else {
        String::new()
    };
    let mut declaration = format!(
        "{}{}{} {setter}{}{}{}{}",
        if property.flags.is_final() { "final " } else { "" },
        if property.flags.is_abstract() { "abstract " } else { "" },
        property.read_visibility,
        if property.flags.is_static() { "static " } else { "" },
        if property.flags.is_readonly() { "readonly " } else { "" },
        property
            .type_metadata
            .as_ref()
            .map_or_else(String::new, |ty| { format!("{} ", display_union(codebase, &ty.type_union)) }),
        BytesDisplay(name.as_bytes()),
    );
    if !property.hooks.is_empty() {
        let mut hooks: Vec<_> = property.hooks.values().collect();
        hooks.sort_unstable_by_key(|hook| hook.name);
        declaration.push_str(" { ");
        for hook in hooks {
            let _ = write!(
                declaration,
                "{}{}{}; ",
                if hook.flags.is_final() { "final " } else { "" },
                if hook.returns_by_ref { "&" } else { "" },
                BytesDisplay(hook.name.as_bytes()),
            );
        }
        declaration.push('}');
    }
    let mut entry = HoverEntry::in_class(owner.as_bytes(), declaration);
    entry.documentation = if magic {
        documentation::for_magic_property(server, owner, property)
    } else {
        documentation::for_property(server, owner, property)
    };
    Some(entry)
}

fn class_constant(server: &Server, class: &ClassLikeMetadata, name: Word) -> Option<HoverEntry> {
    let constant = class.constants.get(&name)?;
    let owner = constant_owner(server.codebase(), constant.span, name).unwrap_or(class.original_name);
    let mut declaration =
        format!("{}{} const ", if constant.flags.is_final() { "final " } else { "" }, constant.visibility);
    if let Some(hint) = &constant.type_declaration {
        declaration.push_str(&declared_type(server, hint));
        declaration.push(' ');
    }
    let _ = write!(declaration, "{}", BytesDisplay(name.as_bytes()));
    append_value(&mut declaration, server, constant.span, name.as_bytes());
    let mut entry = HoverEntry::in_class(owner.as_bytes(), declaration);
    entry.documentation = documentation::for_declaration(server, constant.span, DocblockAnchor::DeclarationName);
    Some(entry)
}

fn enum_case(server: &Server, class: &ClassLikeMetadata, name: Word) -> Option<HoverEntry> {
    let case = class.enum_cases.get(&name)?;
    let mut declaration = format!("case {}", BytesDisplay(name.as_bytes()));
    append_value(&mut declaration, server, Span { start: case.name_span.start, ..case.span }, name.as_bytes());
    let mut entry = HoverEntry::in_class(class.original_name.as_bytes(), declaration);
    entry.documentation = documentation::for_declaration(server, case.span, DocblockAnchor::DeclarationStart);
    Some(entry)
}

pub(super) fn constant(server: &Server, meta: &ConstantMetadata) -> HoverEntry {
    let name = meta.name.as_bytes();
    let mut declaration = format!("const {}", short_name(name));
    append_value(&mut declaration, server, meta.span, name.rsplit(|byte| *byte == b'\\').next().unwrap_or(name));
    let mut entry = HoverEntry::for_symbol(name, declaration);
    entry.documentation = documentation::for_declaration(server, meta.span, DocblockAnchor::DeclarationName);
    entry
}

/// Constants retain their source span when inherited. Search only that file's declarations.
fn constant_owner(codebase: &CodebaseMetadata, span: Span, name: Word) -> Option<Word> {
    codebase
        .file_signatures
        .get(&span.file_id)?
        .ast_nodes
        .iter()
        .filter(|node| !node.is_function)
        .filter_map(|node| codebase.get_class_like_by_name(node.name))
        .filter(|class| {
            class.span.file_id == span.file_id
                && class.span.start.offset <= span.start.offset
                && span.end.offset <= class.span.end.offset
                && class.constants.get(&name).is_some_and(|constant| constant.span == span)
        })
        .min_by_key(|class| class.span.length())
        .map(|class| class.original_name)
}

fn declared_type(server: &Server, hint: &TypeMetadata) -> String {
    server
        .database()
        .get(&hint.span.file_id)
        .ok()
        .and_then(|file| {
            file.contents
                .get(hint.span.start.offset as usize..hint.span.end.offset as usize)
                .filter(|text| !text.is_empty())
                .map(|text| BytesDisplay(text).to_string())
        })
        .unwrap_or_else(|| display_union(server.codebase(), &hint.type_union))
}

fn append_value(declaration: &mut String, server: &Server, span: Span, name: &[u8]) {
    if let Some(value) = assigned_value(server, span, name) {
        declaration.push(' ');
        declaration.push_str(&value);
    }
}

/// Only copy a verified `NAME = value` source span; never turn an inferred type into PHP code.
fn assigned_value(server: &Server, span: Span, name: &[u8]) -> Option<String> {
    let file = server.database().get(&span.file_id).ok()?;
    let bytes = file.contents.get(span.start.offset as usize..span.end.offset as usize)?;
    let mut lexer = Lexer::scripting(Input::new(span.file_id, bytes), server.parser_settings().lexer);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.advance() {
        let token = token.ok()?;
        if !is_trivia(token.kind) {
            tokens.push(token);
        }
    }
    let [identifier, equals, rest @ ..] = tokens.as_slice() else { return None };
    if identifier.value != name || equals.kind != TokenKind::Equal {
        return None;
    }
    let last = rest.last()?;
    let end = if last.kind == TokenKind::Semicolon {
        if rest.len() == 1 {
            return None;
        }
        last.start.offset as usize
    } else {
        bytes.len()
    };
    Some(BytesDisplay(bytes[equals.start.offset as usize..end].trim_ascii_end()).to_string())
}
