//! Parameter names for positional arguments of known calls.

use mago_bytes::BytesDisplay;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::Range;
use crate::Server;
use crate::call_analysis::call_targets;
use crate::call_analysis::cursor::CallCursor;
use crate::call_analysis::signature::CallSignature;
use crate::call_analysis::targets::SignatureTarget;
use crate::call_analysis::targets::resolve_signature_targets;
use crate::domain::HintKind;
use crate::domain::InlayHintItem;
use crate::lookup;
use crate::symbol_resolution::methods_at;

impl Server {
    /// Parameter names for arguments starting within `[range_start, range_end)`.
    pub fn get_inlay_hints(&mut self, file_id: FileId, range_start: u32, range_end: u32) -> Vec<InlayHintItem> {
        if range_start >= range_end {
            return Vec::new();
        }
        let Ok(file) = self.database().get(&file_id) else { return Vec::new() };
        let Some(analysis) = self.file_analysis_for(file_id) else { return Vec::new() };
        let editor = self.editor_file(file_id);
        let tokens: Vec<_> = self.lex(&file).into_iter().filter(|token| !lookup::is_trivia(token.kind)).collect();
        let pairs = delimiter_pairs(&tokens);
        let mut hints = Vec::new();
        for call in &analysis.calls {
            if call.expression.end <= range_start || call.selection.end >= range_end {
                continue;
            }
            if editor.as_ref().is_some_and(|editor| {
                methods_at(editor, call.selection.start).iter().any(|row| row.uncertain || row.magic)
            }) {
                continue;
            }
            let Some(close) = tokens.partition_point(|token| token.start.offset < call.expression.end).checked_sub(1)
            else {
                continue;
            };
            if tokens[close].kind != TokenKind::RightParenthesis
                || pairs[close] == u32::MAX
                || tokens[pairs[close] as usize].start.offset < call.selection.end
            {
                continue;
            }
            let open = pairs[close] as usize;
            let targets = if call.is_callable() {
                let Some(name) = open.checked_sub(1) else { continue };
                let cursor = CallCursor {
                    name,
                    open,
                    argument_start: open + 1,
                    argument: 0,
                    argument_name: None,
                    attribute: false,
                };
                resolve_signature_targets(self, file_id, &tokens, &cursor)
            } else {
                call_targets(self.codebase(), &file, &analysis, editor.as_deref(), call)
                    .into_iter()
                    .map(|key| SignatureTarget { key, bound: None, owner: None, name: None, uncertain: false })
                    .collect()
            };
            if targets.is_empty() || targets.iter().any(|target| target.uncertain) {
                continue;
            }
            let signatures: Vec<_> = targets.iter().filter_map(|target| target.signature(self.codebase())).collect();
            if signatures.len() != targets.len()
                || signatures.iter().any(|signature| signature.function.flags.is_magic_method())
            {
                continue;
            }
            emit_argument_hints(
                &tokens,
                &pairs,
                open + 1..close,
                Range { start: range_start, end: range_end },
                &signatures,
                &mut hints,
            );
        }
        hints.sort_unstable_by_key(|hint| hint.offset);
        hints.dedup_by(|left, right| left.offset == right.offset && left.label == right.label);
        hints
    }
}

fn delimiter_pairs(tokens: &[Token<'_>]) -> Vec<u32> {
    let mut pairs = vec![u32::MAX; tokens.len()];
    let mut stack = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LeftParenthesis => stack.push((index, TokenKind::RightParenthesis)),
            TokenKind::LeftBracket | TokenKind::HashLeftBracket => stack.push((index, TokenKind::RightBracket)),
            TokenKind::LeftBrace | TokenKind::DollarLeftBrace => stack.push((index, TokenKind::RightBrace)),
            TokenKind::RightParenthesis | TokenKind::RightBracket | TokenKind::RightBrace => {
                if let Some((open, expected)) = stack.pop()
                    && expected == token.kind
                {
                    pairs[open] = index as u32;
                    pairs[index] = open as u32;
                } else {
                    stack.clear();
                }
            }
            _ => {}
        }
    }
    pairs
}

fn emit_argument_hints(
    tokens: &[Token<'_>],
    pairs: &[u32],
    arguments: std::ops::Range<usize>,
    requested: Range,
    signatures: &[CallSignature<'_>],
    out: &mut Vec<InlayHintItem>,
) {
    let mut start = arguments.start;
    let mut argument = 0;
    while start < arguments.end {
        let first = &tokens[start];
        // Unpacking changes the position of all later arguments.
        if first.kind == TokenKind::DotDotDot {
            return;
        }
        let named = tokens.get(start + 1).is_some_and(|token| token.kind == TokenKind::Colon);
        if !named
            && requested.start <= first.start.offset
            && first.start.offset < requested.end
            && let Some((name, variadic)) = agreed_parameter(signatures, argument)
        {
            let name = BytesDisplay(name.strip_prefix(b"$").unwrap_or(name));
            let label = if variadic { format!("...{name}:") } else { format!("{name}:") };
            out.push(InlayHintItem { offset: first.start.offset, label, kind: HintKind::Parameter });
        }
        let mut cursor = start;
        while cursor < arguments.end && tokens[cursor].kind != TokenKind::Comma {
            let pair = pairs[cursor];
            cursor = if pair != u32::MAX && pair as usize > cursor { pair as usize + 1 } else { cursor + 1 };
        }
        start = cursor + 1;
        argument += 1;
    }
}

fn agreed_parameter<'a>(signatures: &[CallSignature<'a>], index: usize) -> Option<(&'a [u8], bool)> {
    let parameter = |signature: CallSignature<'a>| {
        signature
            .parameters()
            .nth(index)
            .map(|parameter| (parameter.declaration.name.0.as_bytes(), parameter.flags().is_variadic()))
    };
    let (first, rest) = signatures.split_first()?;
    rest.iter().try_fold(parameter(*first)?, |(name, variadic), signature| {
        let (other, flag) = parameter(*signature)?;
        (other == name).then_some((name, variadic && flag))
    })
}
