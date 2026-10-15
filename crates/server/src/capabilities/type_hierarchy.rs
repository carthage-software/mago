//! Indexed inheritance queries and implementation targets.

use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::symbol::SymbolKind as ClassKind;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_span::Span;
use mago_word::Word;
use mago_word::ascii_lowercase_word;

use crate::Range;
use crate::Server;
use crate::SymbolKind;
use crate::SymbolLocation;
use crate::semantic_index::SymbolKind as IndexedKind;
use crate::symbol_resolution::is_hidden_class_like;
use crate::symbol_resolution::methods_at;

const HIDDEN_DEPTH: u8 = 4;

/// A class-like declaration and its stable, canonical name.
#[derive(Debug, Clone)]
pub struct TypeHierarchyItem {
    pub name: String,
    pub key: Vec<u8>,
    pub kind: SymbolKind,
    pub file: FileId,
    pub range: Range,
    pub selection_range: Range,
}

impl Server {
    /// Prepare a hierarchy at a class-like declaration or reference.
    pub fn prepare_type_hierarchy(&mut self, file: FileId, offset: u32) -> Option<TypeHierarchyItem> {
        let analysis = self.file_analysis_for(file)?;
        let index = analysis.semantic();
        let occurrence = index.at_offset(offset)?;
        let symbol = &index.symbols[occurrence.symbol as usize];
        if symbol.kind != IndexedKind::ClassLike {
            return None;
        }
        self.type_hierarchy_item(&symbol.name)
    }

    /// Find a declaration by its canonical class-like key.
    #[must_use]
    pub fn type_hierarchy_item(&self, key: &[u8]) -> Option<TypeHierarchyItem> {
        hierarchy_item(self.codebase(), key)
    }

    /// Direct nominal parents. Trait use does not establish a subtype relation.
    #[must_use]
    pub fn type_supertypes(&self, key: &[u8]) -> Option<Vec<TypeHierarchyItem>> {
        related_types(self.codebase(), key, true)
    }

    /// Direct nominal children, from the populated descendant index.
    #[must_use]
    pub fn type_subtypes(&self, key: &[u8]) -> Option<Vec<TypeHierarchyItem>> {
        related_types(self.codebase(), key, false)
    }

    /// Concrete descendants or distinct overriding method bodies.
    pub fn get_implementations(&mut self, file: FileId, offset: u32) -> Vec<SymbolLocation> {
        let mut out = Vec::new();
        if let Some(editor) = self.editor_file(file)
            && let Ok(file) = self.database().get(&file)
        {
            for method in methods_at(&editor, offset).iter().filter(|m| !m.dynamic && !m.magic && !m.uncertain) {
                let Some(name) = file.contents.get(method.selector_span.0 as usize..method.selector_span.1 as usize)
                else {
                    continue;
                };
                let lookup = MethodIdentifier::new(method.receiver_class, ascii_lowercase_word(name));
                out.extend(method_implementations(self.codebase(), lookup));
            }
        }
        if let Some(method) = declared_method_at(self.codebase(), file, offset) {
            out.extend(method_implementations(self.codebase(), method));
        } else if let Some(item) = self.prepare_type_hierarchy(file, offset) {
            out.extend(class_implementations(self.codebase(), &item.key));
        }
        out.sort_unstable_by_key(|location| (location.file, location.range.start, location.range.end));
        out.dedup();
        out
    }
}

fn hierarchy_item(codebase: &CodebaseMetadata, key: &[u8]) -> Option<TypeHierarchyItem> {
    let meta = codebase.get_class_like(key)?;
    let selection = meta.name_span?;
    Some(TypeHierarchyItem {
        name: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(),
        key: meta.name.as_bytes().to_vec(),
        kind: match meta.kind {
            ClassKind::Class => SymbolKind::Class,
            ClassKind::Interface => SymbolKind::Interface,
            ClassKind::Trait => SymbolKind::Trait,
            ClassKind::Enum => SymbolKind::Enum,
        },
        file: meta.span.file_id,
        range: Range::new(meta.span.start.offset, meta.span.end.offset),
        selection_range: Range::new(selection.start.offset, selection.end.offset),
    })
}

fn related_types(codebase: &CodebaseMetadata, key: &[u8], parents: bool) -> Option<Vec<TypeHierarchyItem>> {
    let meta = codebase.get_class_like(key)?;
    let mut items = Vec::new();
    if !meta.kind.is_trait() {
        collect_related(codebase, meta, parents, HIDDEN_DEPTH, &mut items);
    }
    items.sort_by(|a, b| a.key.cmp(&b.key));
    items.dedup_by(|a, b| a.key == b.key);
    Some(items)
}

