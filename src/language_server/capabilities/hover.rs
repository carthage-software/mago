//! `textDocument/hover`.
//!
//! Resolves the identifier under the cursor via [`mago_names::ResolvedNames`]
//! and renders a Markdown summary sourced from [`mago_codex::metadata::CodebaseMetadata`].

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::symbol::SymbolKind;
use mago_codex::ttype::TType;
use mago_database::file::File as MagoFile;
use mago_names::ResolvedNames;
use tower_lsp::lsp_types::Hover;
use tower_lsp::lsp_types::HoverContents;
use tower_lsp::lsp_types::MarkupContent;
use tower_lsp::lsp_types::MarkupKind;

use crate::language_server::capabilities::lookup;
use crate::language_server::position::range_at_offsets;

pub fn compute(
    codebase: &CodebaseMetadata,
    resolved: &ResolvedNames<'_>,
    file: &MagoFile,
    offset: u32,
) -> Option<Hover> {
    if let Some((start, end, fqcn, _)) = resolved.at_offset(offset) {
        let markdown = render(codebase, fqcn)?;
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: markdown }),
            range: Some(range_at_offsets(file, start, end)),
        });
    }

    let var = lookup::variable_at_offset(file, offset)?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```php\n${}\n```\n\n*variable*", var.name),
        }),
        range: Some(range_at_offsets(file, var.start, var.end)),
    })
}

fn render(codebase: &CodebaseMetadata, fqcn: &str) -> Option<String> {
    if let Some(meta) = codebase.get_class_like(fqcn) {
        return Some(render_class_like(meta));
    }
    if let Some(meta) = codebase.get_function(fqcn) {
        return Some(render_function_like(meta, None));
    }
    if let Some(meta) = codebase.get_constant(fqcn) {
        return Some(format!("```php\nconst {}\n```", meta.name.as_str()));
    }
    None
}

fn render_class_like(meta: &ClassLikeMetadata) -> String {
    let kind = match meta.kind {
        SymbolKind::Class => "class",
        SymbolKind::Interface => "interface",
        SymbolKind::Trait => "trait",
        SymbolKind::Enum => "enum",
    };

    let mut out = format!("```php\n{kind} {}", meta.original_name.as_str());

    if let Some(parent) = meta.direct_parent_class {
        out.push_str(" extends ");
        out.push_str(parent.as_str());
    }

    if !meta.direct_parent_interfaces.is_empty() {
        let keyword = if matches!(meta.kind, SymbolKind::Interface) { " extends " } else { " implements " };
        out.push_str(keyword);
        let names: Vec<&str> = meta.direct_parent_interfaces.iter().map(|a| a.as_str()).collect();
        out.push_str(&names.join(", "));
    }

    out.push_str("\n```");

    if !meta.used_traits.is_empty() {
        out.push_str("\n\n**Uses traits:** ");
        let names: Vec<&str> = meta.used_traits.iter().map(|a| a.as_str()).collect();
        out.push_str(&names.join(", "));
    }

    out
}

fn render_function_like(meta: &FunctionLikeMetadata, method_of: Option<&str>) -> String {
    let mut signature = String::from("```php\nfunction ");
    if let Some(class) = method_of {
        signature.push_str(class);
        signature.push_str("::");
    }
    if let Some(name) = meta.original_name {
        signature.push_str(name.as_str());
    }
    signature.push('(');
    let mut first = true;
    for param in &meta.parameters {
        if !first {
            signature.push_str(", ");
        }
        first = false;
        if let Some(ty) = &param.type_metadata {
            signature.push_str(ty.type_union.get_id().as_str());
            signature.push(' ');
        }
        signature.push_str(param.name.0.as_str());
    }
    signature.push(')');

    if let Some(rt) = &meta.return_type_metadata {
        signature.push_str(": ");
        signature.push_str(rt.type_union.get_id().as_str());
    }

    signature.push_str("\n```");
    signature
}
