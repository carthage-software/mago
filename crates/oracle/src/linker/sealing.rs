use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::vec::Vec;

use crate::id::SymbolId;
use crate::id::SymbolIdBuildHasher;
use crate::linker::ClassLikeRef;
use crate::linker::index::sorted_unique_ids;
use crate::linker::types::reference_name;
use crate::path::Path;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::ClassLikeSymbol;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::inheritance::Provenance;

/// Resolves `@sealed` declarations into the closed list of permitted inheritors
/// on each sealed class-like, and records the sealed parent on every permitted
/// child so the lattice can answer both directions of the relation. The
/// child→parent map lives on the scratch arena; only the final slices reach the
/// output arena.
pub(crate) fn resolve_sealing<'arena, A, S, St, Ex>(
    arena: &'arena A,
    scratch: &S,
    table: &mut SymbolTable<'arena, A>,
    class_likes: &HashMap<'_, SymbolId, ClassLikeRef<'_, 'arena, St, Ex>, S, SymbolIdBuildHasher>,
) where
    A: Arena,
    S: Arena,
    St: Copy,
    Ex: Copy,
{
    let mut parents: HashMap<'_, SymbolId, Vec<'_, SymbolId, S>, S, SymbolIdBuildHasher> =
        HashMap::with_hasher_in(SymbolIdBuildHasher, scratch);

    for (&id, reference) in class_likes {
        let Some(annotation) = reference.annotation() else {
            continue;
        };
        if annotation.sealings.is_empty() {
            continue;
        }

        let mut permitted = Vec::new_in(scratch);
        for sealing in annotation.sealings {
            for r#type in sealing.types {
                let name = reference_name(&r#type.kind);
                parents.entry(SymbolId::class_like(name)).or_insert_with(|| Vec::new_in(scratch)).push(id);
                permitted.push(InheritedType {
                    span: r#type.span,
                    target: Path::class_like(arena, name),
                    provenance: Provenance::Direct,
                    arguments: &[],
                });
            }
        }

        let Some(symbol) = table.get_class_like(id) else {
            continue;
        };
        let permitted = arena.alloc_slice_fill_iter(permitted);
        if let Some(updated) = set_permitted_inheritors(arena, symbol, permitted) {
            table.class_likes.insert(id, updated);
        }
    }

    for (child, sealed_parents) in parents {
        let Some(symbol) = table.get_class_like(child) else {
            continue;
        };
        let sealed_parents = sorted_unique_ids(arena, sealed_parents);
        if let Some(updated) = set_sealed_parents(arena, symbol, sealed_parents) {
            table.class_likes.insert(child, updated);
        }
    }
}

/// Stores the permitted-inheritor list on the kinds that can be sealed (class,
/// interface); `None` for the others.
fn set_permitted_inheritors<'arena, A>(
    arena: &'arena A,
    symbol: ClassLikeSymbol<'arena>,
    permitted: &'arena [InheritedType<'arena>],
) -> Option<ClassLikeSymbol<'arena>>
where
    A: Arena,
{
    match symbol {
        ClassLikeSymbol::Class(class) => {
            let mut updated = *class;
            updated.permitted_inheritors = permitted;
            Some(ClassLikeSymbol::Class(arena.alloc(updated)))
        }
        ClassLikeSymbol::Interface(interface) => {
            let mut updated = *interface;
            updated.permitted_inheritors = permitted;
            Some(ClassLikeSymbol::Interface(arena.alloc(updated)))
        }
        ClassLikeSymbol::Trait(_) | ClassLikeSymbol::Enum(_) | ClassLikeSymbol::AnonymousClass(_) => None,
    }
}

/// Records the sealed parents on the kinds that carry the field (class,
/// interface, enum, anonymous class); `None` for traits.
fn set_sealed_parents<'arena, A>(
    arena: &'arena A,
    symbol: ClassLikeSymbol<'arena>,
    sealed_parents: &'arena [SymbolId],
) -> Option<ClassLikeSymbol<'arena>>
where
    A: Arena,
{
    match symbol {
        ClassLikeSymbol::Class(class) => {
            let mut updated = *class;
            updated.sealed_parents = sealed_parents;
            Some(ClassLikeSymbol::Class(arena.alloc(updated)))
        }
        ClassLikeSymbol::Interface(interface) => {
            let mut updated = *interface;
            updated.sealed_parents = sealed_parents;
            Some(ClassLikeSymbol::Interface(arena.alloc(updated)))
        }
        ClassLikeSymbol::Enum(r#enum) => {
            let mut updated = *r#enum;
            updated.sealed_parents = sealed_parents;
            Some(ClassLikeSymbol::Enum(arena.alloc(updated)))
        }
        ClassLikeSymbol::AnonymousClass(anonymous_class) => {
            let mut updated = *anonymous_class;
            updated.sealed_parents = sealed_parents;
            Some(ClassLikeSymbol::AnonymousClass(arena.alloc(updated)))
        }
        ClassLikeSymbol::Trait(_) => None,
    }
}