fn collect_related(
    codebase: &CodebaseMetadata,
    meta: &ClassLikeMetadata,
    parents: bool,
    depth: u8,
    items: &mut Vec<TypeHierarchyItem>,
) {
    if parents {
        for name in meta.direct_parent_class.iter().chain(&meta.direct_parent_interfaces) {
            push_related(codebase, *name, parents, depth, items);
        }
    } else if let Some(children) = codebase.direct_classlike_descendants.get(&meta.name) {
        for name in children {
            push_related(codebase, *name, parents, depth, items);
        }
    }
}

fn push_related(codebase: &CodebaseMetadata, name: Word, parents: bool, depth: u8, items: &mut Vec<TypeHierarchyItem>) {
    if !is_hidden_class_like(name.as_bytes()) {
        items.extend(hierarchy_item(codebase, name.as_bytes()));
    } else if depth > 0
        && let Some(hidden) = codebase.get_class_like_by_name(name)
    {
        collect_related(codebase, hidden, parents, depth - 1, items);
    }
}

fn class_implementations(codebase: &CodebaseMetadata, key: &[u8]) -> Vec<SymbolLocation> {
    let Some(meta) = codebase.get_class_like(key) else { return Vec::new() };
    codebase
        .all_class_like_descendants
        .get(&meta.name)
        .into_iter()
        .flatten()
        .filter_map(|name| {
            let child = codebase.get_class_like_by_name(*name)?;
            if !matches!(child.kind, ClassKind::Class | ClassKind::Enum) || child.flags.is_abstract() {
                return None;
            }
            child.name_span.map(location)
        })
        .collect()
}

fn method_implementations(codebase: &CodebaseMetadata, method: MethodIdentifier) -> Vec<SymbolLocation> {
    let Some(source) = codebase.get_declaring_method_by_id(&method) else { return Vec::new() };
    if codebase.get_method_visibility_by_id(&method).is_some_and(|visibility| visibility.is_private())
        || codebase.method_is_final(method.get_class_name().as_bytes(), method.get_method_name().as_bytes())
    {
        return Vec::new();
    }
    let Some(receiver) = codebase.get_class_like_by_name(method.get_class_name()) else { return Vec::new() };
    codebase
        .all_class_like_descendants
        .get(&receiver.name)
        .into_iter()
        .flatten()
        .filter_map(|name| {
            let body = codebase.get_declaring_method(name.as_bytes(), method.get_method_name().as_bytes())?;
            let metadata = body.method_metadata.as_ref()?;
            let span = body.name_span?;
            if metadata.is_abstract
                || codebase
                    .get_method_visibility_by_id(&MethodIdentifier::new(*name, method.get_method_name()))
                    .is_some_and(|visibility| visibility.is_private())
                || body.flags.is_magic_method()
                || Some(span) == source.name_span
            {
                return None;
            }
            Some(location(span))
        })
        .collect()
}

fn declared_method_at(codebase: &CodebaseMetadata, file: FileId, offset: u32) -> Option<MethodIdentifier> {
    for class in &codebase.file_signatures.get(&file)?.ast_nodes {
        if class.is_function {
            continue;
        }
        for method in class.children.iter().filter(|child| child.is_function) {
            let id = MethodIdentifier::new(class.name, method.name);
            if codebase
                .get_method_by_id(&id)
                .and_then(|m| m.name_span)
                .is_some_and(|span| span.start.offset <= offset && offset < span.end.offset)
            {
                return Some(id);
            }
        }
    }
    None
}

