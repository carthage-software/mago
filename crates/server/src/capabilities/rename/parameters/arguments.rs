//! Bind argument labels to parameter declarations before making edits.

use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};
use mago_syntax::token::{Token, TokenKind};

use crate::call_analysis::cursor::CallCursor;
use crate::call_analysis::targets::resolve_signature_targets;
use crate::capabilities::rename::STALE;
use crate::lookup;
use crate::{Range, RefusalReason, RenameRefusal, Server, SymbolLocation};

use super::{Parameter, parameter_for_call};

pub(super) fn selected(
    server: &mut Server,
    file: FileId,
    offset: u32,
) -> Result<Option<(Parameter, Range)>, RenameRefusal> {
    let file = server.database().get(&file).map_err(|_| STALE)?;
    let tokens = tokens(server, &file);
    for call in labels(&tokens) {
        let label = tokens[call.argument_start];
        let range = Range::new(label.start.offset, label.start.offset + label.value.len() as u32);
        if !range.contains(offset) {
            continue;
        }
        let unresolved = RenameRefusal::at(RefusalReason::Unresolved, file.id, range);
        let targets = resolve_signature_targets(server, file.id, &tokens, &call);
        let first = targets.first().ok_or(unresolved)?;
        let parameter = parameter_for_call(server, first, label.value).ok_or(unresolved)?;
        if targets.iter().any(|target| target.uncertain || parameter_for_call(server, target, label.value).is_none()) {
            return Err(unresolved);
        }
        return Ok(Some((parameter, range)));
    }
    Ok(None)
}

pub(super) fn collect(
    server: &mut Server,
    parameters: &[Parameter],
    new_name: Option<&str>,
    locations: &mut Vec<SymbolLocation>,
) -> Result<(), RenameRefusal> {
    if parameters.iter().all(|parameter| parameter.variadic) {
        return Ok(());
    }
    let files: Vec<_> = server.database().files().filter(|file| file.file_type == FileType::Host).collect();
    for file in files {
        // Most files have no argument labels; keep them out of the token cache.
        if !file.contents.contains(&b':') {
            continue;
        }
        let tokens = tokens(server, &file);
        for call in labels(&tokens) {
            let label = tokens[call.argument_start];
            let range = Range::new(label.start.offset, label.start.offset + label.value.len() as u32);
            let matches_name = parameters.iter().any(|parameter| !parameter.variadic && parameter.name == label.value);
            let captures_name = new_name.is_some_and(|name| name.as_bytes() == label.value);
            if !matches_name && !captures_name {
                continue;
            }
            let targets = resolve_signature_targets(server, file.id, &tokens, &call);
            // Keep known origins even when a partial's remaining names disagree.
            // Skipping such a call could leave an affected argument unchanged.
            if targets.iter().any(|target| {
                (target.uncertain || target.signature(server.codebase()).is_none())
                    && parameters.iter().any(|parameter| parameter.callable == target.key)
            }) {
                return Err(RenameRefusal::at(RefusalReason::Unresolved, file.id, range));
            }
            if !matches_name {
                if targets.iter().any(|target| {
                    server.codebase().get_function_like(&target.key).is_some_and(|function| {
                        parameters.iter().any(|parameter| !parameter.variadic && parameter.function == function.span)
                    })
                }) {
                    return Err(RenameRefusal::at(RefusalReason::NamedArgTaken, file.id, range));
                }
                continue;
            }
            let bindings: Vec<_> =
                targets.iter().map(|target| parameter_for_call(server, target, label.value)).collect();
            let affected = |binding: &Option<Parameter>| {
                binding.as_ref().is_some_and(|binding| {
                    parameters.iter().any(|parameter| parameter.declaration == binding.declaration)
                })
            };
            if !bindings.iter().any(affected) {
                continue;
            }
            if targets.iter().any(|target| target.uncertain) || !bindings.iter().all(affected) {
                return Err(RenameRefusal::at(RefusalReason::Unresolved, file.id, range));
            }
            locations.push(SymbolLocation { file: file.id, range });
        }
    }
    Ok(())
}

fn tokens<'file>(server: &Server, file: &'file File) -> Vec<Token<'file>> {
    server.lex(file).into_iter().filter(|token| !lookup::is_trivia(token.kind)).collect()
}

/// Walk delimiter frames once. A colon is a label only at an argument's start,
/// so ternaries, return types, array keys and statement labels stay untouched.
fn labels<'a>(tokens: &[Token<'a>]) -> Vec<CallCursor<'a>> {
    let mut frames: Vec<(TokenKind, usize, usize, u32)> = Vec::new();
    let mut labels = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LeftParenthesis
            | TokenKind::LeftBracket
            | TokenKind::HashLeftBracket
            | TokenKind::LeftBrace
            | TokenKind::DollarLeftBrace => {
                frames.push((token.kind, index, index + 1, 0));
            }
            TokenKind::RightParenthesis | TokenKind::RightBracket | TokenKind::RightBrace => {
                frames.pop();
            }
            TokenKind::Comma => {
                if let Some((_, _, start, argument)) = frames.last_mut() {
                    *start = index + 1;
                    *argument += 1;
                }
            }
            TokenKind::Colon => {
                let Some(&(TokenKind::LeftParenthesis, open, start, argument)) = frames.last() else { continue };
                if open == 0 || index != start + 1 || !tokens[start].kind.is_identifier_maybe_reserved() {
                    continue;
                }
                let attribute = frames.len() >= 2 && frames[frames.len() - 2].0 == TokenKind::HashLeftBracket;
                labels.push(CallCursor {
                    name: open - 1,
                    open,
                    argument_start: start,
                    argument,
                    argument_name: Some(tokens[start].value),
                    attribute,
                });
            }
            _ => {}
        }
    }
    labels
}
