//! Cursor context and the PHP name expected at that position.

use mago_database::file::File as MagoFile;
use mago_names::kind::NameKind;
use mago_php_version::PHPVersion;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::file_analysis::FileAnalysis;
use crate::lookup;
use crate::semantic_index::ScopeKind;

use super::native_type_context;

#[derive(Debug, Clone, Copy)]
pub(super) enum NameIntent {
    Declaration,
    Expression,
    Type,
    NativeType(native_type_context::NativeTypeContext),
    Instantiation,
    ClassExtends,
    Interface,
    Trait,
    Import(NameKind),
}

impl NameIntent {
    pub(super) fn includes(self, kind: NameKind) -> bool {
        match self {
            Self::Declaration => false,
            Self::Expression => true,
            Self::Import(import) => import == kind,
            _ => kind == NameKind::Default,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Context<'a> {
    Variable { prefix: &'a [u8] },
    InstanceMember { receiver_end: u32, prefix: &'a [u8], static_access: bool },
    StaticMember { class: &'a [u8], prefix: &'a [u8] },
    Qualified { qualifier: &'a [u8], prefix: &'a [u8], intent: NameIntent },
    Bare { prefix: &'a [u8], intent: NameIntent },
}

/// Closures keep their enclosing class scope; named functions do not.
pub(super) fn has_class_scope(analysis: &FileAnalysis, offset: u32) -> bool {
    let Some(mut scope) = crate::call_analysis::scope_at(analysis, offset) else { return true };
    loop {
        match scope.kind {
            ScopeKind::Function => return false,
            ScopeKind::Method | ScopeKind::PropertyHook | ScopeKind::File => return true,
            ScopeKind::Closure | ScopeKind::ArrowFunction => {}
        }
        let Some(parent) = scope.parent else { return true };
        scope = &analysis.semantic().scopes[parent as usize];
    }
}

fn byte_before(file: &MagoFile, offset: u32) -> Option<u8> {
    if offset == 0 {
        return None;
    }

    file.contents.get((offset - 1) as usize).copied()
}

pub(super) fn classify<'a>(
    file: &MagoFile,
    tokens: &'a [Token<'a>],
    offset: u32,
    version: PHPVersion,
) -> Option<Context<'a>> {
    let mut current_idx: Option<usize> = None;
    for (i, t) in tokens.iter().enumerate() {
        if lookup::is_trivia(t.kind) {
            continue;
        }

        let start = t.start.offset;
        let end = start + t.value.len() as u32;
        if start <= offset && offset <= end {
            let starts_here_after_prev = offset == start
                && current_idx.is_some_and(|c| tokens[c].start.offset + tokens[c].value.len() as u32 == offset);
            if !starts_here_after_prev {
                current_idx = Some(i);
            }
        }
        if start > offset {
            break;
        }
    }

    let Some(current_idx) = current_idx else {
        return classify_after_punctuation(file, tokens, offset, version);
    };

    let cur = &tokens[current_idx];
    let prev_idx = (0..current_idx).rev().find(|j| !lookup::is_trivia(tokens[*j].kind));
    let prev = prev_idx.map(|j| tokens[j].kind);
    let consumed = (offset - cur.start.offset) as usize;
    let typed = &cur.value[..consumed.min(cur.value.len())];
    if prev == Some(TokenKind::ColonColon)
        && let Some(prev_idx) = prev_idx
    {
        return static_member(tokens, prev_idx, typed);
    }

    if matches!(cur.kind, TokenKind::Variable) {
        let raw = cur.value;
        let consumed = (offset - cur.start.offset) as usize;
        let prefix: &[u8] = if consumed >= 1 && consumed <= raw.len() { &raw[1..consumed] } else { b"" };
        return Some(Context::Variable { prefix });
    }

    let cursor_at_end_of_cur = offset == cur.start.offset + cur.value.len() as u32;
    if cursor_at_end_of_cur {
        match cur.kind {
            TokenKind::Dollar => return Some(Context::Variable { prefix: b"" }),
            TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan => {
                return instance_member(tokens, current_idx, b"");
            }
            TokenKind::ColonColon => return static_member(tokens, current_idx, b""),
            _ => {}
        }
    }

    if matches!(prev, Some(TokenKind::MinusGreaterThan) | Some(TokenKind::QuestionMinusGreaterThan))
        && let Some(prev_idx) = prev_idx
    {
        let prefix: &[u8] = if cur.kind.is_identifier_maybe_reserved() { typed } else { b"" };
        return instance_member(tokens, prev_idx, prefix);
    }

    Some(match cur.kind {
        TokenKind::Class
        | TokenKind::Interface
        | TokenKind::Trait
        | TokenKind::Enum
        | TokenKind::Function
        | TokenKind::Const
        | TokenKind::Case
            if super::snippets::has_declaration(tokens, offset, version) =>
        {
            Context::Bare { prefix: typed, intent: NameIntent::Declaration }
        }
        kind if kind.is_identifier_maybe_reserved() => {
            Context::Bare { prefix: typed, intent: name_intent(tokens, current_idx, version) }
        }
        TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier => match rsplit_once_byte(typed, b'\\') {
            Some((qual, last)) => {
                Context::Qualified { qualifier: qual, prefix: last, intent: name_intent(tokens, current_idx, version) }
            }
            None => Context::Bare { prefix: typed, intent: name_intent(tokens, current_idx, version) },
        },
        TokenKind::NamespaceSeparator => match prev_idx.map(|j| &tokens[j]) {
            Some(name)
                if matches!(
                    name.kind,
                    TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
                ) =>
            {
                Context::Qualified {
                    qualifier: name.value,
                    prefix: b"",
                    intent: name_intent(tokens, prev_idx.unwrap_or(current_idx), version),
                }
            }
            _ => Context::Qualified { qualifier: b"", prefix: b"", intent: name_intent(tokens, current_idx, version) },
        },
        _ => Context::Bare {
            prefix: b"",
            intent: name_intent(tokens, current_idx + usize::from(cursor_at_end_of_cur), version),
        },
    })
}

pub(super) fn name_intent(tokens: &[Token<'_>], before: usize, version: PHPVersion) -> NameIntent {
    let mut list = false;
    for (index, token) in tokens[..before].iter().enumerate().rev() {
        if lookup::is_trivia(token.kind) {
            continue;
        }
        match token.kind {
            TokenKind::Class | TokenKind::Interface | TokenKind::Trait | TokenKind::Enum | TokenKind::Function => {
                return NameIntent::Declaration;
            }
            TokenKind::Case if super::snippets::in_enum_body(tokens, token.start.offset) => {
                return NameIntent::Declaration;
            }
            TokenKind::New if !list => return NameIntent::Instantiation,
            TokenKind::Instanceof if !list => return NameIntent::Type,
            TokenKind::Implements => return NameIntent::Interface,
            TokenKind::Use | TokenKind::Insteadof => return NameIntent::Trait,
            TokenKind::Extends => {
                let interface = tokens[..index]
                    .iter()
                    .rev()
                    .find(|token| {
                        matches!(
                            token.kind,
                            TokenKind::Interface | TokenKind::Class | TokenKind::LeftBrace | TokenKind::Semicolon
                        )
                    })
                    .is_some_and(|token| token.kind == TokenKind::Interface);
                return if interface { NameIntent::Interface } else { NameIntent::ClassExtends };
            }
            TokenKind::Comma => list = true,
            TokenKind::Identifier
            | TokenKind::QualifiedIdentifier
            | TokenKind::FullyQualifiedIdentifier
            | TokenKind::NamespaceSeparator
            | TokenKind::Ampersand => {}
            _ => break,
        }
    }
    let Some(context) = native_type_context::detect(tokens, before) else { return NameIntent::Expression };
    if context.position == native_type_context::Position::Constant {
        let declaration = &tokens[context.start_token - 1];
        let previous = tokens[context.start_token..before].iter().rev().find(|token| !lookup::is_trivia(token.kind));
        // A complete hint or a grouped declaration needs a new constant name.
        if version < PHPVersion::PHP83
            || !super::snippets::in_class_body(tokens, declaration.start.offset)
            || previous.is_some_and(|token| {
                token.kind.is_identifier_maybe_reserved()
                    || matches!(
                        token.kind,
                        TokenKind::QualifiedIdentifier
                            | TokenKind::FullyQualifiedIdentifier
                            | TokenKind::RightParenthesis
                            | TokenKind::Comma
                    )
            })
        {
            return NameIntent::Declaration;
        }
    }
    NameIntent::NativeType(context)
}

fn rsplit_once_byte(s: &[u8], byte: u8) -> Option<(&[u8], &[u8])> {
    memchr::memrchr(byte, s).map(|i| (&s[..i], &s[i + 1..]))
}

fn classify_after_punctuation<'a>(
    file: &MagoFile,
    tokens: &'a [Token<'a>],
    offset: u32,
    version: PHPVersion,
) -> Option<Context<'a>> {
    if byte_before(file, offset) == Some(b'$') {
        return Some(Context::Variable { prefix: b"" });
    }

