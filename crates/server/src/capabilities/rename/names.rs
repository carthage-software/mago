//! Check replacement names under the existing PHP rename rules.

use mago_database::file::File as MagoFile;
use mago_syntax::lexer::Lexer;
use mago_syntax::settings::LexerSettings;
use mago_syntax::token::TokenKind;
use mago_syntax_core::input::Input;

use crate::semantic_index::SymbolKind;
use crate::{NameProblem, RefusalReason, RenameRefusal};

pub(super) fn identifier(name: &str) -> Result<(), RenameRefusal> {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else { return Err(invalid(NameProblem::Empty)) };
    if (first.is_ascii_alphabetic() || first == b'_' || first >= 0x80)
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80)
    {
        Ok(())
    } else {
        Err(invalid(NameProblem::NotIdentifier))
    }
}

pub(super) fn valid_name(file: &MagoFile, kind: SymbolKind, name: &str) -> Result<(), RenameRefusal> {
    identifier(name)?;
    if kind == SymbolKind::Variable {
        return if crate::variables::is_builtin_variable(name.as_bytes()) {
            Err(invalid(NameProblem::ReservedVariable))
        } else {
            Ok(())
        };
    }
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "self"
            | "parent"
            | "static"
            | "int"
            | "float"
            | "bool"
            | "string"
            | "true"
            | "false"
            | "null"
            | "void"
            | "never"
            | "mixed"
            | "iterable"
            | "object"
            | "resource"
            | "numeric"
    ) {
        return Err(invalid(NameProblem::ReservedType));
    }
    let source = format!("<?php {name}");
    let mut lexer = Lexer::new(Input::new(file.id, source.as_bytes()), LexerSettings::default());
    let mut identifier = false;
    while let Some(token) = lexer.advance() {
        let Ok(token) = token else { return Err(invalid(NameProblem::Keyword)) };
        if token.start.offset >= 6 {
            identifier = token.kind == TokenKind::Identifier;
        }
    }
    if identifier { Ok(()) } else { Err(invalid(NameProblem::Keyword)) }
}

pub(crate) const fn invalid(problem: NameProblem) -> RenameRefusal {
    RenameRefusal::new(RefusalReason::InvalidName(problem))
}
