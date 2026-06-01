//! `get_signature_help`: the signature of the call enclosing the cursor.
//!
//! When the cursor sits inside a call's arguments, return the called function's
//! signature and the active-parameter index. The call is found by a token-level
//! backward walk that balances parentheses.

use mago_bytes::BytesDisplay;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::TType;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::Range;
use crate::Server;
use crate::domain::SignatureInfo;
use crate::lookup;

impl Server {
    /// Signature help for the call enclosing `offset` in `file_id`, or `None`
    /// if the cursor isn't inside a resolvable function call.
    pub fn get_signature_help(&mut self, file_id: FileId, offset: u32) -> Option<SignatureInfo> {
        let file = self.database().get(&file_id).ok()?;
        let analysis = self.file_analysis_for(file_id)?;
        let tokens = lookup::lex(&file);
        let (call_name_token_idx, comma_count) = find_enclosing_call(&tokens, offset)?;
        let call_name_token = &tokens[call_name_token_idx];

        if !matches!(
            call_name_token.kind,
            TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
        ) {
            return None;
        }

        let fqcn: &[u8] = match analysis.resolved().resolve(&call_name_token.start) {
            Some(name) => name,
            None => mago_bytes::trim_start_byte(call_name_token.value, b'\\'),
        };
        let function = self.codebase().get_function(fqcn)?;

        let mut info = signature_info(function);
        info.active_parameter = comma_count.min(function.parameters.len().saturating_sub(1) as u32);
        Some(info)
    }
}

fn signature_info(meta: &FunctionLikeMetadata) -> SignatureInfo {
    use std::fmt::Write;
    let name = String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned();
    let mut label = format!("function {name}(");
    let mut parameters: Vec<Range> = Vec::with_capacity(meta.parameters.len());

    for (i, param) in meta.parameters.iter().enumerate() {
        if i > 0 {
            label.push_str(", ");
        }

        let param_start = label.len() as u32;
        if let Some(ty) = &param.type_metadata {
            let _ = write!(label, "{}", BytesDisplay(ty.type_union.get_id().as_bytes()));
            label.push(' ');
        }

        let _ = write!(label, "{}", BytesDisplay(param.name.0.as_bytes()));
        if param.flags.has_default() {
            label.push_str(" = ?");
        }

        let param_end = label.len() as u32;
        parameters.push(Range::new(param_start, param_end));
    }

    label.push(')');
    if let Some(rt) = &meta.return_type_metadata {
        label.push_str(": ");
        let _ = write!(label, "{}", BytesDisplay(rt.type_union.get_id().as_bytes()));
    }

    SignatureInfo { label, parameters, active_parameter: 0 }
}

/// Walk backwards from `offset` to find the function-call name token whose open
/// paren we're inside, and the number of commas passed (the active-parameter
/// index). Returns `(name_token_idx, comma_count)`.
fn find_enclosing_call(tokens: &[Token<'_>], offset: u32) -> Option<(usize, u32)> {
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
