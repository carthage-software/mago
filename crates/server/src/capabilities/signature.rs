//! Shared parameter rendering; callers choose names and type display style.

use std::fmt::Write;

use mago_bytes::BytesDisplay;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::ttype::display::NameDisplay;
use mago_codex::ttype::union::TUnion;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_syntax::lexer::Lexer;
use mago_syntax::settings::LexerSettings;
use mago_syntax::token::TokenKind;
use mago_syntax_core::input::Input;

use crate::Range;
use crate::call_analysis::targets::SignatureTarget;

pub(super) fn signature_types(function: &FunctionLikeMetadata) -> impl Iterator<Item = &TUnion> {
    function
        .parameters
        .iter()
        .filter_map(|parameter| parameter.type_metadata.as_ref())
        .chain(function.return_type_metadata.as_ref())
        .map(|ty| &ty.type_union)
}

pub(super) fn type_names<'code>(
    codebase: &'code CodebaseMetadata,
    function: &FunctionLikeMetadata,
) -> NameDisplay<'code> {
    NameDisplay::for_types(codebase, signature_types(function))
}

pub(super) fn call_type_names<'code>(
    codebase: &'code CodebaseMetadata,
    targets: &[SignatureTarget],
) -> NameDisplay<'code> {
    if targets.iter().any(|target| target.bound.is_some()) {
        return NameDisplay::fully_qualified(codebase);
    }
    NameDisplay::for_types(
        codebase,
        targets.iter().filter_map(|target| codebase.get_function_like(&target.key)).flat_map(signature_types),
    )
}

pub(super) fn append_parameter(
    output: &mut String,
    database: &Database<'_>,
    parameter: &FunctionLikeParameterMetadata,
    displayed_type: Option<&[u8]>,
) -> Range {
    append_parameter_with_flags(output, database, parameter, displayed_type, parameter.flags)
}

pub(super) fn append_parameter_with_flags(
    output: &mut String,
    database: &Database<'_>,
    parameter: &FunctionLikeParameterMetadata,
    displayed_type: Option<&[u8]>,
    flags: MetadataFlags,
) -> Range {
    let start = output.len() as u32;
    if let Some(displayed_type) = displayed_type {
        let _ = write!(output, "{} ", BytesDisplay(displayed_type));
    }
    if flags.is_by_reference() {
        output.push('&');
    }
    if flags.is_variadic() {
        output.push_str("...");
    }
    let _ = write!(output, "{}", BytesDisplay(parameter.name.0.as_bytes()));
    if flags.has_default() {
        let file = database.get(&parameter.span.file_id).ok();
        let default = file
            .as_ref()
            .and_then(|file| {
                file.contents.get(parameter.name_span.end.offset as usize..parameter.span.end.offset as usize)
            })
            .and_then(|bytes| {
                let mut lexer = Lexer::scripting(Input::new(parameter.span.file_id, bytes), LexerSettings::default());
                while let Some(token) = lexer.advance() {
                    let token = token.ok()?;
                    if crate::lookup::is_trivia(token.kind) {
                        continue;
                    }
                    return (token.kind == TokenKind::Equal)
                        .then(|| bytes[token.start.offset as usize..].trim_ascii_end());
                }
                None
            });
        let _ = write!(output, " {}", BytesDisplay(default.unwrap_or(b"= ?")));
    }
    Range::new(start, output.len() as u32)
}
