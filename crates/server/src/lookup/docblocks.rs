//! Find PHPDoc attached to declarations through cached tokens.

use std::sync::Arc;

use mago_database::file::File as MagoFile;
use mago_syntax::token::TokenKind;

use super::CacheEntry;
use super::RawToken;
use super::is_trivia;

#[derive(Clone, Copy)]
pub(crate) enum DocblockAnchor {
    DeclarationStart,
    /// The name follows type, modifier, or attribute tokens.
    DeclarationName,
}

impl DocblockAnchor {
    fn skips(self, kind: TokenKind) -> bool {
        is_trivia(kind)
            || matches!(self, Self::DeclarationName)
                && matches!(
                    kind,
                    TokenKind::Identifier
                        | TokenKind::QualifiedIdentifier
                        | TokenKind::FullyQualifiedIdentifier
                        | TokenKind::Public
                        | TokenKind::Protected
                        | TokenKind::Private
                        | TokenKind::PublicSet
                        | TokenKind::ProtectedSet
                        | TokenKind::PrivateSet
                        | TokenKind::Static
                        | TokenKind::Readonly
                        | TokenKind::Final
                        | TokenKind::Abstract
                        | TokenKind::Var
                        | TokenKind::Const
                        | TokenKind::Self_
                        | TokenKind::Parent
                        | TokenKind::Array
                        | TokenKind::Null
                        | TokenKind::True
                        | TokenKind::False
                        | TokenKind::Question
                        | TokenKind::Pipe
                        | TokenKind::Ampersand
                        | TokenKind::LeftParenthesis
                        | TokenKind::RightParenthesis
                )
    }
}

/// A source snapshot and its tokens, shared by PHPDoc lookups within a request.
pub(crate) struct DocblockSource {
    pub(super) file: Arc<MagoFile>,
    pub(super) entry: Arc<CacheEntry>,
}

impl DocblockSource {
    /// Find attached PHPDoc without hashing the file or copying its tokens.
    pub(crate) fn before(&self, offset: u32, anchor: DocblockAnchor) -> Option<&[u8]> {
        self.entry.docblock_before(&self.file, offset, anchor)
    }
}

impl CacheEntry {
    pub(super) fn docblock_before<'file>(
        &self,
        file: &'file MagoFile,
        offset: u32,
        anchor: DocblockAnchor,
    ) -> Option<&'file [u8]> {
        let mut cursor = self.tokens.partition_point(|token| token.start < offset);
        let mut covered = offset;
        let mut brackets = 0;
        while let Some(index) = cursor.checked_sub(1) {
            cursor = index;
            let token = &self.tokens[index];
            if token.end != covered {
                return None;
            }
            covered = token.start;
            if matches!(anchor, DocblockAnchor::DeclarationName) && token.kind == TokenKind::RightBracket {
                brackets += 1;
            } else if brackets > 0 {
                if matches!(token.kind, TokenKind::LeftBracket | TokenKind::HashLeftBracket) {
                    brackets -= 1;
                    if brackets == 0 && token.kind != TokenKind::HashLeftBracket {
                        return None;
                    }
                }
            } else if token.kind == TokenKind::DocBlockComment {
                return file.contents.get(token.start as usize..token.end as usize);
            } else if matches!(anchor, DocblockAnchor::DeclarationName) && token.kind == TokenKind::Comma {
                cursor = previous_declarator(&self.tokens[..index], token.start)?;
                covered = self.tokens[cursor].start;
            } else if !anchor.skips(token.kind) {
                return None;
            }
        }
        None
    }
}

/// Skip one earlier constant or property in the same declaration.
fn previous_declarator(tokens: &[RawToken], mut covered: u32) -> Option<usize> {
    let mut delimiters = Vec::new();
    let mut first = true;
    let mut expect_name = false;
    for (index, token) in tokens.iter().enumerate().rev() {
        if token.end != covered {
            return None;
        }
        covered = token.start;
        if is_trivia(token.kind) {
            continue;
        }
        if expect_name || first && token.kind == TokenKind::Variable {
            return (token.kind.is_identifier_maybe_reserved() || token.kind == TokenKind::Variable).then_some(index);
        }
        first = false;
        match token.kind {
            TokenKind::RightParenthesis => delimiters.push(TokenKind::LeftParenthesis),
            TokenKind::RightBracket => delimiters.push(TokenKind::LeftBracket),
            TokenKind::RightBrace => delimiters.push(TokenKind::LeftBrace),
            TokenKind::LeftParenthesis | TokenKind::LeftBracket | TokenKind::LeftBrace => {
                if delimiters.pop() != Some(token.kind) {
                    return None;
                }
            }
            TokenKind::HashLeftBracket | TokenKind::DollarLeftBrace => {
                let expected = if token.kind == TokenKind::HashLeftBracket {
                    TokenKind::LeftBracket
                } else {
                    TokenKind::LeftBrace
                };
                if delimiters.pop() != Some(expected) {
                    return None;
                }
            }
            _ if !delimiters.is_empty() => {}
            TokenKind::Equal => expect_name = true,
            TokenKind::Comma | TokenKind::Semicolon => return None,
            _ => {}
        }
    }
    None
}