    let prev_idx = (0..tokens.len()).rev().find(|i| {
        let token = &tokens[*i];
        let token_end = token.start.offset + token.value.len() as u32;
        token_end <= offset && !lookup::is_trivia(token.kind)
    });

    let Some(prev_idx) = prev_idx else {
        return Some(Context::Bare { prefix: b"", intent: NameIntent::Expression });
    };

    match tokens[prev_idx].kind {
        TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan => instance_member(tokens, prev_idx, b""),
        TokenKind::ColonColon => static_member(tokens, prev_idx, b""),
        _ => Some(Context::Bare { prefix: b"", intent: name_intent(tokens, prev_idx + 1, version) }),
    }
}

fn instance_member<'a>(tokens: &[Token<'_>], arrow_idx: usize, prefix: &'a [u8]) -> Option<Context<'a>> {
    receiver_before(tokens, arrow_idx).map(|receiver_end| Context::InstanceMember {
        receiver_end,
        prefix,
        static_access: false,
    })
}

fn static_member<'a>(tokens: &'a [Token<'a>], colon_idx: usize, prefix: &'a [u8]) -> Option<Context<'a>> {
    let mut before = tokens[..colon_idx].iter().rev().filter(|token| !lookup::is_trivia(token.kind));
    let receiver = before.next()?;
    let member = receiver.kind.is_identifier_maybe_reserved()
        && before.next().is_some_and(|token| {
            matches!(
                token.kind,
                TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan | TokenKind::ColonColon
            )
        });
    if !member && let Some(class) = static_receiver_before(tokens, colon_idx) {
        return Some(Context::StaticMember { class, prefix });
    }

    (member || matches!(receiver.kind, TokenKind::Variable | TokenKind::RightParenthesis | TokenKind::RightBracket))
        .then(|| Context::InstanceMember {
            receiver_end: receiver.start.offset + receiver.value.len() as u32,
            prefix,
            static_access: true,
        })
}

fn receiver_before(tokens: &[Token<'_>], arrow_idx: usize) -> Option<u32> {
    let mut j = arrow_idx;
    while j > 0 {
        j -= 1;
        if !lookup::is_trivia(tokens[j].kind) {
            let end_token = &tokens[j];
            let end = end_token.start.offset + end_token.value.len() as u32;
            return Some(end);
        }
    }
    None
}

/// The static receiver name written before `::`. The scope resolves it to a
/// fully-qualified name (handling `use` aliases and namespace relativity).
fn static_receiver_before<'a>(tokens: &'a [Token<'a>], colon_idx: usize) -> Option<&'a [u8]> {
    let mut k = colon_idx;
    while k > 0 {
        k -= 1;
        if lookup::is_trivia(tokens[k].kind) {
            continue;
        }

        return match tokens[k].kind {
            TokenKind::Self_ | TokenKind::Static | TokenKind::Parent => Some(tokens[k].value),
            TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier => {
                Some(tokens[k].value)
            }
            _ => None,
        };
    }
    None
}
