//! Declaration templates at statement boundaries, including unfinished source.

use mago_php_version::PHPVersion;
use mago_syntax::token::{Token, TokenKind};

use crate::domain::{CompletionEntry, CompletionInsert, CompletionKind, Range};
use crate::lookup;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    Statement,
    Class,
    Interface,
    Enum,
    BackedEnum,
}

struct DeclarationSite {
    prefix: String,
    replace: Range,
    body: Body,
}

pub(super) fn complete(tokens: &[Token<'_>], offset: u32, version: PHPVersion) -> Vec<CompletionEntry> {
    let mut items = Vec::new();
    visit_declarations(tokens, offset, version, |site, header, label, tail| {
        site.push(&mut items, header, label, tail);
    });
    items
}

pub(super) fn has_declaration(tokens: &[Token<'_>], offset: u32, version: PHPVersion) -> bool {
    let mut found = false;
    visit_declarations(tokens, offset, version, |_, _, _, _| found = true);
    found
}

fn visit_declarations(
    tokens: &[Token<'_>],
    offset: u32,
    version: PHPVersion,
    mut visit: impl FnMut(&DeclarationSite, &str, &str, &str),
) {
    let Some(site) = declaration_site(tokens, offset) else { return };
    let mut push = |header: &str, label: &str, tail: &str| {
        if header.starts_with(&site.prefix) {
            visit(&site, header, label, tail);
        }
    };
    if site.body == Body::Statement {
        for header in ["class", "final class", "abstract class", "interface", "trait"] {
            push(header, header, " ${1:Name}\n{\n    $0\n}");
        }
        if version >= PHPVersion::PHP81 {
            push("enum", "enum", " ${1:Name}\n{\n    $0\n}");
        }
        if version >= PHPVersion::PHP82 {
            for header in ["readonly class", "final readonly class", "readonly final class"] {
                push(header, header, " ${1:Name}\n{\n    $0\n}");
            }
        }
        push("function", "function", " ${1:name}(${2}): ${3:void}\n{\n    $0\n}");
        push("const", "const", " ${1:NAME} = ${2:null};$0");
        return;
    }
    let tail = if site.body == Body::Interface {
        " ${1:name}(${2}): ${3:void};$0"
    } else {
        " ${1:name}(${2}): ${3:void}\n{\n    $0\n}"
    };
    push("function", "function", tail);
    push("const", "const", " ${1:NAME} = ${2:null};$0");
    for visibility in ["public", "protected", "private"] {
        if site.body == Body::Interface && visibility != "public" {
            continue;
        }
        for modifier in ["", "static "] {
            let header = format!("{visibility} {modifier}function");
            push(&header, &header, tail);
        }
        let header = format!("{visibility} const");
        push(&header, &header, " ${1:NAME} = ${2:null};$0");
        if site.body == Body::Class {
            push(visibility, &format!("{visibility} property"), " ${1:mixed} \\$${2:name};$0");
            let header = format!("{visibility} static");
            push(&header, &format!("{header} property"), " ${1:mixed} \\$${2:name};$0");
            if version >= PHPVersion::PHP81 {
                let header = format!("{visibility} readonly");
                push(&header, &format!("{header} property"), " ${1:mixed} \\$${2:name};$0");
            }
        }
    }
    if site.body == Body::Class {
        for visibility in ["public", "protected"] {
            for header in [format!("final {visibility} function"), format!("{visibility} final function")] {
                push(&header, &header, tail);
            }
            for header in [format!("abstract {visibility} function"), format!("{visibility} abstract function")] {
                push(&header, &header, " ${1:name}(${2}): ${3:void};$0");
            }
        }
    }
    match site.body {
        Body::Enum => push("case", "case", " ${1:Name};$0"),
        Body::BackedEnum => push("case", "case", " ${1:Name} = ${2:value};$0"),
        _ => {}
    }
}

impl DeclarationSite {
    fn push(&self, items: &mut Vec<CompletionEntry>, header: &str, label: &str, tail: &str) {
        items.push(CompletionEntry {
            insert: Some(CompletionInsert::Snippet(format!("{header}{tail}"))),
            replace: Some(self.replace),
            detail: Some("PHP declaration".into()),
            filter_text: Some(header.into()),
            ..CompletionEntry::new(label.into(), CompletionKind::Snippet)
        });
    }
}

pub(super) fn in_enum_body(tokens: &[Token<'_>], offset: u32) -> bool {
    declaration_context(tokens, offset).is_some_and(|(_, body)| matches!(body, Body::Enum | Body::BackedEnum))
}

pub(super) fn in_class_body(tokens: &[Token<'_>], offset: u32) -> bool {
    declaration_context(tokens, offset).is_some_and(|(_, body)| body != Body::Statement)
}

fn declaration_context(tokens: &[Token<'_>], offset: u32) -> Option<(usize, Body)> {
    let mut bodies = Vec::new();
    let mut delimiters = 0u32;
    let mut attribute_depth = None;
    let mut pending = Body::Statement;
    let mut previous = None;
    let mut start = 0;
    for (index, token) in tokens.iter().enumerate().take_while(|(_, token)| token.start.offset < offset) {
        if lookup::is_trivia(token.kind) {
            continue;
        }
        match token.kind {
            TokenKind::HashLeftBracket => {
                if tokens[start..index].iter().all(|token| lookup::is_trivia(token.kind)) {
                    attribute_depth = Some(delimiters);
                }
                delimiters += 1;
            }
            TokenKind::LeftParenthesis | TokenKind::LeftBracket => delimiters += 1,
            TokenKind::RightParenthesis => delimiters = delimiters.saturating_sub(1),
            TokenKind::RightBracket => {
                delimiters = delimiters.saturating_sub(1);
                if attribute_depth == Some(delimiters) {
                    start = index + 1;
                    attribute_depth = None;
                }
            }
            TokenKind::Class | TokenKind::Trait if previous != Some(TokenKind::ColonColon) => pending = Body::Class,
            TokenKind::Interface => pending = Body::Interface,
            TokenKind::Enum => pending = Body::Enum,
            TokenKind::Colon if pending == Body::Enum => pending = Body::BackedEnum,
            TokenKind::Function => pending = Body::Statement,
            TokenKind::LeftBrace => {
                bodies.push((pending, delimiters));
                pending = Body::Statement;
                start = index + 1;
            }
            TokenKind::RightBrace => {
                bodies.pop();
                start = index + 1;
            }
            TokenKind::Semicolon | TokenKind::OpenTag | TokenKind::ShortOpenTag => {
                pending = Body::Statement;
                start = index + 1;
            }
            _ => {}
        }
        previous = Some(token.kind);
    }
    let (body, body_depth) = bodies.last().copied().unwrap_or((Body::Statement, 0));
    (delimiters == body_depth).then_some((start, body))
}

fn declaration_site(tokens: &[Token<'_>], offset: u32) -> Option<DeclarationSite> {
    let (start, body) = declaration_context(tokens, offset)?;
    let words: Vec<_> = tokens[start..]
        .iter()
        .take_while(|token| token.start.offset < offset)
        .filter(|token| !lookup::is_trivia(token.kind))
        .collect();
    if words.is_empty() || words.iter().any(|token| !token.kind.is_identifier_maybe_reserved()) {
        return None;
    }
    let prefix = words
        .iter()
        .map(|token| {
            let end = (offset - token.start.offset).min(token.value.len() as u32) as usize;
            String::from_utf8_lossy(&token.value[..end]).to_ascii_lowercase()
        })
        .collect::<Vec<_>>()
        .join(" ");
    let first = words[0];
    let last = words[words.len() - 1];
    let replace = Range::new(first.start.offset, offset.max(last.start.offset + last.value.len() as u32));
    if tokens.iter().find(|token| token.start.offset >= replace.end && !lookup::is_trivia(token.kind)).is_some_and(
        |token| {
            token.kind.is_identifier_maybe_reserved()
                || matches!(token.kind, TokenKind::LeftBrace | TokenKind::LeftParenthesis)
        },
    ) {
        return None;
    }
    Some(DeclarationSite { prefix, replace, body })
}
