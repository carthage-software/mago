//! `get_context`: resolve the identifier (or variable) under the cursor and
//! describe it for hover.

use mago_analyzer::artifacts::editor::{EditorFileData, MemberKind};
use mago_bytes::BytesDisplay;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId};
use mago_span::Span;

use crate::Server;
use crate::domain::Range;
use crate::domain::{HoverEntry, HoverInfo};
use crate::method_presentation::MethodPresentation;
use crate::semantic_index::SymbolKind as NameKind;
use crate::symbol_resolution::members_at;
use crate::symbol_resolution::methods_at;
use crate::symbol_resolution::parameter_at;
use crate::symbol_resolution::resolved_symbol_name;

mod description;
mod literal;
mod member;
mod parameters;
mod types;

impl Server {
    /// Describe each symbol under the cursor and keep the token's exact span.
    pub fn get_context(&mut self, file_id: FileId, offset: u32) -> Option<HoverInfo> {
        let file = self.database().get(&file_id).ok()?;
        if let Some(editor) = self.editor_file(file_id)
            && let Some(hover) = method_reference_hover(self, &file, &editor, offset)
                .or_else(|| member_reference_hover(self, &editor, offset))
        {
            return Some(hover);
        }
        if let Some(hover) = declaration_hover(self, file_id, offset) {
            return Some(hover);
        }
        let analysis = self.file_analysis_for(file_id)?;
        let parameter = parameter_at(self.codebase(), &file, analysis.semantic(), offset);
        if let Some((parameter, owner)) = parameter
            && Range::new(parameter.name_span.start.offset, parameter.name_span.end.offset).contains(offset)
        {
            return Some(HoverInfo {
                entries: vec![description::parameter(self, parameter, owner)],
                range: Range::new(parameter.name_span.start.offset, parameter.name_span.end.offset),
            });
        }
        if let Some(editor) = self.editor_file(file_id)
            && let Some((span, union)) = editor.variable_types.at_offset(offset)
            && let Some(name) = file.contents.get(span.0 as usize..span.1 as usize)
        {
            if name == b"$this"
                && crate::call_analysis::scope_at(&analysis, offset).is_some_and(|scope| scope.is_static)
            {
                return None;
            }
            let mut entry = description::variable(self, name, union);
            if let Some((parameter, owner)) = parameter {
                entry.documentation = crate::documentation::for_parameter(self, owner, parameter.name.0.as_bytes());
            }
            return Some(HoverInfo { entries: vec![entry], range: Range::new(span.0, span.1) });
        }
        let Some(occurrence) = analysis.semantic().at_offset(offset) else {
            return literal::at(self, &file, &analysis, offset).or_else(|| parameters::argument(self, &file, offset));
        };
        let symbol = &analysis.semantic().symbols[occurrence.symbol as usize];
        let name = resolved_symbol_name(self.codebase(), &file.contents, analysis.semantic(), occurrence);
        let entry = match symbol.kind {
            NameKind::ClassLike => description::class_like(self, self.codebase().get_class_like(name)?),
            NameKind::Function => {
                let function = self.codebase().get_function(name)?;
                description::function_like(self, FunctionLikeIdentifier::Function(function.name), function, None)
            }
            NameKind::Constant => member::constant(self, self.codebase().get_constant(name)?),
            NameKind::Variable => parameter.map_or_else(
                || HoverEntry::for_symbol(&symbol.name, format!("${}", BytesDisplay(&symbol.name))),
                |(parameter, owner)| description::parameter(self, parameter, owner),
            ),
            NameKind::Namespace => return None,
        };
        Some(HoverInfo { entries: vec![entry], range: occurrence.range })
    }
}

fn method_reference_hover(server: &mut Server, file: &File, editor: &EditorFileData, offset: u32) -> Option<HoverInfo> {
    let methods = methods_at(editor, offset);
    let mut descriptions = Vec::new();
    for method in methods {
        let presentation = if method.dynamic || method.magic {
            None
        } else {
            file.contents
                .get(method.selector_span.0 as usize..method.selector_span.1 as usize)
                .and_then(|name| server.method_presentation(method.receiver_class, name))
        }
        .or_else(|| {
            MethodPresentation::resolve(
                server.codebase(),
                method.method.get_class_name(),
                method.method.get_method_name().as_bytes(),
            )
        });
        if let Some(metadata) = server.codebase().get_method_by_id(&method.method) {
            let description = description::function_like(server, method.method.into(), metadata, presentation.as_ref());
            if !descriptions.contains(&description) {
                descriptions.push(description);
            }
        }
    }
    if let Some(method) = methods.first()
        && !descriptions.is_empty()
    {
        return Some(HoverInfo {
            entries: descriptions,
            range: Range::new(method.selector_span.0, method.selector_span.1),
        });
    }
    None
}

fn member_reference_hover(server: &Server, editor: &EditorFileData, offset: u32) -> Option<HoverInfo> {
    let members = members_at(editor, offset);
    let mut descriptions = Vec::new();
    for member in members {
        if let Some(description) = member
            .lookup_class
            .and_then(|class| member::describe(server, class, member.name, member.kind, member.magic))
            && !descriptions.contains(&description)
        {
            descriptions.push(description);
        }
    }
    if let Some(member) = members.first()
        && !descriptions.is_empty()
    {
        return Some(HoverInfo {
            entries: descriptions,
            range: Range::new(member.selector_span.0, member.selector_span.1),
        });
    }
    None
}

fn declaration_hover(server: &Server, file: FileId, offset: u32) -> Option<HoverInfo> {
    let codebase = server.codebase();
    let contains = |span: Span| span.file_id == file && span.start.offset <= offset && offset < span.end.offset;
    let hover =
        |span: Span, entry| HoverInfo { entries: vec![entry], range: Range::new(span.start.offset, span.end.offset) };
    for signature in &codebase.file_signatures.get(&file)?.ast_nodes {
        if signature.is_function {
            continue;
        }
        let Some(class) = codebase.get_class_like_by_name(signature.name) else { continue };
        for method in signature.children.iter().filter(|child| child.is_function) {
            if let Some(method) = codebase.get_method(class.name.as_bytes(), method.name.as_bytes())
                && let Some(span) = method.name_span.filter(|span| contains(*span))
            {
                let presentation = MethodPresentation::resolve(codebase, class.name, method.name.as_bytes());
                let key = FunctionLikeIdentifier::Method(class.name, method.name);
                return Some(hover(span, description::function_like(server, key, method, presentation.as_ref())));
            }
        }
        for (name, property) in &class.properties {
            if let Some(span) = property.name_span.filter(|span| contains(*span)) {
                return Some(hover(span, member::describe(server, class.name, *name, MemberKind::Property, false)?));
            }
        }
        for constant in class.constants.values() {
            let mut span = constant.span;
            span.end.offset = span.start.offset + constant.name.as_bytes().len() as u32;
            if contains(span) {
                return Some(hover(
                    span,
                    member::describe(server, class.name, constant.name, MemberKind::ClassConstant, false)?,
                ));
            }
        }
        for (name, case) in &class.enum_cases {
            if contains(case.name_span) {
                return Some(hover(
                    case.name_span,
                    member::describe(server, class.name, *name, MemberKind::EnumCase, false)?,
                ));
            }
        }
    }
    None
}
