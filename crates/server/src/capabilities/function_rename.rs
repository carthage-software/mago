//! Check quoted runtime names before renaming a function.

use mago_allocator::LocalArena;
use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_syntax::token::TokenKind;
use mago_syntax_core::utils::parse_literal_string_in;

use crate::{Range, RefusalReason, RenameRefusal, Server, SymbolLocation};

impl Server {
    /// Find a complete quoted string matching one of `names`, and which name it matched.
    /// Without value-flow data, a matching data string may also be a callback.
    ///
    /// # Errors
    ///
    /// Returns a [`RenameRefusal`] when a quoted string cannot be decoded.
    pub fn callable_function_reference(
        &self,
        names: &[&[u8]],
    ) -> Result<Option<(SymbolLocation, usize)>, RenameRefusal> {
        for file in self.database().files().filter(|file| file.file_type == FileType::Host) {
            let arena = LocalArena::new();
            for token in self.tokens.tokens(&file).filter(|token| token.kind == TokenKind::LiteralString) {
                let range = Range::new(token.start.offset, token.start.offset + token.value.len() as u32);
                let literal = parse_literal_string_in(&arena, token.value, None, true).ok_or(RenameRefusal::at(
                    RefusalReason::Stale,
                    file.id,
                    range,
                ))?;
                let name = literal.strip_prefix(b"\\").unwrap_or(literal);
                if let Some(matched) = names.iter().position(|candidate| name.eq_ignore_ascii_case(candidate)) {
                    return Ok(Some((SymbolLocation { file: file.id, range }, matched)));
                }
            }
        }
        Ok(None)
    }
}
