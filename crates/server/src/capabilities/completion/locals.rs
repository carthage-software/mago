//! Variable candidates and recovery for unfinished arrow functions.

use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::domain::CompletionEntry;
use crate::domain::CompletionKind;
use crate::domain::Range;
use crate::file_analysis::FileAnalysis;
use crate::semantic_index::ScopeKind;
use crate::semantic_index::SymbolKind;

use super::MAX_RESULTS;
use super::rank::compare_names;

const LOCAL: u8 = 0;
const THIS: u8 = 1;
const SUPERGLOBAL: u8 = 2;

struct Local<'a> {
    group: u8,
    order: u8,
    name: &'a [u8],
}

pub(super) fn variable_items(
    analysis: &FileAnalysis,
    tokens: &[Token<'_>],
    offset: u32,
    prefix: &[u8],
) -> Vec<CompletionEntry> {
    let index = analysis.semantic();
    let Some(scope) = index.scopes.iter().rposition(|scope| scope.range.contains(offset.saturating_sub(1))) else {
        return Vec::new();
    };
    let mut scopes = vec![scope as u32];
    let mut current = scope;
    while index.scopes[current].kind == ScopeKind::ArrowFunction {
        let Some(parent) = index.scopes[current].parent else { break };
        scopes.push(parent);
        current = parent as usize;
    }
    let mut seen = foldhash::HashSet::<&[u8]>::default();
    let mut found = Vec::new();
    let start = offset.saturating_sub(prefix.len() as u32 + 1);
    let end = tokens
        .iter()
        .find(|token| token.start.offset == start && token.kind == TokenKind::Variable)
        .map_or(offset, |token| token.start.offset + token.value.len() as u32);
    let replace = Range::new(start, end);
    let arrow = incomplete_arrow_header(tokens, offset);
    let binds_this = crate::call_analysis::binds_this(analysis, offset.saturating_sub(1))
        && arrow.as_ref().is_none_or(|arrow| !arrow.is_static);
    for (order, name) in crate::variables::BUILTIN_VARIABLES.iter().enumerate() {
        if (*name == "this" && !binds_this) || !name.as_bytes().starts_with(prefix) {
            continue;
        }
        seen.insert(name.as_bytes());
        found.push(Local {
            group: if *name == "this" { THIS } else { SUPERGLOBAL },
            order: order as u8,
            name: name.as_bytes(),
        });
    }
    for name in arrow.into_iter().flat_map(|arrow| arrow.parameters) {
        if name.starts_with(prefix) && seen.insert(name) {
            found.push(Local { group: LOCAL, order: 0, name });
        }
    }
    for occurrence in &index.occurrences {
        if occurrence.range.end >= offset {
            continue;
        }
        let symbol = &index.symbols[occurrence.symbol as usize];
        if symbol.kind != SymbolKind::Variable || !symbol.name.starts_with(prefix) {
            continue;
        }
        let local = symbol.scope.is_some_and(|scope| scopes.contains(&scope));
        let captured = !local
            && index.scopes[scope].range.contains(occurrence.range.start)
            && index.scopes.iter().rposition(|scope| scope.range.contains(occurrence.range.start)) == Some(scope);
        if (!local && !captured) || !seen.insert(&symbol.name) {
            continue;
        }
        found.push(Local { group: LOCAL, order: 0, name: &symbol.name });
    }

    found.sort_unstable_by(|left, right| {
        (left.name != prefix)
            .cmp(&(right.name != prefix))
            .then(left.group.cmp(&right.group))
            .then(left.order.cmp(&right.order))
            .then_with(|| compare_names(left.name, right.name))
    });
    found.truncate(MAX_RESULTS);
    let preselect = found.first().is_some_and(|local| local.name == prefix);
    let mut out: Vec<_> = found
        .into_iter()
        .map(|local| CompletionEntry {
            replace: Some(replace),
            ..CompletionEntry::new(format!("${}", String::from_utf8_lossy(local.name)), CompletionKind::Variable)
        })
        .collect();
    if let Some(first) = out.first_mut() {
        first.preselect = preselect;
    }
    out
}

struct ArrowHeader<'a> {
    parameters: Vec<&'a [u8]>,
    is_static: bool,
}

/// A missing first expression makes the parser drop the whole arrow function.
/// Recover only a complete header followed directly by the variable being typed.
fn incomplete_arrow_header<'a>(tokens: &[Token<'a>], offset: u32) -> Option<ArrowHeader<'a>> {
    let current =
        tokens.iter().rposition(|token| token.start.offset < offset && !crate::lookup::is_trivia(token.kind))?;
    let arrow = tokens[..current].iter().rposition(|token| !crate::lookup::is_trivia(token.kind))?;
    if tokens[arrow].value != b"=>" || !matches!(tokens[current].kind, TokenKind::Dollar | TokenKind::Variable) {
        return None;
    }
    let start = tokens[..arrow].iter().rposition(|token| token.kind == TokenKind::Fn)?;
    let mut depth = 0u32;
    let mut closed = false;
    let mut names = Vec::new();
    for token in tokens[start + 1..arrow].iter().filter(|token| !crate::lookup::is_trivia(token.kind)) {
        if matches!(token.value, b";" | b"{" | b"}" | b"=>") {
            return None;
        }
        match token.value {
            b"(" if !closed => depth += 1,
            b")" if !closed => {
                let next = depth.checked_sub(1)?;
                depth = next;
                closed = depth == 0;
            }
            _ if !closed && depth == 1 && token.kind == TokenKind::Variable => names.push(&token.value[1..]),
            _ => {}
        }
    }
    closed.then(|| ArrowHeader {
        parameters: names,
        is_static: tokens[..start]
            .iter()
            .rfind(|token| !crate::lookup::is_trivia(token.kind))
            .is_some_and(|token| token.kind == TokenKind::Static),
    })
}
