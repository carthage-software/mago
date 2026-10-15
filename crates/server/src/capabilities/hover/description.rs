//! Declarations and source details for hover descriptions.

use std::fmt::Write;

use mago_bytes::BytesDisplay;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::symbol::SymbolKind;
use mago_codex::ttype::TType;
use mago_codex::ttype::union::TUnion;
use mago_span::Span;

use crate::Server;
use crate::documentation;
use crate::domain::HoverEntry;
use crate::lookup::DocblockAnchor;
use crate::method_presentation::MethodPresentation;
use crate::symbol_resolution::is_hidden_class_like;

use super::types::{NameDisplay, display_union, display_variable, short_name};

pub(super) fn class_like(server: &Server, meta: &ClassLikeMetadata) -> HoverEntry {
    let codebase = server.codebase();
    let kind = match meta.kind {
        SymbolKind::Class => "class",
        SymbolKind::Interface => "interface",
        SymbolKind::Trait => "trait",
        SymbolKind::Enum => "enum",
    };

    let interfaces: Vec<_> = meta
        .direct_parent_interfaces
        .iter()
        .filter(|name| {
            meta.kind != SymbolKind::Enum
                || !(is_hidden_class_like(name.as_bytes()) || matches!(name.as_bytes(), b"unitenum" | b"backedenum"))
        })
        .copied()
        .collect();
    let names = NameDisplay::for_names(
        codebase,
        std::iter::once(meta.name)
            .chain(meta.direct_parent_class)
            .chain(interfaces.iter().copied())
            .chain(meta.direct_trait_names()),
    );

    let mut out = format!(
        "{}{}{}{kind} {}",
        if meta.flags.is_final() && meta.kind == SymbolKind::Class { "final " } else { "" },
        if meta.flags.is_abstract() && meta.kind == SymbolKind::Class { "abstract " } else { "" },
        if meta.flags.is_readonly() { "readonly " } else { "" },
        short_name(meta.original_name.as_bytes())
    );

    if let Some(backing) = &meta.enum_type {
        let _ = write!(out, ": {}", BytesDisplay(backing.get_id().as_bytes()));
    }

    if let Some(parent) = meta.direct_parent_class {
        out.push_str(" extends ");
        out.push_str(&names.class_name(parent));
    }

    let mut interfaces: Vec<_> = interfaces.into_iter().map(|name| names.class_name(name)).collect();
    if !interfaces.is_empty() {
        let keyword = if matches!(meta.kind, SymbolKind::Interface) { " extends " } else { " implements " };
        out.push_str(keyword);
        interfaces.sort_unstable();
        out.push_str(&interfaces.join(", "));
    }

    let mut traits: Vec<_> = meta.direct_trait_names().map(|name| names.class_name(name)).collect();
    if !traits.is_empty() {
        out.push_str(" {\n    use ");
        traits.sort_unstable();
        out.push_str(&traits.join(", "));
        out.push_str(";\n}");
    }

    let mut entry = HoverEntry::for_symbol(meta.original_name.as_bytes(), out);
    entry.documentation = documentation::for_declaration(server, meta.span, DocblockAnchor::DeclarationStart);
    entry
}

pub(super) fn function_like(
    server: &Server,
    key: FunctionLikeIdentifier,
    meta: &FunctionLikeMetadata,
    presentation: Option<&MethodPresentation>,
) -> HoverEntry {
    let codebase = server.codebase();
    let mut signature = String::new();
    if let Some(method) = &meta.method_metadata {
        if presentation.map_or(method.is_final, |method| method.is_final) {
            signature.push_str("final ");
        }
        if method.is_abstract
            && presentation
                .and_then(|method| codebase.get_class_like_by_name(method.owner))
                .is_none_or(|class| class.kind != SymbolKind::Interface)
        {
            signature.push_str("abstract ");
        }
        let _ = write!(signature, "{} ", presentation.map_or(method.visibility, |method| method.visibility));
        if method.is_static {
            signature.push_str("static ");
        }
    }
    signature.push_str("function ");
    if meta.flags.is_by_reference() {
        signature.push('&');
    }
    let name = presentation.map_or(meta.original_name, |method| method.name);
    signature.push_str(&short_name(name.as_bytes()));
    let names = NameDisplay::for_types(
        codebase,
        meta.parameters
            .iter()
            .filter_map(|parameter| parameter.type_metadata.as_ref())
            .chain(meta.return_type_metadata.as_ref())
            .map(|ty| &ty.type_union),
    );
    let mut parameters = Vec::with_capacity(meta.parameters.len());
    for param in &meta.parameters {
        let mut parameter = String::new();
        let displayed_type = param.type_metadata.as_ref().map(|ty| names.union(&ty.type_union));
        crate::capabilities::signature::append_parameter(
            &mut parameter,
            server.database(),
            param,
            displayed_type.as_deref().map(str::as_bytes),
        );
        parameters.push(parameter);
    }
    let return_type = meta.return_type_metadata.as_ref().map(|ty| names.union(&ty.type_union));
    let signature = with_parameters(signature, &parameters, return_type.as_deref());

    let mut entry = match presentation {
        Some(method) => HoverEntry::in_class(method.owner.as_bytes(), signature),
        None => HoverEntry::for_symbol(meta.original_name.as_bytes(), signature),
    };
    entry.documentation = documentation::for_callable(server, key);
    entry
}

