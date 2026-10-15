//! Document outlines collected during the cached analysis pass.

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Node;
use mago_syntax::cst::Program;

use crate::DocumentSymbolItem;
use crate::Range;
use crate::Server;
use crate::SymbolKind;

impl Server {
    /// Return declarations in source order, nested by their containing declaration.
    pub fn get_document_symbols(&mut self, file_id: FileId) -> Vec<DocumentSymbolItem> {
        self.file_analysis_for(file_id).map(|analysis| analysis.document_symbols.clone()).unwrap_or_default()
    }
}

pub(crate) fn collect(program: &Program<'_>) -> Vec<DocumentSymbolItem> {
    let mut symbols = Vec::new();
    visit(Node::Program(program), program.source_text, &mut symbols);
    symbols
}

fn symbol(source: &[u8], span: Span, name: Span, kind: SymbolKind) -> DocumentSymbolItem {
    DocumentSymbolItem {
        name: String::from_utf8_lossy(&source[name.start.offset as usize..name.end.offset as usize]).into_owned(),
        kind,
        range: Range::new(span.start.offset, span.end.offset),
        selection_range: Range::new(name.start.offset, name.end.offset),
        children: Vec::new(),
    }
}

fn visit(node: Node<'_, '_>, source: &[u8], out: &mut Vec<DocumentSymbolItem>) {
    let (name, kind) = match node {
        Node::Namespace(n) => (n.name.as_ref().map_or(n.namespace.span(), HasSpan::span), SymbolKind::Namespace),
        Node::Class(n) => (n.name.span(), SymbolKind::Class),
        Node::AnonymousClass(n) => (n.class.span(), SymbolKind::Class),
        Node::Interface(n) => (n.name.span(), SymbolKind::Interface),
        Node::Trait(n) => (n.name.span(), SymbolKind::Trait),
        Node::Enum(n) => (n.name.span(), SymbolKind::Enum),
        Node::Function(n) => (n.name.span(), SymbolKind::Function),
        Node::Method(n) => {
            let constructor = n.name.value.eq_ignore_ascii_case(b"__construct");
            if constructor {
                for parameter in n.parameter_list.parameters.iter().filter(|p| p.is_promoted_property()) {
                    out.push(symbol(source, parameter.span(), parameter.variable.span(), SymbolKind::Property));
                }
            }
            (n.name.span(), if constructor { SymbolKind::Constructor } else { SymbolKind::Method })
        }
        Node::EnumCase(n) => (n.item.name().span(), SymbolKind::EnumMember),
        Node::Property(n) => {
            for variable in n.variables() {
                out.push(symbol(source, n.span(), variable.span(), SymbolKind::Property));
            }
            return;
        }
        Node::Constant(n) => {
            for item in &n.items {
                out.push(symbol(source, n.span(), item.name.span(), SymbolKind::Constant));
            }
            return;
        }
        Node::ClassLikeConstant(n) => {
            for item in &n.items {
                out.push(symbol(source, n.span(), item.name.span(), SymbolKind::Constant));
            }
            return;
        }
        _ => {
            node.visit_children(|child| visit(child, source, out));
            return;
        }
    };
    if name.length() == 0 {
        return;
    }
    let mut item = symbol(source, node.span(), name, kind);
    match node {
        Node::Namespace(n) if n.name.is_none() => item.name = "(global)".into(),
        Node::AnonymousClass(_) => item.name = "(anonymous class)".into(),
        _ => {}
    }
    node.visit_children(|child| visit(child, source, &mut item.children));
    item.children.sort_by_key(|child| child.range.start);
    out.push(item);
}
