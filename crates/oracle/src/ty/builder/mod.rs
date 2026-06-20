//! Construction and hash-consing of types.
//!
//! [`TypeBuilder`] is the only sanctioned way to create [`Type`]s and
//! reference-payload [`Atom`]s. It deduplicates every payload, name, and atom
//! slice it allocates: building the same value twice returns the same
//! `&'arena` reference, so within one builder structural equality coincides
//! with pointer equality (see [`Type::ptr_eq`]).
//!
//! Deduplication tables live on a scratch arena and die with the builder;
//! the payloads they index live on the output arena and outlive it. This is
//! the same split [`mago_allocator`] documents for compiler pipelines: one
//! builder per worker over a scratch that resets per file, payloads on the
//! arena that the resulting types borrow from.
//!
//! The builder is seeded with the [`well_known`] constants, so well-known
//! shapes (`string`, `int`, `array<array-key, mixed>`, …) resolve to
//! `'static` instances that are pointer-identical across builders and arenas.
//!
//! [`UnionBuffer`](union_buffer::UnionBuffer) layers a mutate-then-finalize
//! workflow on top of this; constructive algebra (join, meet, subtract)
//! builds on both.

use core::hash::Hash;

use mago_allocator::Arena;
use mago_allocator::collections::HashSet;
use mago_allocator::copy::CopyInto;

use crate::path::Path;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::alias::AliasAtom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayKey;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::callable::CallableAlias;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Parameter;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::conditional::ConditionalAtom;
use crate::ty::atom::payload::derived::DerivedAtom;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::intersected::IntersectedAtom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::atom::payload::object::enumeration::EnumAtom;
use crate::ty::atom::payload::object::has_method::HasMethodAtom;
use crate::ty::atom::payload::object::has_property::HasPropertyAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::shape::KnownProperty;
use crate::ty::atom::payload::object::shape::ObjectShapeAtom;
use crate::ty::atom::payload::reference::GlobalReferenceAtom;
use crate::ty::atom::payload::reference::MemberReferenceAtom;
use crate::ty::atom::payload::reference::NameSelector;
use crate::ty::atom::payload::reference::SymbolReferenceAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::int::IntRange;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::well_known;
use crate::var::Var;

pub mod union_buffer;

mod sugar;

