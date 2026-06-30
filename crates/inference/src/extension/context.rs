use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_flags::U8Flags;
use mago_oracle::assertion::Assertion;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::KnownElement;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::array::ListAtom;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::float::LiteralFloat;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::well_known::EMPTY_ARRAY;
use mago_oracle::ty::well_known::NON_NEGATIVE_INT;
use mago_oracle::ty::well_known::TYPE_FLOAT;
use mago_oracle::ty::well_known::TYPE_NEVER;
use ordered_float::OrderedFloat;

use crate::reconciler::reconcile;

/// What an extension may read and build while refining an expression. Holds the
/// real [`TypeBuilder`] directly (no trait object), so the type-construction
/// helpers monomorphize.
pub struct ExtensionContext<'ctx, 'source, 'arena, A: Arena> {
    builder: &'ctx mut TypeBuilder<'source, 'arena, A, A>,
    symbols: &'ctx SymbolTable<'arena, A>,
    namespace: &'ctx [u8],
}

impl<'ctx, 'source, 'arena, A: Arena> ExtensionContext<'ctx, 'source, 'arena, A> {
    pub(crate) fn new(
        builder: &'ctx mut TypeBuilder<'source, 'arena, A, A>,
        symbols: &'ctx SymbolTable<'arena, A>,
        namespace: &'ctx [u8],
    ) -> Self {
        Self { builder, symbols, namespace }
    }

    /// The union of the given atoms.
    pub fn union(&mut self, atoms: &[Atom<'arena>]) -> Type<'arena> {
        self.builder.union_of(atoms)
    }

    /// A literal integer type.
    pub fn int(&mut self, value: i64) -> Type<'arena> {
        self.builder.union_of(&[Atom::Int(IntAtom::Literal(value))])
    }

    /// A literal string type.
    pub fn string(&mut self, value: &[u8]) -> Type<'arena> {
        let atom = self.builder.string_literal(value);

        self.builder.union_of(&[atom])
    }

    /// An integer range type; `None` bounds are open (`Some(0), None` is
    /// `non-negative-int`, `Some(1), None` is `positive-int`).
    pub fn int_range(&mut self, lower: Option<i64>, upper: Option<i64>) -> Type<'arena> {
        let atom = self.builder.int_range(lower, upper);

