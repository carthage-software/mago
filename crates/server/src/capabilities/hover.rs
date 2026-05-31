//! `get_context`: resolve the identifier (or variable) under the cursor and
//! render a Markdown summary of it for hover.

use mago_bytes::BytesDisplay;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::symbol::SymbolKind;
use mago_codex::ttype::TType;
use mago_database::DatabaseReader;
use mago_database::file::FileId;

use crate::Server;
use crate::domain::HoverInfo;
use crate::domain::Range;
use crate::lookup;

impl Server {
    /// Hover context for the token covering `offset` in `file_id`: rendered
    /// markdown plus the token's span. Resolves named symbols against the
    /// codebase, and falls back to a plain summary for `$variable`s.
    pub fn get_context(&mut self, file_id: FileId, offset: u32) -> Option<HoverInfo> {
        let file = self.database().get(&file_id).ok()?;
        let analysis = self.file_analysis_for(file_id)?;

        if let Some((start, end, fqcn, _)) = analysis.resolved().at_offset(offset) {
            let markdown = render(self.codebase(), fqcn)?;
            return Some(HoverInfo { markdown, range: Range::new(start, end) });
        }

        let var = lookup::variable_at_offset(&file, offset)?;
        Some(HoverInfo {
            markdown: format!("```php\n${}\n```\n\n*variable*", BytesDisplay(var.name)),
            range: Range::new(var.start, var.end),
        })
    }
}

fn render(codebase: &CodebaseMetadata, fqcn: &[u8]) -> Option<String> {
    if let Some(meta) = codebase.get_class_like(fqcn) {
        return Some(render_class_like(meta));
    }
    if let Some(meta) = codebase.get_function(fqcn) {
        return Some(render_function_like(meta, None));
    }
    if let Some(meta) = codebase.get_constant(fqcn) {
        return Some(format!("```php\nconst {}\n```", BytesDisplay(meta.name.as_bytes())));
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

    let mut out = format!("```php\n{kind} {}", BytesDisplay(meta.original_name.as_bytes()));

    if let Some(parent) = meta.direct_parent_class {
        out.push_str(" extends ");
        out.push_str(&String::from_utf8_lossy(parent.as_bytes()));
    }

    if !meta.direct_parent_interfaces.is_empty() {
        let keyword = if matches!(meta.kind, SymbolKind::Interface) { " extends " } else { " implements " };
        out.push_str(keyword);
        let names: Vec<String> =
            meta.direct_parent_interfaces.iter().map(|a| String::from_utf8_lossy(a.as_bytes()).into_owned()).collect();
        out.push_str(&names.join(", "));
    }

    out.push_str("\n```");

    if !meta.used_traits.is_empty() {
        out.push_str("\n\n**Uses traits:** ");
        let names: Vec<String> =
            meta.used_traits.iter().map(|a| String::from_utf8_lossy(a.as_bytes()).into_owned()).collect();
        out.push_str(&names.join(", "));
    }

    out
}

fn render_function_like(meta: &FunctionLikeMetadata, method_of: Option<&[u8]>) -> String {
    use std::fmt::Write;
    let mut signature = String::from("```php\nfunction ");
    if let Some(class) = method_of {
        let _ = write!(signature, "{}", BytesDisplay(class));
        signature.push_str("::");
    }
    let _ = write!(signature, "{}", BytesDisplay(meta.original_name.as_bytes()));
    signature.push('(');
    let mut first = true;
    for param in &meta.parameters {
        if !first {
            signature.push_str(", ");
        }
        first = false;
        if let Some(ty) = &param.type_metadata {
            let _ = write!(signature, "{}", BytesDisplay(ty.type_union.get_id().as_bytes()));
            signature.push(' ');
        }
        let _ = write!(signature, "{}", BytesDisplay(param.name.0.as_bytes()));
    }
    signature.push(')');

    if let Some(rt) = &meta.return_type_metadata {
        signature.push_str(": ");
        let _ = write!(signature, "{}", BytesDisplay(rt.type_union.get_id().as_bytes()));
    }

    signature.push_str("\n```");
    signature
}
