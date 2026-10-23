//! Copy checked native signatures for missing inherited methods.

use foldhash::{HashMap, HashSet};
use mago_allocator::LocalArena;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_database::DatabaseReader;
use mago_names::resolver::NameResolver;
use mago_syntax::cst::*;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::Server;

mod native_signature;

impl Server {
    /// Complete throwing stubs for missing inherited methods, or `None` when any obligation is unsafe.
    #[must_use]
    pub fn get_method_stubs(&self, name: &[u8]) -> Option<Vec<String>> {
        let class = self.codebase().get_class_like(name)?;
        if !(class.kind.is_class() || class.kind.is_enum()) || class.has_incomplete_hierarchy() || generic(class) {
            return None;
        }
        for parent in &class.all_parent_interfaces {
            let Some(parent) = self.codebase().get_class_like_by_name(*parent) else {
                continue;
            };
            for (name, id) in &parent.declaring_method_ids {
                let actual = self.codebase().get_declaring_method(class.name.as_bytes(), name.as_bytes())?;
                if class.kind.is_enum()
                    && actual.name_span == Some(class.span)
                    && parent.flags.is_built_in()
                    && parent.flags.is_enum_interface()
                {
                    continue;
                }
                if !actual.method_metadata.as_ref()?.is_abstract
                    && !same_contract(actual, self.codebase().get_method_by_id(id)?)
                {
                    return None;
                }
            }
        }
        let mut groups = Vec::new();
        for (name, id) in &class.declaring_method_ids {
            let method = self.codebase().get_method_by_id(id)?;
            if !method.method_metadata.as_ref()?.is_abstract {
                continue;
            }
            if id.get_class_name() == class.name {
                return None;
            }
            let mut ids = HashSet::from_iter([*id]);
            ids.extend(class.overridden_method_ids.get(name).into_iter().flat_map(|ids| ids.values()).copied());
            for parent in &class.all_parent_interfaces {
                let Some(parent) = self.codebase().get_class_like_by_name(*parent) else {
                    continue;
                };
                ids.extend(parent.declaring_method_ids.get(name).copied());
            }
            groups.push((*name, ids));
        }
        if groups.is_empty() {
            return Some(Vec::new());
        }
        groups.sort_unstable_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
        let needed: HashSet<_> = groups.iter().flat_map(|(_, ids)| ids.iter()).copied().collect();
        let mut by_file: HashMap<_, HashMap<_, _>> = HashMap::default();
        for id in &needed {
            let span = self.codebase().get_method_by_id(id)?.name_span?;
            by_file.entry(span.file_id).or_default().insert(span, *id);
        }
        let mut stubs = HashMap::default();
        for (file_id, ids) in by_file {
            let file = self.database().get(&file_id).ok()?;
            let arena = LocalArena::new();
            let program = parse_file_borrowed_with_settings(&arena, &file, self.parser_settings());
            if program.has_errors() {
                return None;
            }
            let names = NameResolver::new(&arena).resolve(program);
            let mut nodes = vec![Node::Program(program)];
            while let Some(node) = nodes.pop() {
                if let Node::Method(method) = node
                    && let Some(id) = ids.get(&method.name.span)
                {
                    stubs.insert(*id, native_signature::render(self, class, *id, &file, &names, method)?);
                }
                node.visit_children(|child| nodes.push(child));
            }
        }
        if stubs.len() != needed.len() {
            return None;
        }
        groups
            .into_iter()
            .map(|(_, ids)| {
                let first = stubs.get(ids.iter().next()?)?;
                ids.iter().all(|id| stubs.get(id) == Some(first)).then(|| first.clone())
            })
            .collect()
    }
}

fn generic(class: &ClassLikeMetadata) -> bool {
    !class.template_types.is_empty()
        || class.template_extended_parameters.values().any(|parameters| !parameters.is_empty())
}

fn same_contract(actual: &FunctionLikeMetadata, required: &FunctionLikeMetadata) -> bool {
    let (Some(actual_method), Some(required_method)) = (&actual.method_metadata, &required.method_metadata) else {
        return false;
    };
    actual_method.is_static == required_method.is_static
        && actual_method.visibility.is_public()
        && actual.flags.is_by_reference() == required.flags.is_by_reference()
        && actual.return_type_declaration_metadata.as_ref().map(|ty| &ty.type_union)
            == required.return_type_declaration_metadata.as_ref().map(|ty| &ty.type_union)
        && actual.parameters.len() == required.parameters.len()
        && actual.parameters.iter().zip(&required.parameters).all(|(left, right)| {
            left.name == right.name
                && left.flags.is_by_reference() == right.flags.is_by_reference()
                && left.flags.is_variadic() == right.flags.is_variadic()
                && left.flags.has_default() == right.flags.has_default()
                && left.type_declaration_metadata.as_ref().map(|ty| &ty.type_union)
                    == right.type_declaration_metadata.as_ref().map(|ty| &ty.type_union)
        })
}

#[cfg(test)]
mod tests;