fn location(span: Span) -> SymbolLocation {
    SymbolLocation { file: span.file_id, range: Range::new(span.start.offset, span.end.offset) }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use mago_allocator::LocalArena;
    use mago_codex::populator::populate_codebase;
    use mago_codex::reference::SymbolReferences;
    use mago_codex::scanner::scan_program;
    use mago_codex::signature_builder::build_file_signature;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_php_version::PHPVersion;
    use mago_syntax::parser::parse_file;
    use mago_word::word;

    use super::*;

    fn codebase(source: &str) -> (File, CodebaseMetadata) {
        let file = File::ephemeral(b"types.php".as_slice().into(), source.as_bytes().to_vec().into());
        let arena = LocalArena::new();
        let program = parse_file(&arena, &file);
        assert!(program.errors.is_empty(), "fixture must parse: {:?}", program.errors);
        let names = NameResolver::new(&arena).resolve(program);
        let mut codebase = scan_program(&arena, &file, program, &names, PHPVersion::PHP85);
        codebase.set_file_signature(file.id, build_file_signature(&file, program, &names));
        populate_codebase(
            &mut codebase,
            &mut SymbolReferences::new(),
            mago_word::WordSet::default(),
            foldhash::HashSet::default(),
        );
        (file, codebase)
    }

    #[test]
    fn hierarchy_uses_direct_edges_and_exact_names() {
        let (file, codebase) = codebase(
            "<?php interface Root {} interface Mid extends Root {} interface Extra {} class Base implements Mid {} class Child extends Base implements Extra {} enum Flag implements Root { case On; } trait Helper {} class User { use Helper; } class Inherited extends User {}",
        );
        let names = |items: Vec<TypeHierarchyItem>| items.into_iter().map(|item| item.name).collect::<Vec<_>>();
        assert_eq!(names(related_types(&codebase, b"ROOT", false).unwrap()), ["Flag", "Mid"]);
        assert_eq!(names(related_types(&codebase, b"Child", true).unwrap()), ["Base", "Extra"]);
        assert_eq!(names(related_types(&codebase, b"Helper", false).unwrap()), Vec::<String>::new());
        let item = hierarchy_item(&codebase, b"Flag").unwrap();
        assert_eq!(item.kind, SymbolKind::Enum);
        assert_eq!(item.key, b"flag");
        assert!(item.range.start <= item.selection_range.start && item.selection_range.end <= item.range.end);
        assert_eq!(&file.contents[item.selection_range.start as usize..item.selection_range.end as usize], b"Flag");
        assert!(related_types(&codebase, b"Missing", true).is_none());
        assert_eq!(class_implementations(&codebase, b"Helper").len(), 2);
    }

    #[test]
    fn hierarchy_replaces_hidden_contracts_with_public_relatives() {
        let (_, codebase) = codebase(
            "<?php interface UnitEnum {} interface BackedEnum extends UnitEnum {}
interface __INTERNAL_DO_NOT_USE__StringBackedEnum extends BackedEnum {}
interface __INTERNAL_DO_NOT_USE__IntBackedEnum extends BackedEnum {}
interface Contract {} enum Suit: string implements Contract { case H = 'h'; }
enum Level: int { case Low = 1; } enum Flag { case On; }
interface __INTERNAL_DO_NOT_USE__A extends __INTERNAL_DO_NOT_USE__B {}
interface __INTERNAL_DO_NOT_USE__B extends __INTERNAL_DO_NOT_USE__A {}
class Looped implements __INTERNAL_DO_NOT_USE__A {}",
        );
        let names = |key: &[u8], parents: bool| {
            related_types(&codebase, key, parents).unwrap().into_iter().map(|item| item.name).collect::<Vec<_>>()
        };
        assert_eq!(names(b"Suit", true), ["BackedEnum", "Contract"]);
        assert_eq!(names(b"Level", true), ["BackedEnum"]);
        assert_eq!(names(b"Flag", true), ["UnitEnum"]);
        assert_eq!(names(b"BackedEnum", false), ["Level", "Suit"]);
        assert_eq!(names(b"UnitEnum", false), ["BackedEnum", "Flag"]);
        assert_eq!(names(b"__internal_do_not_use__stringbackedenum", true), ["BackedEnum"]);
        assert_eq!(names(b"__internal_do_not_use__stringbackedenum", false), ["Suit"]);
        assert_eq!(names(b"Looped", true), Vec::<String>::new());
    }

    #[test]
    fn implementations_skip_abstract_types_and_inherited_only_methods() {
        let (file, codebase) = codebase(
            "<?php interface Contract { public function run(); } abstract class AbstractType implements Contract {} class Base { public function run() {} private function hidden() {} } class Via extends Base implements Contract {} class Child extends Via {} class Override extends Base { public function run() {} public function hidden() {} }",
        );
        let targets = class_implementations(&codebase, b"Contract");
        assert_eq!(targets.len(), 2);
        let base = codebase.get_method(b"Base", b"run").unwrap().name_span.unwrap();
        let contract = MethodIdentifier::new(word("contract"), word("run"));
        let bodies = method_implementations(&codebase, contract);
        assert!(!bodies.is_empty(), "the inherited concrete body implements the interface");
        assert!(bodies.iter().all(|body| *body == location(base)));
        let base_id = MethodIdentifier::new(word("base"), word("run"));
        let overrides = method_implementations(&codebase, base_id);
        assert_eq!(overrides, [location(codebase.get_method(b"Override", b"run").unwrap().name_span.unwrap())]);
        assert_eq!(declared_method_at(&codebase, file.id, base.start.offset), Some(base_id));
        assert_eq!(declared_method_at(&codebase, file.id, base.end.offset), None);
        assert!(method_implementations(&codebase, MethodIdentifier::new(word("base"), word("hidden"))).is_empty());
    }
}
