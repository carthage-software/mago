//! `textDocument/signatureHelp`.
//!
//! When the cursor is inside a function/method call, return the signature
//! and highlight the active parameter. The mapping from "cursor inside
//! call" → "this is the function being called" is computed from a
//! token-level rewind: walk backwards from the cursor counting parens
//! until we land on the function-name token preceding the open paren.

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::TType;
use mago_database::file::File as MagoFile;
use mago_names::ResolvedNames;
use mago_syntax::token::TokenKind;
use tower_lsp::lsp_types::ParameterInformation;
use tower_lsp::lsp_types::ParameterLabel;
use tower_lsp::lsp_types::SignatureHelp;
use tower_lsp::lsp_types::SignatureInformation;

use crate::language_server::capabilities::lookup;

pub fn compute(
    codebase: &CodebaseMetadata,
    resolved: &ResolvedNames<'_>,
    file: &MagoFile,
    offset: u32,
) -> Option<SignatureHelp> {
    let tokens = lookup::lex(file);
    let (call_name_token_idx, comma_count) = find_enclosing_call(&tokens, offset)?;
    let call_name_token = &tokens[call_name_token_idx];

    if !matches!(
        call_name_token.kind,
        TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
    ) {
        return None;
    }

    let fqcn: &str = match resolved.resolve(&call_name_token.start) {
        Some(name) => name,
        None => call_name_token.value.trim_start_matches('\\'),
    };
    let function = codebase.get_function(fqcn)?;

    Some(SignatureHelp {
        signatures: vec![signature_information(function)],
        active_signature: Some(0),
        active_parameter: Some(comma_count.min(function.parameters.len().saturating_sub(1) as u32)),
    })
}

fn signature_information(meta: &FunctionLikeMetadata) -> SignatureInformation {
    let name = meta.original_name.map(|n| n.as_str().to_string()).unwrap_or_default();
    let mut label = format!("function {name}(");
    let mut params: Vec<ParameterInformation> = Vec::with_capacity(meta.parameters.len());

    for (i, param) in meta.parameters.iter().enumerate() {
        if i > 0 {
            label.push_str(", ");
        }
        let param_start = label.len() as u32;
        if let Some(ty) = &param.type_metadata {
            label.push_str(ty.type_union.get_id().as_str());
            label.push(' ');
        }
        label.push_str(param.name.0.as_str());
        if param.flags.has_default() {
            label.push_str(" = ?");
        }
        let param_end = label.len() as u32;
        params.push(ParameterInformation {
            label: ParameterLabel::LabelOffsets([param_start, param_end]),
            documentation: None,
        });
    }

    label.push(')');
    if let Some(rt) = &meta.return_type_metadata {
        label.push_str(": ");
        label.push_str(rt.type_union.get_id().as_str());
    }

    SignatureInformation { label, documentation: None, parameters: Some(params), active_parameter: None }
}

/// Walk backwards from `offset` to find the function-call name token whose
/// open paren we're inside, and the number of commas we've passed (which
/// is the active-parameter index). Returns `(name_token_idx, comma_count)`.
fn find_enclosing_call(tokens: &[mago_syntax::token::Token<'_>], offset: u32) -> Option<(usize, u32)> {
    let mut i = match tokens.iter().position(|t| t.start.offset >= offset) {
        Some(p) => p,
        None => tokens.len(),
    };

    let mut paren_depth: i32 = 0;
    let mut commas: u32 = 0;
    while i > 0 {
        i -= 1;
        let t = &tokens[i];
        match t.kind {
            TokenKind::RightParenthesis => paren_depth += 1,
            TokenKind::LeftParenthesis if paren_depth > 0 => paren_depth -= 1,
            TokenKind::LeftParenthesis if paren_depth == 0 => {
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    if !lookup::is_trivia(tokens[j].kind) {
                        return Some((j, commas));
                    }
                }
                return None;
            }
            TokenKind::Comma if paren_depth == 0 => commas += 1,
            _ => {}
        }
    }
    None
}
