//! Rename a parameter's lexical binding, PHPDoc and checked argument labels.

use foldhash::HashSet;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_database::DatabaseReader;
use mago_database::file::{FileId, FileType};
use mago_span::Span;

use crate::call_analysis::targets::SignatureTarget;
use crate::capabilities::member_rename::{Family, MemberRename};
use crate::semantic_index::{BindingKind, SymbolKind};
use crate::{NameProblem, RefusalReason, RenameRefusal, Server, SymbolLocation};

use super::{STALE, bindings, declared_outside, invalid, names};

mod arguments;
mod docs;

struct Parameter {
    callable: FunctionLikeIdentifier,
    declaration: Span,
    function: Span,
    name: Vec<u8>,
    variadic: bool,
}

pub(super) fn rename(
    server: &mut Server,
    file: FileId,
    offset: u32,
    new_name: Option<&str>,
) -> Result<Option<MemberRename>, RenameRefusal> {
    let analysis = server.file_analysis_for(file).ok_or(STALE)?;
    let (parameter, range) = if let Some(occurrence) = analysis.semantic().at_offset(offset) {
        let symbol = &analysis.semantic().symbols[occurrence.symbol as usize];
        if !matches!(symbol.binding, Some(BindingKind::Parameter | BindingKind::PromotedProperty)) {
            return Ok(None);
        }
        let declaration = analysis
            .semantic()
            .occurrences
            .iter()
            .find(|entry| entry.symbol == occurrence.symbol && entry.declaration)
            .ok_or(STALE)?;
        let Some(parameter) = at_declaration(server, file, declaration.range.start) else { return Ok(None) };
        (parameter, occurrence.selection_range)
    } else {
        let Some(selected) = arguments::selected(server, file, offset)? else { return Ok(None) };
        selected
    };
    let source = server.database().get(&parameter.declaration.file_id).map_err(|_| STALE)?;
    if parameter.declaration == parameter.function {
        return Err(RenameRefusal::at(RefusalReason::Builtin, file, range));
    }
    if source.file_type != FileType::Host {
        return Err(declared_outside(&source, parameter.declaration));
    }
    if let Some(name) = new_name {
        names::valid_name(&source, SymbolKind::Variable, name)?;
    }
    let declaration_analysis = server.file_analysis_for(source.id).ok_or(STALE)?;
    let occurrence = declaration_analysis.semantic().at_offset(parameter.declaration.start.offset).ok_or(STALE)?;
    if declaration_analysis.semantic().symbols[occurrence.symbol as usize].binding
        == Some(BindingKind::PromotedProperty)
    {
        let mut rename =
            server.get_member_rename(source.id, parameter.declaration.start.offset, new_name.map(str::as_bytes))?;
        rename.range = range;
        return Ok(Some(rename));
    }
    let (parameters, methods) = family(server, parameter)?;
    if let Some(methods) = methods {
        methods.refuse_outside(server, parameters.iter().map(|parameter| parameter.declaration), |span| {
            match parameters.iter().find(|parameter| parameter.declaration == span)?.callable {
                FunctionLikeIdentifier::Method(class, _) => Some(class),
                FunctionLikeIdentifier::Function(_) | FunctionLikeIdentifier::Closure(_) => None,
            }
        })?;
        methods.check(server, None)?;
    }
    let mut locations = Vec::new();
    collect(server, &parameters, new_name, &mut locations)?;
    Ok(Some(MemberRename { range, locations }))
}

/// Property families may contain promoted declarations with a second binding.
pub(in crate::capabilities) fn extend_properties(
    server: &mut Server,
    declarations: &HashSet<Span>,
    new_name: Option<&[u8]>,
    locations: &mut Vec<SymbolLocation>,
) -> Result<(), RenameRefusal> {
    let parameters: Vec<_> =
        declarations.iter().filter_map(|span| at_declaration(server, span.file_id, span.start.offset)).collect();
    if parameters.is_empty() {
        return Ok(());
    }
    let new_name = new_name.map(std::str::from_utf8).transpose().map_err(|_| invalid(NameProblem::NotIdentifier))?;
    collect(server, &parameters, new_name, locations)
}

