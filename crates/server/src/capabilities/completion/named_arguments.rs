//! Parameter names at an empty argument slot or within its label.

use mago_bytes::BytesDisplay;
use mago_database::file::FileId;
use mago_php_version::feature::Feature;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::Server;
use crate::call_analysis::cursor::{CallCursor, CallEnd, argument_ranges, enclosing_call};
use crate::call_analysis::signature::{CallParameter, CallSignature};
use crate::call_analysis::targets::{SignatureTarget, resolve_signature_targets};
use crate::domain::{
    CompletionAlternative, CompletionDocumentation, CompletionEntry, CompletionInsert, CompletionKind, HoverEntry,
    Range,
};
use crate::lookup;
use crate::matcher;

use super::items;
use super::rank::Rank;

#[derive(Default)]
pub(super) struct ArgumentCompletions {
    pub items: Vec<CompletionEntry>,
    /// An existing label excludes expressions even when no target or safe name is known.
    pub labels_only: bool,
}

pub(super) fn complete(server: &mut Server, file_id: FileId, lexed: &[Token<'_>], offset: u32) -> ArgumentCompletions {
    let tokens: Vec<_> = lexed.iter().copied().filter(|token| !lookup::is_trivia(token.kind)).collect();
    let Some(call) = enclosing_call(&tokens, offset) else { return ArgumentCompletions::default() };
    let mut result = ArgumentCompletions {
        labels_only: tokens.get(call.argument_start).is_some_and(|token| {
            token.kind.is_identifier_maybe_reserved()
                && token.start.offset <= offset
                && offset <= token.start.offset + token.value.len() as u32
                && tokens.get(call.argument_start + 1).is_some_and(|next| next.kind == TokenKind::Colon)
        }),
        items: Vec::new(),
    };
    if !server.php_version.is_supported(Feature::NamedArguments) {
        return result;
    }
    let Some(slot) = argument_slot(&tokens, lexed, &call, offset) else { return result };
    let targets = resolve_signature_targets(server, file_id, &tokens, &call);
    if targets.is_empty() || targets.iter().any(|target| target.uncertain) {
        return result;
    }
    let signatures: Vec<_> = targets.iter().filter_map(|target| target.signature(server.codebase())).collect();
    if signatures.len() != targets.len()
        || signatures.iter().any(|signature| {
            let meta = signature.function;
            meta.flags.forbids_named_arguments()
                || meta.flags.is_magic_method()
                || !meta.is_available_in_version(server.php_version)
        })
    {
        return result;
    }
    let mut scored = Vec::new();
    for (index, parameter) in signatures[0].parameters().skip(slot.positional).enumerate() {
        let Some(name) = parameter.declaration.name.0.as_bytes().strip_prefix(b"$") else { continue };
        if parameter.flags().is_variadic()
            || slot.named.contains(&name)
            || !signatures.iter().all(|signature| {
                signature.parameters().skip(slot.positional).any(|parameter| {
                    !parameter.flags().is_variadic()
                        && parameter.declaration.name.0.as_bytes().strip_prefix(b"$") == Some(name)
                })
            })
        {
            continue;
        }
        let Some(matched) = matcher::score(slot.prefix, name) else { continue };
        let alternatives = (signatures.len() > 1).then(|| {
            targets
                .iter()
                .zip(&signatures)
                .filter_map(|(target, signature)| {
                    let parameter = signature.parameter(name)?;
                    Some(parameter_alternative(server, target, *signature, parameter))
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        });
        let name = String::from_utf8_lossy(name);
        let label = format!("{name}:");
        let insert = if slot.colon { name.into_owned() } else { format!("{name}: ") };
        let detail = alternatives.as_ref().and_then(|alternatives| alternatives.first()).map_or_else(
            || parameter_detail(server, parameter),
            |alternative| alternative.description.declaration.clone(),
        );
        scored.push((
            Rank { proximity: u16::try_from(index).unwrap_or(u16::MAX), ..Rank::new(matched) },
            CompletionEntry {
                detail: Some(detail),
                documentation_source: Some(CompletionDocumentation::Parameter {
                    callable: targets[0].key,
                    name: parameter.declaration.name.0,
                }),
                member_alternatives: alternatives,
                insert: Some(CompletionInsert::Text(insert)),
                replace: Some(slot.replace),
                ..CompletionEntry::new(label, CompletionKind::Variable)
            },
        ));
    }
    result.items = items::finalize(scored);
    result
}

fn parameter_detail(server: &Server, parameter: CallParameter<'_>) -> String {
    let mut detail = String::new();
    let displayed_type = parameter.qualified_type_name(server.codebase());
    super::super::signature::append_parameter_with_flags(
        &mut detail,
        server.database(),
        parameter.declaration,
        displayed_type.as_deref(),
        parameter.flags(),
    );
    detail
}

fn parameter_alternative(
    server: &Server,
    target: &SignatureTarget,
    signature: CallSignature<'_>,
    parameter: CallParameter<'_>,
) -> CompletionAlternative {
    let function = signature.function;
    let owner = target
        .key
        .as_method_identifier()
        .and_then(|method| server.codebase().get_class_like_by_name(method.get_class_name()))
        .map_or_else(
            || {
                if function.kind.is_closure() {
                    "closure".into()
                } else if function.kind.is_arrow_function() {
                    "arrow function".into()
                } else {
                    BytesDisplay(function.original_name.as_bytes()).to_string()
                }
            },
            |class| {
                format!(
                    "{}::{}",
                    BytesDisplay(class.original_name.as_bytes()),
                    BytesDisplay(function.original_name.as_bytes())
                )
            },
        );
    CompletionAlternative {
        description: HoverEntry {
            scope: Some(owner),
            declaration: parameter_detail(server, parameter),
            documentation: None,
        },
        documentation_source: Some(CompletionDocumentation::Parameter {
            callable: target.key,
            name: parameter.declaration.name.0,
        }),
    }
}

struct ArgumentSlot<'a> {
    prefix: &'a [u8],
    replace: Range,
    colon: bool,
    positional: usize,
    named: Vec<&'a [u8]>,
}

fn argument_slot<'a>(
    tokens: &[Token<'a>],
    lexed: &[Token<'_>],
    call: &CallCursor<'_>,
    offset: u32,
) -> Option<ArgumentSlot<'a>> {
    let (mut ranges, end) = argument_ranges(tokens, call.open)?;
    if end == CallEnd::Boundary {
        let line_end = line_end(lexed, offset);
        let boundary = tokens.partition_point(|token| token.start.offset < line_end);
        ranges.retain(|range| range.start < boundary || range.start == call.argument_start);
        for range in &mut ranges {
            range.end = range.end.min(boundary).max(range.start);
        }
    }
    let active = ranges.iter().find(|range| range.start == call.argument_start)?;
    let mut slot = ArgumentSlot {
        prefix: b"",
        replace: Range::new(offset, offset),
        colon: false,
        positional: 0,
        named: Vec::new(),
    };
    if let Some(token) = tokens[active.clone()].first() {
        let end = token.start.offset + token.value.len() as u32;
        if !token.kind.is_identifier_maybe_reserved() || offset < token.start.offset || offset > end {
            return None;
        }
        slot.prefix = &token.value[..(offset - token.start.offset) as usize];
        slot.replace = Range::new(token.start.offset, end);
        slot.colon = active.len() > 1 && tokens[active.start + 1].kind == TokenKind::Colon;
        if !slot.colon && active.len() != 1 {
            return None;
        }
    }
    let mut has_named = false;
    for range in &ranges {
        if range == active {
            continue;
        }
        let Some(first) = tokens[range.clone()].first() else {
            // A trailing comma is valid; an empty argument before another is not.
            if range.end != ranges.last()?.end {
                return None;
            }
            continue;
        };
        if first.kind == TokenKind::DotDotDot {
            return None;
        }
        if first.kind.is_identifier_maybe_reserved()
            && tokens.get(range.start + 1).is_some_and(|token| token.kind == TokenKind::Colon)
        {
            if slot.named.contains(&first.value) {
                return None;
            }
            slot.named.push(first.value);
            has_named = true;
        } else {
            if has_named || range.start > active.start {
                return None;
            }
            slot.positional += 1;
        }
    }
    Some(slot)
}

fn line_end(lexed: &[Token<'_>], offset: u32) -> u32 {
    let start = lexed.partition_point(|token| token.start.offset + token.value.len() as u32 <= offset);
    lexed[start..]
        .iter()
        .filter(|token| token.kind == TokenKind::Whitespace)
        .find_map(|token| {
            let skip = offset.saturating_sub(token.start.offset) as usize;
            memchr::memchr2(b'\n', b'\r', &token.value[skip..]).map(|index| token.start.offset + (skip + index) as u32)
        })
        .unwrap_or(u32::MAX)
}