        self.builder.union_of(&[atom])
    }

    /// A sealed list (`list{0: ..., 1: ...}`) of the given element types in order.
    pub fn list(&mut self, elements: &[Type<'arena>], non_empty: bool) -> Type<'arena> {
        let arena = self.builder.arena();
        let mut known = Vec::with_capacity_in(elements.len(), arena);
        for (index, &value) in elements.iter().enumerate() {
            known.push(KnownElement { index: index as u32, value, optional: false });
        }

        let atom = self.builder.sealed_list(&known, non_empty);

        self.builder.union_of(&[atom])
    }

    /// An unsealed list of `element` (`list<T>` / `non-empty-list<T>`).
    pub fn list_of(&mut self, element: Type<'arena>, non_empty: bool) -> Type<'arena> {
        let atom = self.builder.list_of(element, non_empty);

        self.builder.union_of(&[atom])
    }

    pub fn truthy(&mut self, ty: Type<'arena>) -> Type<'arena> {
        reconcile(self.builder, self.symbols, Assertion::Truthy, ty)
    }

    pub fn keyed(&mut self, key: Type<'arena>, value: Type<'arena>, non_empty: bool) -> Type<'arena> {
        let atom = self.builder.keyed_unsealed(key, value, non_empty);

        self.builder.union_of(&[atom])
    }

    pub fn filter_array(&mut self, source: Type<'arena>, narrow: bool) -> Option<Type<'arena>> {
        let arena = self.builder.arena();
        match source.atoms {
            [Atom::Array(array)] => {
                let mut items = Vec::new_in(arena);
                if let Some(known) = array.known_items {
                    for item in known {
                        let value = if narrow { self.truthy(item.value) } else { item.value };
                        if !value.is_never() {
                            items.push(KnownItem { key: item.key, value, optional: true });
                        }
                    }
                }

                let (key_param, value_param) = match (array.key_param, array.value_param) {
                    (Some(key), Some(value)) => {
                        let value = if narrow { self.truthy(value) } else { value };
                        if value.is_never() { (None, None) } else { (Some(key), Some(value)) }
                    }
                    _ => (None, None),
                };

                if items.is_empty() && value_param.is_none() {
                    return Some(self.builder.union_of(&[EMPTY_ARRAY]));
                }

                let known_items = (!items.is_empty()).then(|| self.builder.known_items(&items));
                let atom =
                    self.builder.array(ArrayAtom { key_param, value_param, known_items, flags: U8Flags::empty() });

                Some(self.builder.union_of(&[atom]))
            }
            [Atom::List(list)] => {
                let mut values = Vec::new_in(arena);
                if let Some(known) = list.known_elements {
                    for element in known {
                        values.extend_from_slice(element.value.atoms);
                    }
                }

                if !list.element_type.is_never() {
                    values.extend_from_slice(list.element_type.atoms);
                }

                if values.is_empty() {
                    return Some(self.builder.union_of(&[EMPTY_ARRAY]));
                }

                let value = self.builder.union_of(&values);
                let value = if narrow { self.truthy(value) } else { value };
                if value.is_never() {
                    return Some(self.builder.union_of(&[EMPTY_ARRAY]));
                }

                let key = self.builder.union_of(&[NON_NEGATIVE_INT]);
                let atom = self.builder.keyed_unsealed(key, value, false);

                Some(self.builder.union_of(&[atom]))
            }
            _ => None,
        }
    }

    pub fn reverse_list(&mut self, source: Type<'arena>) -> Option<Type<'arena>> {
        match source.atoms {
            [Atom::List(list)] if list.element_type.is_never() && list.known_elements.is_some() => {
                let elements = list.known_elements.unwrap_or(&[]);
                let arena = self.builder.arena();
                let mut reversed = Vec::with_capacity_in(elements.len(), arena);
                for (index, element) in elements.iter().rev().enumerate() {
                    reversed.push(KnownElement { index: index as u32, value: element.value, optional: element.optional });
                }
                let known_elements = Some(self.builder.known_elements(&reversed));
                let atom = self.builder.list(ListAtom {
                    element_type: TYPE_NEVER,
                    known_elements,
                    known_count: list.known_count,
                    flags: list.flags,
                });

                Some(self.builder.union_of(&[atom]))
            }
            [Atom::List(_)] | [Atom::Array(_)] => Some(source),
            _ => None,
        }
    }

    pub fn remap_array_values(&mut self, source: Type<'arena>, value: Type<'arena>) -> Option<Type<'arena>> {
        let arena = self.builder.arena();
        let atom = match source.atoms {
            [Atom::List(list)] => {
                let known_elements = list.known_elements.map(|elements| {
                    let mut remapped = Vec::with_capacity_in(elements.len(), arena);
                    for element in elements {
                        remapped.push(KnownElement { index: element.index, value, optional: element.optional });
                    }

                    self.builder.known_elements(&remapped)
                });
                let element_type = if list.element_type.is_never() { TYPE_NEVER } else { value };

                self.builder.list(ListAtom {
                    element_type,
                    known_elements,
                    known_count: list.known_count,
                    flags: list.flags,
                })
            }
            [Atom::Array(array)] => {
                let known_items = array.known_items.map(|items| {
                    let mut remapped = Vec::with_capacity_in(items.len(), arena);
                    for item in items {
                        remapped.push(KnownItem { key: item.key, value, optional: item.optional });
                    }

                    self.builder.known_items(&remapped)
                });
                let (key_param, value_param) =
                    if array.is_sealed() { (None, None) } else { (array.key_param, Some(value)) };

                self.builder.array(ArrayAtom { key_param, value_param, known_items, flags: array.flags })
            }
            _ => return None,
        };

        Some(self.builder.union_of(&[atom]))
    }

    /// A literal float type; non-finite values widen to `float`.
    pub fn float(&mut self, value: f64) -> Type<'arena> {
        if !value.is_finite() {
            return TYPE_FLOAT;
        }

        self.builder.union_of(&[Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(value))))])
    }

    /// The output arena. Returns a `&'arena` reference whose lifetime is
    /// independent of `&self`, so scratch [`Vec`]s built on it can coexist with
    /// the `&mut self` builder calls that consume them.
    #[must_use]
    pub fn arena(&self) -> &'arena A {
        self.builder.arena()
    }

    /// The symbol table for the program under inference.
    #[must_use]
    pub fn symbols(&self) -> &SymbolTable<'arena, A> {
        self.symbols
    }

    /// The namespace the current expression is in (empty for the global namespace).
    #[must_use]
    pub fn namespace(&self) -> &[u8] {
        self.namespace
    }
}