/// Hash-consing constructor for [`Type`]s and reference-payload [`Atom`]s.
///
/// See the [module documentation](self) for the consing guarantees and the
/// scratch/arena lifetime split. One builder serves one output arena; it is
/// single-threaded by design - share types across threads, not builders.
#[derive(Debug)]
pub struct TypeBuilder<'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    arena: &'arena A,
    scratch: &'scratch S,
    names: HashSet<'scratch, &'arena [u8], S>,
    int_ranges: HashSet<'scratch, &'arena IntRange, S>,
    strings: HashSet<'scratch, &'arena StringAtom<'arena>, S>,
    class_like_strings: HashSet<'scratch, &'arena ClassLikeStringAtom<'arena>, S>,
    objects: HashSet<'scratch, &'arena ObjectAtom<'arena>, S>,
    enumerations: HashSet<'scratch, &'arena EnumAtom<'arena>, S>,
    object_shapes: HashSet<'scratch, &'arena ObjectShapeAtom<'arena>, S>,
    arrays: HashSet<'scratch, &'arena ArrayAtom<'arena>, S>,
    lists: HashSet<'scratch, &'arena ListAtom<'arena>, S>,
    iterables: HashSet<'scratch, &'arena IterableAtom<'arena>, S>,
    signatures: HashSet<'scratch, &'arena Signature<'arena>, S>,
    callable_aliases: HashSet<'scratch, &'arena CallableAlias<'arena>, S>,
    generic_parameters: HashSet<'scratch, &'arena GenericParameterAtom<'arena>, S>,
    references: HashSet<'scratch, &'arena SymbolReferenceAtom<'arena>, S>,
    member_references: HashSet<'scratch, &'arena MemberReferenceAtom<'arena>, S>,
    global_references: HashSet<'scratch, &'arena GlobalReferenceAtom<'arena>, S>,
    aliases: HashSet<'scratch, &'arena AliasAtom<'arena>, S>,
    conditionals: HashSet<'scratch, &'arena ConditionalAtom<'arena>, S>,
    deriveds: HashSet<'scratch, &'arena DerivedAtom<'arena>, S>,
    negated_types: HashSet<'scratch, &'arena Type<'arena>, S>,
    intersecteds: HashSet<'scratch, &'arena IntersectedAtom<'arena>, S>,
    atom_references: HashSet<'scratch, &'arena Atom<'arena>, S>,
    atom_slices: HashSet<'scratch, &'arena [Atom<'arena>], S>,
    type_slices: HashSet<'scratch, &'arena [Type<'arena>], S>,
    known_item_slices: HashSet<'scratch, &'arena [KnownItem<'arena>], S>,
    known_element_slices: HashSet<'scratch, &'arena [KnownElement<'arena>], S>,
    known_property_slices: HashSet<'scratch, &'arena [KnownProperty<'arena>], S>,
    parameter_slices: HashSet<'scratch, &'arena [Parameter<'arena>], S>,
    sort_buffer: Vec<Atom<'arena>>,
}

fn cons<'arena, T, S, A>(set: &mut HashSet<'_, &'arena T, S>, arena: &'arena A, value: T) -> &'arena T
where
    T: Copy + Eq + Hash,
    S: Arena,
    A: Arena,
{
    match set.get(&value) {
        Some(existing) => existing,
        None => {
            let allocated = &*arena.alloc(value);
            set.insert(allocated);

            allocated
        }
    }
}

fn cons_slice<'arena, T, S, A>(set: &mut HashSet<'_, &'arena [T], S>, arena: &'arena A, values: &[T]) -> &'arena [T]
where
    T: Copy + Eq + Hash,
    S: Arena,
    A: Arena,
{
    match set.get(values) {
        Some(existing) => existing,
        None => {
            let allocated = &*arena.alloc_slice_copy(values);
            set.insert(allocated);

            allocated
        }
    }
}

impl<'scratch, 'arena, S, A> TypeBuilder<'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// Construct a builder over `arena`, with deduplication tables on
    /// `scratch`, seeded with every [`well_known`] constant.
    #[must_use]
    pub fn new(arena: &'arena A, scratch: &'scratch S) -> TypeBuilder<'scratch, 'arena, S, A> {
        let mut builder = TypeBuilder {
            arena,
            scratch,
            names: HashSet::new_in(scratch),
            int_ranges: HashSet::new_in(scratch),
            strings: HashSet::new_in(scratch),
            class_like_strings: HashSet::new_in(scratch),
            objects: HashSet::new_in(scratch),
            enumerations: HashSet::new_in(scratch),
            object_shapes: HashSet::new_in(scratch),
            arrays: HashSet::new_in(scratch),
            lists: HashSet::new_in(scratch),
            iterables: HashSet::new_in(scratch),
            signatures: HashSet::new_in(scratch),
            callable_aliases: HashSet::new_in(scratch),
            generic_parameters: HashSet::new_in(scratch),
            references: HashSet::new_in(scratch),
            member_references: HashSet::new_in(scratch),
            global_references: HashSet::new_in(scratch),
            aliases: HashSet::new_in(scratch),
            conditionals: HashSet::new_in(scratch),
            deriveds: HashSet::new_in(scratch),
            negated_types: HashSet::new_in(scratch),
            intersecteds: HashSet::new_in(scratch),
            atom_references: HashSet::new_in(scratch),
            atom_slices: HashSet::new_in(scratch),
            type_slices: HashSet::new_in(scratch),
            known_item_slices: HashSet::new_in(scratch),
            known_element_slices: HashSet::new_in(scratch),
            known_property_slices: HashSet::new_in(scratch),
            parameter_slices: HashSet::new_in(scratch),
            sort_buffer: Vec::new(),
        };

        builder.seed();
        builder
    }

    /// The output arena this builder allocates into.
    #[inline]
    #[must_use]
    pub(crate) const fn arena(&self) -> &'arena A {
        self.arena
    }

    fn seed(&mut self) {
        self.names.insert(b"");
        for atom in well_known::ATOMS {
            match *atom {
                Atom::Int(IntAtom::Range(range)) => {
                    self.int_ranges.insert(range);
                }
                Atom::String(payload) => {
                    self.strings.insert(payload);
                }
                Atom::ClassLikeString(payload) => {
                    self.class_like_strings.insert(payload);
                }
                Atom::Array(payload) => {
                    self.arrays.insert(payload);
                }
                Atom::Iterable(payload) => {
                    self.iterables.insert(payload);
                }
                _ => {}
            }
        }

        for ty in well_known::types() {
            self.atom_slices.insert(ty.atoms);
        }
    }

    /// The scratch arena backing this builder's deduplication tables. It
    /// resets per file, so it doubles as the temp-allocation arena for the
    /// lattice operations: the returned reference is tied to `'scratch`, not
    /// to `&self`, so a scratch-allocated [`Vec`](mago_allocator::vec::Vec)
    /// coexists with later `&mut self` builder calls.
    #[inline]
    #[must_use]
    pub fn scratch(&self) -> &'scratch S {
        self.scratch
    }

    /// An empty growable buffer allocated on the scratch arena (never the
    /// heap). Use for the lattice operations' temporary atom/type lists; it is
    /// freed wholesale when the scratch arena resets.
    #[inline]
    #[must_use]
    pub fn scratch_vec<T>(&self) -> mago_allocator::vec::Vec<'scratch, T, S> {
        mago_allocator::vec::Vec::new_in(self.scratch)
    }

    /// Like [`scratch_vec`](Self::scratch_vec) but pre-sized to `capacity`, so
    /// a temporary that never grows past its bound makes a single scratch
    /// allocation and never leaks intermediate buffers into the bump arena.
    #[inline]
    #[must_use]
    pub fn scratch_vec_with<T>(&self, capacity: usize) -> mago_allocator::vec::Vec<'scratch, T, S> {
        mago_allocator::vec::Vec::with_capacity_in(capacity, self.scratch)
    }

    /// A scratch-arena buffer pre-filled with a copy of `values`. The
    /// scratch-arena replacement for `slice.to_vec()` and `vec![..]` when the
    /// elements are already in hand.
    #[inline]
    #[must_use]
    pub fn scratch_vec_from_slice<T>(&self, values: &[T]) -> mago_allocator::vec::Vec<'scratch, T, S>
    where
        T: Copy,
    {
        let mut buffer = mago_allocator::vec::Vec::with_capacity_in(values.len(), self.scratch);
        buffer.extend_from_slice(values);
        buffer
    }

    /// Intern a name. Within one builder, equal byte sequences share one
    /// allocation.
    #[must_use]
    pub fn intern(&mut self, bytes: &[u8]) -> &'arena [u8] {
        cons_slice(&mut self.names, self.arena, bytes)
    }

    /// Re-intern a [`Path`] from any arena into this builder, deep-copying its
    /// parts and preserving its precomputed [`SymbolId`](crate::id::SymbolId).
    #[must_use]
    pub fn intern_fqn(&mut self, fqn: Path<'_>) -> Path<'arena> {
        fqn.copy_into(self.arena)
    }

    /// Intern `name` and build a class-like [`Path`] in this builder's arena.
    #[must_use]
    pub fn intern_class_like_path(&mut self, name: &[u8]) -> Path<'arena> {
        let interned = self.intern(name);

        Path::class_like(self.arena, interned)
    }

    /// Intern `name` and build a function-like [`Path`] in this builder's arena.
    #[must_use]
    pub fn intern_function_like_path(&mut self, name: &[u8]) -> Path<'arena> {
        let interned = self.intern(name);

        Path::function_like(self.arena, interned)
    }

    /// Intern a class-like constant `class::name` as a [`Path`] in this builder's arena.
    #[must_use]
    pub fn intern_class_like_constant_path(&mut self, class: &[u8], name: &[u8]) -> Path<'arena> {
        let class = self.intern(class);
        let name = self.intern(name);

        Path::class_like_constant(self.arena, class, name)
    }

    /// Intern an enum case `enum::name` as a [`Path`] in this builder's arena.
    #[must_use]
    pub fn intern_enum_case_path(&mut self, enum_name: &[u8], name: &[u8]) -> Path<'arena> {
        let enum_name = self.intern(enum_name);
        let name = self.intern(name);

        Path::enum_case(self.arena, enum_name, name)
    }

    /// Re-intern a [`Var`] from any arena into this builder.
    #[must_use]
    pub fn intern_var(&mut self, var: Var<'_>) -> Var<'arena> {
        Var::new(self.intern(var.as_bytes()))
    }

    /// `int<lower, upper>` with either bound open when `None`.
    #[must_use]
    pub fn int_range(&mut self, lower: Option<i64>, upper: Option<i64>) -> Atom<'arena> {
        Atom::Int(IntAtom::Range(cons(&mut self.int_ranges, self.arena, IntRange::new(lower, upper))))
    }

    #[must_use]
    pub fn string(&mut self, payload: StringAtom<'arena>) -> Atom<'arena> {
        Atom::String(cons(&mut self.strings, self.arena, payload))
    }

    #[must_use]
    pub fn class_like_string(&mut self, payload: ClassLikeStringAtom<'arena>) -> Atom<'arena> {
        Atom::ClassLikeString(cons(&mut self.class_like_strings, self.arena, payload))
    }

    #[must_use]
    pub fn object(&mut self, payload: ObjectAtom<'arena>) -> Atom<'arena> {
        Atom::Object(cons(&mut self.objects, self.arena, payload))
    }

    #[must_use]
    pub fn enumeration(&mut self, payload: EnumAtom<'arena>) -> Atom<'arena> {
        Atom::Enum(cons(&mut self.enumerations, self.arena, payload))
    }

    #[must_use]
    pub fn object_shape(&mut self, payload: ObjectShapeAtom<'arena>) -> Atom<'arena> {
        Atom::ObjectShape(cons(&mut self.object_shapes, self.arena, payload))
    }

    #[must_use]
    pub fn array(&mut self, payload: ArrayAtom<'arena>) -> Atom<'arena> {
        Atom::Array(cons(&mut self.arrays, self.arena, payload))
    }

    #[must_use]
    pub fn list(&mut self, payload: ListAtom<'arena>) -> Atom<'arena> {
        Atom::List(cons(&mut self.lists, self.arena, payload))
    }

    #[must_use]
    pub fn iterable(&mut self, payload: IterableAtom<'arena>) -> Atom<'arena> {
        Atom::Iterable(cons(&mut self.iterables, self.arena, payload))
    }

    /// Intern a callable signature. The caller wraps the result in
    /// [`CallableAtom::Closure`] or [`CallableAtom::Signature`].
    #[must_use]
    pub fn signature(&mut self, signature: Signature<'arena>) -> &'arena Signature<'arena> {
        cons(&mut self.signatures, self.arena, signature)
    }

    /// Intern a callable alias. The caller wraps the result in
    /// [`CallableAtom::Alias`].
    #[must_use]
    pub fn callable_alias(&mut self, alias: CallableAlias<'arena>) -> &'arena CallableAlias<'arena> {
        cons(&mut self.callable_aliases, self.arena, alias)
    }

    #[must_use]
    pub fn generic_parameter(&mut self, payload: GenericParameterAtom<'arena>) -> Atom<'arena> {
        Atom::GenericParameter(cons(&mut self.generic_parameters, self.arena, payload))
    }

    #[must_use]
    pub fn reference(&mut self, payload: SymbolReferenceAtom<'arena>) -> Atom<'arena> {
        Atom::Reference(cons(&mut self.references, self.arena, payload))
    }

    #[must_use]
    pub fn member_reference(&mut self, payload: MemberReferenceAtom<'arena>) -> Atom<'arena> {
        Atom::MemberReference(cons(&mut self.member_references, self.arena, payload))
    }

    #[must_use]
    pub fn global_reference(&mut self, payload: GlobalReferenceAtom<'arena>) -> Atom<'arena> {
        Atom::GlobalReference(cons(&mut self.global_references, self.arena, payload))
    }

    #[must_use]
    pub fn alias(&mut self, payload: AliasAtom<'arena>) -> Atom<'arena> {
        Atom::Alias(cons(&mut self.aliases, self.arena, payload))
    }

    #[must_use]
    pub fn conditional(&mut self, payload: ConditionalAtom<'arena>) -> Atom<'arena> {
        Atom::Conditional(cons(&mut self.conditionals, self.arena, payload))
    }

    #[must_use]
    pub fn derived(&mut self, payload: DerivedAtom<'arena>) -> Atom<'arena> {
        Atom::Derived(cons(&mut self.deriveds, self.arena, payload))
    }

    /// `!T` (the complement of `T` against `mixed`). Universal collapses
    /// fire here: `!never` is `mixed`, `!mixed` is `never`, and `!!T`
    /// unwraps when both layers carry single-atom types (multi-atom `!!T`
    /// is left as-is because the result would not fit a single atom).
    #[must_use]
    pub fn negated(&mut self, ty: Type<'arena>) -> Atom<'arena> {
        if ty.is_never() {
            return well_known::MIXED;
        }

        if ty == well_known::TYPE_MIXED {
            return well_known::NEVER;
        }

        if let [Atom::Negated(inner)] = ty.atoms
            && let [single] = inner.atoms
        {
            return *single;
        }

        Atom::Negated(cons(&mut self.negated_types, self.arena, ty))
    }

    /// `head & conjunct & …`. Empty conjuncts return `head`; nested
    /// intersected heads are flattened; conjuncts are sorted, deduplicated,
    /// and self-references to `head` dropped. `head & !head` (and direct
    /// `X & !X` pairs) collapse to `never`.
    #[must_use]
    pub fn intersected(&mut self, head: Atom<'arena>, conjuncts: &[Atom<'arena>]) -> Atom<'arena> {
        if conjuncts.is_empty() {
            return head;
        }

        let (real_head, mut all_conjuncts) = match head {
            Atom::Intersected(existing) => {
                let mut accumulated = self.scratch_vec_from_slice(existing.conjuncts);
                accumulated.extend_from_slice(conjuncts);

                (*existing.head, accumulated)
            }
            _ => (head, self.scratch_vec_from_slice(conjuncts)),
        };

        all_conjuncts.retain(|conjunct| *conjunct != real_head);
        all_conjuncts.sort_unstable();
        all_conjuncts.dedup();

        if all_conjuncts.is_empty() {
            return real_head;
        }

        for conjunct in &all_conjuncts {
            let Atom::Negated(inner) = conjunct else {
                continue;
            };
            let [single] = inner.atoms else {
                continue;
            };

            if *single == real_head {
                return well_known::NEVER;
            }

            if all_conjuncts.iter().any(|other| other != conjunct && other == single) {
                return well_known::NEVER;
            }
        }

        let head_reference = cons(&mut self.atom_references, self.arena, real_head);
        let conjunct_slice = cons_slice(&mut self.atom_slices, self.arena, &all_conjuncts);

        Atom::Intersected(cons(
            &mut self.intersecteds,
            self.arena,
            IntersectedAtom { head: head_reference, conjuncts: conjunct_slice },
        ))
    }

    /// Intern a type-argument list. Order is preserved (positional).
    #[must_use]
    pub fn types(&mut self, types: &[Type<'arena>]) -> &'arena [Type<'arena>] {
        cons_slice(&mut self.type_slices, self.arena, types)
    }

    /// Intern a keyed-array known-items list. Order is preserved; callers
    /// keep entries sorted by key.
    #[must_use]
    pub fn known_items(&mut self, items: &[KnownItem<'arena>]) -> &'arena [KnownItem<'arena>] {
        cons_slice(&mut self.known_item_slices, self.arena, items)
    }

    /// Intern a list known-elements list. Order is preserved; callers keep
    /// entries sorted by index.
    #[must_use]
    pub fn known_elements(&mut self, elements: &[KnownElement<'arena>]) -> &'arena [KnownElement<'arena>] {
        cons_slice(&mut self.known_element_slices, self.arena, elements)
    }

    /// Intern an object-shape known-properties list. Order is preserved;
    /// callers keep entries sorted by name.
    #[must_use]
    pub fn known_properties(&mut self, properties: &[KnownProperty<'arena>]) -> &'arena [KnownProperty<'arena>] {
        cons_slice(&mut self.known_property_slices, self.arena, properties)
    }

    /// Intern a signature parameter list. Order is preserved (positional).
    #[must_use]
    pub fn parameters(&mut self, parameters: &[Parameter<'arena>]) -> &'arena [Parameter<'arena>] {
        cons_slice(&mut self.parameter_slices, self.arena, parameters)
    }

    /// Build a union from `atoms`: sort, deduplicate, intern. Empty input
    /// collapses to [`well_known::TYPE_NEVER`]. No merge rules are applied:
    /// `true|false` does not collapse to `bool` - lattice-canonical
    /// collapsing belongs to the join.
    #[must_use]
    pub fn union_of(&mut self, atoms: &[Atom<'arena>]) -> Type<'arena> {
        if atoms.is_empty() {
            return well_known::TYPE_NEVER;
        }

        let union = Type::from_canonical_atoms(self.canonical_atom_slice(atoms));
        union.assert_canonical();

        union
    }

    fn canonical_atom_slice(&mut self, atoms: &[Atom<'arena>]) -> &'arena [Atom<'arena>] {
        if atoms.is_sorted_by(|left, right| left < right) {
            return cons_slice(&mut self.atom_slices, self.arena, atoms);
        }

        self.sort_buffer.clear();
        self.sort_buffer.extend_from_slice(atoms);
        self.sort_buffer.sort_unstable();
        self.sort_buffer.dedup();

        cons_slice(&mut self.atom_slices, self.arena, &self.sort_buffer)
    }

    /// Deep-copy a type from any other arena into this builder, re-consing
    /// every payload, name, and slice. This is the cross-arena promotion path: the
    /// consing equivalent of [`CopyInto`](mago_allocator::copy::CopyInto).
    #[must_use]
    pub fn import(&mut self, ty: Type<'_>) -> Type<'arena> {
        let mut imported = self.scratch_vec_with(ty.atoms.len());
        for atom in ty.atoms {
            imported.push(self.import_atom(*atom));
        }

        self.union_of(&imported)
    }

    /// Deep-copy a single atom from any other arena into this builder.
    #[must_use]
    pub fn import_atom(&mut self, atom: Atom<'_>) -> Atom<'arena> {
        match atom {
            Atom::Null => Atom::Null,
            Atom::Never => Atom::Never,
            Atom::Void => Atom::Void,
            Atom::Placeholder => Atom::Placeholder,
            Atom::Mixed(payload) => Atom::Mixed(payload),
            Atom::Bool => Atom::Bool,
            Atom::True => Atom::True,
            Atom::False => Atom::False,
            Atom::Int(IntAtom::Unspecified) => Atom::Int(IntAtom::Unspecified),
            Atom::Int(IntAtom::UnspecifiedLiteral) => Atom::Int(IntAtom::UnspecifiedLiteral),
            Atom::Int(IntAtom::Literal(value)) => Atom::Int(IntAtom::Literal(value)),
            Atom::Int(IntAtom::Range(range)) => self.int_range(range.lower(), range.upper()),
            Atom::Float(payload) => Atom::Float(payload),
            Atom::String(payload) => {
                let literal = match payload.literal {
                    StringLiteral::None => StringLiteral::None,
                    StringLiteral::Unspecified => StringLiteral::Unspecified,
                    StringLiteral::Value(value) => StringLiteral::Value(self.intern(value)),
                };

                self.string(StringAtom { literal, casing: payload.casing, flags: payload.flags })
            }
            Atom::ClassLikeString(payload) => {
                let specifier = match payload.specifier {
                    ClassLikeStringSpecifier::Any => ClassLikeStringSpecifier::Any,
                    ClassLikeStringSpecifier::Literal { value } => {
                        ClassLikeStringSpecifier::Literal { value: self.intern_fqn(value) }
                    }
                    ClassLikeStringSpecifier::OfType { constraint } => {
                        ClassLikeStringSpecifier::OfType { constraint: self.import(constraint) }
                    }
                    ClassLikeStringSpecifier::Generic { constraint } => {
                        ClassLikeStringSpecifier::Generic { constraint: self.import(constraint) }
                    }
                };

                self.class_like_string(ClassLikeStringAtom { kind: payload.kind, specifier })
            }
            Atom::Scalar => Atom::Scalar,
            Atom::Numeric => Atom::Numeric,
            Atom::ArrayKey => Atom::ArrayKey,
            Atom::Object(payload) => {
                let name = self.intern_fqn(payload.name);
                let type_arguments = payload.type_arguments.map(|type_arguments| self.import_types(type_arguments));

                self.object(ObjectAtom { name, type_arguments, flags: payload.flags })
            }
            Atom::Enum(payload) => {
                let name = self.intern_fqn(payload.name);
                let case = payload.case.map(|case| self.intern(case));

                self.enumeration(EnumAtom { name, case })
            }
            Atom::ObjectShape(payload) => {
                let known_properties = match payload.known_properties {
                    Some(entries) => {
                        let mut imported = self.scratch_vec_with(entries.len());
                        for entry in entries {
                            imported.push(KnownProperty {
                                name: self.intern(entry.name),
                                value: self.import(entry.value),
                                optional: entry.optional,
                            });
                        }

                        Some(self.known_properties(&imported))
                    }
                    None => None,
                };

                self.object_shape(ObjectShapeAtom { known_properties, flags: payload.flags })
            }
            Atom::HasMethod(payload) => {
                Atom::HasMethod(HasMethodAtom { method_name: self.intern(payload.method_name) })
            }
            Atom::HasProperty(payload) => {
                Atom::HasProperty(HasPropertyAtom { property_name: self.intern(payload.property_name) })
            }
            Atom::Array(payload) => {
                let key_param = payload.key_param.map(|key_param| self.import(key_param));
                let value_param = payload.value_param.map(|value_param| self.import(value_param));
                let known_items = match payload.known_items {
                    Some(entries) => {
                        let mut imported = self.scratch_vec_with(entries.len());
                        for entry in entries {
                            imported.push(KnownItem {
                                key: self.import_array_key(entry.key),
                                value: self.import(entry.value),
                                optional: entry.optional,
                            });
                        }

                        Some(self.known_items(&imported))
                    }
                    None => None,
                };

                self.array(ArrayAtom { key_param, value_param, known_items, flags: payload.flags })
            }
            Atom::List(payload) => {
                let element_type = self.import(payload.element_type);
                let known_elements = match payload.known_elements {
                    Some(entries) => {
                        let mut imported = self.scratch_vec_with(entries.len());
                        for entry in entries {
                            imported.push(KnownElement {
                                index: entry.index,
                                value: self.import(entry.value),
                                optional: entry.optional,
                            });
                        }

                        Some(self.known_elements(&imported))
                    }
                    None => None,
                };

                self.list(ListAtom {
                    element_type,
                    known_elements,
                    known_count: payload.known_count,
                    flags: payload.flags,
                })
            }
            Atom::Iterable(payload) => {
                let key_type = self.import(payload.key_type);
                let value_type = self.import(payload.value_type);

                self.iterable(IterableAtom { key_type, value_type })
            }
            Atom::Callable(payload) => match payload {
                CallableAtom::Any => Atom::Callable(CallableAtom::Any),
                CallableAtom::Closure(signature) => {
                    Atom::Callable(CallableAtom::Closure(self.import_signature(signature)))
                }
                CallableAtom::Signature(signature) => {
                    Atom::Callable(CallableAtom::Signature(self.import_signature(signature)))
                }
                CallableAtom::Alias(alias) => Atom::Callable(CallableAtom::Alias(self.import_callable_alias(alias))),
            },
            Atom::Resource(payload) => Atom::Resource(payload),
            Atom::GenericParameter(payload) => {
                let name = self.intern(payload.name);
                let defining_entity = match payload.defining_entity {
                    DefiningEntity::ClassLike(class_like) => DefiningEntity::ClassLike(self.intern_fqn(class_like)),
                    DefiningEntity::Method { class, method } => {
                        DefiningEntity::Method { class: self.intern_fqn(class), method: self.intern(method) }
                    }
                    DefiningEntity::Function(function) => DefiningEntity::Function(self.intern_fqn(function)),
                };
                let constraint = self.import(payload.constraint);

                self.generic_parameter(GenericParameterAtom { name, defining_entity, constraint })
            }
            Atom::Variable(payload) => {
                Atom::Variable(crate::ty::atom::payload::variable::VariableAtom { name: self.intern_var(payload.name) })
            }
            Atom::Reference(payload) => {
                let name = self.intern_fqn(payload.name);
                let type_arguments = payload.type_arguments.map(|type_arguments| self.import_types(type_arguments));

                self.reference(SymbolReferenceAtom { name, type_arguments })
            }
            Atom::MemberReference(payload) => {
                let class_like_name = self.intern_fqn(payload.class_like_name);
                let selector = self.import_name_selector(payload.selector);

                self.member_reference(MemberReferenceAtom { class_like_name, selector })
            }
            Atom::GlobalReference(payload) => {
                let selector = self.import_name_selector(payload.selector);

                self.global_reference(GlobalReferenceAtom { selector })
            }
            Atom::Alias(payload) => {
                let class_name = self.intern_fqn(payload.class_name);
                let alias_name = self.intern(payload.alias_name);

                self.alias(AliasAtom { class_name, alias_name })
            }
            Atom::Conditional(payload) => {
                let subject = self.import(payload.subject);
                let target = self.import(payload.target);
                let then = self.import(payload.then);
                let otherwise = self.import(payload.otherwise);

                self.conditional(ConditionalAtom { subject, target, then, otherwise, negated: payload.negated })
            }
            Atom::Derived(payload) => {
                let imported = match *payload {
                    DerivedAtom::KeyOf(target) => DerivedAtom::KeyOf(self.import(target)),
                    DerivedAtom::ValueOf(target) => DerivedAtom::ValueOf(self.import(target)),
                    DerivedAtom::PropertiesOf { target, visibility } => {
                        DerivedAtom::PropertiesOf { target: self.import(target), visibility }
                    }
                    DerivedAtom::IndexAccess { target, index } => {
                        DerivedAtom::IndexAccess { target: self.import(target), index: self.import(index) }
                    }
                    DerivedAtom::IntMask(members) => DerivedAtom::IntMask(self.import_types(members)),
                    DerivedAtom::IntMaskOf(target) => DerivedAtom::IntMaskOf(self.import(target)),
                    DerivedAtom::TemplateType { object, class_name, template_name } => DerivedAtom::TemplateType {
                        object: self.import(object),
                        class_name: self.import(class_name),
                        template_name: self.import(template_name),
                    },
                    DerivedAtom::New(target) => DerivedAtom::New(self.import(target)),
                };

                self.derived(imported)
            }
            Atom::ObjectAny => Atom::ObjectAny,
            Atom::Negated(inner) => {
                let imported = self.import(*inner);

                self.negated(imported)
            }
            Atom::Intersected(payload) => {
                let head = self.import_atom(*payload.head);
                let mut conjuncts = self.scratch_vec_with(payload.conjuncts.len());
                for conjunct in payload.conjuncts {
                    conjuncts.push(self.import_atom(*conjunct));
                }

                self.intersected(head, &conjuncts)
            }
        }
    }

    fn import_types(&mut self, types: &[Type<'_>]) -> &'arena [Type<'arena>] {
        let mut imported = self.scratch_vec_with(types.len());
        for ty in types {
            imported.push(self.import(*ty));
        }

        self.types(&imported)
    }

    fn import_array_key(&mut self, key: ArrayKey<'_>) -> ArrayKey<'arena> {
        match key {
            ArrayKey::Int(value) => ArrayKey::Int(value),
            ArrayKey::String(name) => ArrayKey::String(self.intern(name)),
            ArrayKey::Const { class, name } => {
                ArrayKey::Const { class: self.intern_fqn(class), name: self.intern(name) }
            }
        }
    }

    fn import_signature(&mut self, signature: &Signature<'_>) -> &'arena Signature<'arena> {
        let parameters = match signature.parameters {
            Some(parameters) => {
                let mut imported = self.scratch_vec_with(parameters.len());
                for parameter in parameters {
                    imported.push(Parameter {
                        name: self.intern(parameter.name),
                        r#type: self.import(parameter.r#type),
                        flags: parameter.flags,
                    });
                }

                Some(self.parameters(&imported))
            }
            None => None,
        };
        let return_type = self.import(signature.return_type);
        let throws = signature.throws.map(|throws| self.import(throws));

        self.signature(Signature { parameters, return_type, throws, flags: signature.flags })
    }

    fn import_callable_alias(&mut self, alias: &CallableAlias<'_>) -> &'arena CallableAlias<'arena> {
        let imported = match *alias {
            CallableAlias::Function(function) => CallableAlias::Function(self.intern_fqn(function)),
            CallableAlias::Method { class, method } => {
                CallableAlias::Method { class: self.intern_fqn(class), method: self.intern(method) }
            }
            CallableAlias::Closure(span) => CallableAlias::Closure(span),
        };

        self.callable_alias(imported)
    }

    fn import_name_selector(&mut self, selector: NameSelector<'_>) -> NameSelector<'arena> {
        match selector {
            NameSelector::Identifier(name) => NameSelector::Identifier(self.intern(name)),
            NameSelector::StartsWith(name) => NameSelector::StartsWith(self.intern(name)),
            NameSelector::EndsWith(name) => NameSelector::EndsWith(self.intern(name)),
            NameSelector::Contains(name) => NameSelector::Contains(self.intern(name)),
            NameSelector::Wildcard => NameSelector::Wildcard,
        }
    }
}
