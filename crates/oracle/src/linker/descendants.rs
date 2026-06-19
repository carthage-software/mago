use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::collections::HashSet;
use mago_allocator::vec::Vec;

use crate::id::SymbolId;
use crate::id::SymbolIdBuildHasher;
use crate::linker::index::sorted_unique_ids;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::ClassLikeSymbol;

/// Computes the direct and transitive descendant indices for every class-like
/// from the inheritance edges already lowered onto each symbol, and stores the
/// id-sorted slices the table binary-searches. Every intermediate buffer lives
/// on the scratch arena; only the final sorted slices reach the output arena.
pub(crate) fn build_descendants<'arena, S, A>(arena: &'arena A, scratch: &S, table: &mut SymbolTable<'arena, A>)
where
    S: Arena,
    A: Arena,
{
    let mut direct: HashMap<'_, SymbolId, Vec<'_, SymbolId, S>, S, SymbolIdBuildHasher> =
        HashMap::with_hasher_in(SymbolIdBuildHasher, scratch);
    for (&child, symbol) in &table.class_likes {
        for_each_parent(symbol, |parent| {
            direct.entry(parent).or_insert_with(|| Vec::new_in(scratch)).push(child);
        });
    }

    for (&parent, children) in &direct {
        let mut closure = Vec::new_in(scratch);
        let mut seen = HashSet::with_hasher_in(SymbolIdBuildHasher, scratch);
        let mut stack = Vec::new_in(scratch);
        stack.extend(children.iter().copied());
        while let Some(node) = stack.pop() {
            if !seen.insert(node) {
                continue;
            }

            closure.push(node);
            if let Some(grandchildren) = direct.get(&node) {
                stack.extend(grandchildren.iter().copied());
            }
        }

        table.all_descendants.insert(parent, sorted_unique_ids(arena, closure));
    }

    for (parent, children) in direct {
        table.direct_descendants.insert(parent, sorted_unique_ids(arena, children));
    }
}

/// Invokes `visit` with the id of every class-like a symbol directly inherits
/// from (parent class, interfaces, used traits), without allocating.
pub(crate) fn for_each_parent(symbol: &ClassLikeSymbol<'_>, mut visit: impl FnMut(SymbolId)) {
    match symbol {
        ClassLikeSymbol::Class(class) => {
            if let Some(extends) = &class.extends {
                visit(extends.target.id);
            }
            class.implements.iter().for_each(|edge| visit(edge.target.id));
            class.uses.iter().for_each(|edge| visit(edge.target.id));
        }
        ClassLikeSymbol::Interface(interface) => {
            interface.extends.iter().for_each(|edge| visit(edge.target.id));
        }
        ClassLikeSymbol::Trait(r#trait) => {
            r#trait.uses.iter().for_each(|edge| visit(edge.target.id));
        }
        ClassLikeSymbol::Enum(r#enum) => {
            r#enum.implements.iter().for_each(|edge| visit(edge.target.id));
            r#enum.uses.iter().for_each(|edge| visit(edge.target.id));
        }
        ClassLikeSymbol::AnonymousClass(anonymous_class) => {
            if let Some(extends) = &anonymous_class.extends {
                visit(extends.target.id);
            }

            anonymous_class.implements.iter().for_each(|edge| visit(edge.target.id));
            anonymous_class.uses.iter().for_each(|edge| visit(edge.target.id));
        }
    }
}