pub(super) fn parameter(server: &Server, parameter: &FunctionLikeParameterMetadata, owner: Span) -> HoverEntry {
    let displayed_type = parameter.type_metadata.as_ref().map(|ty| display_union(server.codebase(), &ty.type_union));
    let mut declaration = String::new();
    crate::capabilities::signature::append_parameter(
        &mut declaration,
        server.database(),
        parameter,
        displayed_type.as_deref().map(str::as_bytes),
    );
    let mut entry = HoverEntry::for_symbol(parameter.name.0.as_bytes(), declaration);
    entry.documentation = documentation::for_parameter(server, owner, parameter.name.0.as_bytes());
    entry
}

pub(super) fn variable(server: &Server, name: &[u8], union: &TUnion) -> HoverEntry {
    let variable = BytesDisplay(name);
    let declaration = if name == b"$this" {
        format!("{} {variable}", display_variable(server.codebase(), union, name))
    } else if let Some(value) = union.get_single_literal_int_value() {
        format!("int {variable} = {value}")
    } else if let Some(value) = union.get_single_literal_float_value() {
        format!("float {variable} = {value:?}")
    } else if union.is_true() || union.is_false() {
        format!("bool {variable} = {}", union.is_true())
    } else if let Some(value) = union.get_single_literal_string_value().and_then(|s| std::str::from_utf8(s).ok()) {
        let value = value.replace('\\', "\\\\").replace('\'', "\\'");
        format!("string {variable} = '{value}'")
    } else {
        format!("{} {variable}", display_variable(server.codebase(), union, name))
    };
    HoverEntry::for_symbol(name, declaration)
}

fn with_parameters(mut declaration: String, parameters: &[String], return_type: Option<&str>) -> String {
    let suffix = return_type.map_or_else(String::new, |name| format!(": {name}"));
    let joined = parameters.join(", ");
    let single_line = !joined.contains(['\n', '\r'])
        && declaration.chars().count() + joined.chars().count() + suffix.chars().count() + 2 <= 100;
    declaration.push('(');
    if parameters.is_empty() || single_line {
        declaration.push_str(&joined);
    } else {
        declaration.push_str("\n    ");
        declaration.push_str(&parameters.join(",\n    "));
        declaration.push('\n');
    }
    declaration.push(')');
    declaration.push_str(&suffix);
    declaration
}

#[cfg(test)]
mod tests {
    use super::with_parameters;

    #[test]
    fn signatures_wrap_after_one_hundred_characters() {
        let padding = 100 - "function render(string $value = ''): void".chars().count();
        for extra in [0, 1] {
            let parameter = format!("string $value = '{}'", "é".repeat(padding + extra));
            let actual = with_parameters("function render".into(), std::slice::from_ref(&parameter), Some("void"));
            let expected = if extra == 0 {
                format!("function render({parameter}): void")
            } else {
                format!("function render(\n    {parameter}\n): void")
            };
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn wrapping_keeps_multiline_default_bytes() {
        let parameters = ["string $text = 'first\n  second\nlast'".into(), "int $limit = 2".into()];
        assert_eq!(
            with_parameters("function sample".into(), &parameters, Some("void")),
            "function sample(\n    string $text = 'first\n  second\nlast',\n    int $limit = 2\n): void",
        );
    }
}
