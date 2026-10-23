//! Match parameter bindings and named argument labels to their metadata.

use mago_database::file::File;
use mago_syntax::token::TokenKind;

use crate::HoverInfo;
use crate::Range;
use crate::Server;
use crate::call_analysis::cursor::enclosing_call;
use crate::call_analysis::targets::resolve_signature_targets;
use crate::documentation::Documentation;
use crate::domain::HoverEntry;
use crate::lookup;

pub(super) fn argument(server: &mut Server, file: &File, offset: u32) -> Option<HoverInfo> {
    let tokens: Vec<_> = server.lex(file).into_iter().filter(|token| !lookup::is_trivia(token.kind)).collect();
    let index = tokens.partition_point(|token| token.start.offset <= offset).checked_sub(1)?;
    let label = &tokens[index];
    let range = Range::new(label.start.offset, label.start.offset + label.value.len() as u32);
    if !range.contains(offset) || !label.kind.is_identifier_maybe_reserved() {
        return None;
    }
    let colon = tokens.get(index + 1).filter(|token| token.kind == TokenKind::Colon)?;
    let call = enclosing_call(&tokens, colon.start.offset + 1)?;
    if call.argument_start != index {
        return None;
    }
    let targets = resolve_signature_targets(server, file.id, &tokens, &call);
    if targets.iter().any(|target| target.uncertain) {
        return None;
    }
    let names = crate::capabilities::signature::call_type_names(server.codebase(), &targets);
    let mut documentation = Documentation::new(server);
    let mut entries = Vec::new();
    for target in targets {
        let signature = target.signature(server.codebase())?;
        let function = signature.function;
        if function.flags.is_magic_method() {
            return None;
        }
        let parameter = signature.parameter(label.value)?;
        let mut declaration = String::new();
        // Bound spellings already share their source union's collision context.
        let displayed_type = parameter.type_name(Some(&names));
        crate::capabilities::signature::append_parameter_with_flags(
            &mut declaration,
            server.database(),
            parameter.declaration,
            displayed_type.as_deref(),
            parameter.flags(),
        );
        let mut entry = HoverEntry::for_symbol(parameter.declaration.name.0.as_bytes(), declaration);
        entry.documentation = documentation.for_callable_parameter(target.key, parameter.declaration.name.0.as_bytes());
        if !entries.contains(&entry) {
            entries.push(entry);
        }
    }
    (!entries.is_empty()).then_some(HoverInfo { entries, range })
}