fn at_declaration(server: &Server, file: FileId, offset: u32) -> Option<Parameter> {
    server.codebase().function_likes.iter().find_map(|((owner, _), function)| {
        if function.span.file_id != file {
            return None;
        }
        let parameter = function.parameters.iter().find(|parameter| parameter.name_span.start.offset == offset)?;
        Some(Parameter {
            callable: match function.kind {
                FunctionLikeKind::Function => FunctionLikeIdentifier::Function(function.name),
                FunctionLikeKind::Method => FunctionLikeIdentifier::Method(*owner, function.name),
                FunctionLikeKind::Closure | FunctionLikeKind::ArrowFunction => {
                    FunctionLikeIdentifier::Closure(function.name)
                }
            },
            declaration: parameter.name_span,
            function: function.span,
            name: parameter.name.0.as_bytes().strip_prefix(b"$")?.to_vec(),
            variadic: parameter.flags.is_variadic(),
        })
    })
}

/// Match the same parameter name across related method declarations. PHP permits
/// overrides to use different names; those bindings remain separate.
fn family(server: &Server, selected: Parameter) -> Result<(Vec<Parameter>, Option<Family>), RenameRefusal> {
    let metadata = server.codebase().get_function_like(&selected.callable).ok_or(STALE)?;
    let Some(method) = metadata.method_metadata.as_ref() else { return Ok((vec![selected], None)) };
    if method.is_constructor || method.visibility.is_private() {
        return Ok((vec![selected], None));
    }
    let methods = server.method_family(metadata)?;
    let mut parameters = Vec::new();
    for function in server.codebase().function_likes.values() {
        if function.name_span.is_none_or(|span| !methods.declarations.contains(&span)) {
            continue;
        }
        if let Some(parameter) = function
            .parameters
            .iter()
            .find(|parameter| parameter.name.0.as_bytes().strip_prefix(b"$") == Some(selected.name.as_slice()))
        {
            parameters.push(
                at_declaration(server, parameter.name_span.file_id, parameter.name_span.start.offset).ok_or(STALE)?,
            );
        }
    }
    Ok((parameters, Some(methods)))
}

fn collect(
    server: &mut Server,
    parameters: &[Parameter],
    new_name: Option<&str>,
    locations: &mut Vec<SymbolLocation>,
) -> Result<(), RenameRefusal> {
    for parameter in parameters {
        let file = server.database().get(&parameter.declaration.file_id).map_err(|_| STALE)?;
        let analysis = server.file_analysis_for(file.id).ok_or(STALE)?;
        let index = analysis.semantic();
        let occurrence = index.at_offset(parameter.declaration.start.offset).ok_or(STALE)?;
        if let Some(new_name) = new_name {
            names::valid_name(&file, SymbolKind::Variable, new_name)?;
        }
        bindings::ensure_unbound(index, file.id, occurrence.symbol, new_name)?;
        locations.extend(
            index
                .occurrences
                .iter()
                .filter(|entry| entry.symbol == occurrence.symbol)
                .map(|entry| SymbolLocation { file: file.id, range: entry.selection_range }),
        );
        docs::collect(server, &file, parameter, index, locations);
    }
    arguments::collect(server, parameters, new_name, locations)?;
    locations.sort_unstable_by_key(|location| (location.file, location.range.start, location.range.end));
    locations.dedup();
    Ok(())
}

fn parameter_for_call(server: &Server, target: &SignatureTarget, name: &[u8]) -> Option<Parameter> {
    let signature = target.signature(server.codebase())?;
    let parameter = signature.parameter(name)?;
    if parameter.flags().is_variadic() {
        return None;
    }
    Some(Parameter {
        callable: target.key,
        declaration: parameter.declaration.name_span,
        function: signature.function.span,
        name: name.to_vec(),
        variadic: false,
    })
}
