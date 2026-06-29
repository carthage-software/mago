use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::vec::Vec;
use mago_database::file::File;
use mago_flags::U8Flags;
use mago_hir::ir::IR;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownElement;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::float::LiteralFloat;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;
use mago_oracle::ty::well_known::EMPTY_ARRAY;
use mago_oracle::ty::well_known::TYPE_FLOAT;
use mago_oracle::var::Var;
use ordered_float::OrderedFloat;

use crate::flow::Flow;

mod condition;
mod expression;
mod statement;

/// The local-variable type environment threaded through inference.
pub(crate) type Environment<'source, 'arena, A> = HashMap<'source, Var<'arena>, Type<'arena>, A>;

#[derive(Debug)]
pub struct InferenceFolder<'source, 'symbol, 'arena, A, S, E>
where
    A: Arena,
{
    source: &'source A,
    arena: &'arena A,
    symbols: &'symbol SymbolTable<'arena, A>,
    ty: TypeBuilder<'source, 'arena, A, A>,
    environment: Environment<'source, 'arena, A>,
    line_starts: &'source [u32],
    namespace: &'source [u8],
    reachable: bool,
    _phantom: std::marker::PhantomData<(S, E)>,
}

impl<'source, 'symbol, 'arena, A, S, E> InferenceFolder<'source, 'symbol, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn new(source: &'source A, arena: &'arena A, symbols: &'symbol SymbolTable<'arena, A>, file: &File) -> Self {
        let ty = TypeBuilder::new(arena, source);
        let line_starts = source.alloc_slice_copy(file.lines.as_slice());

        Self {
            source,
            arena,
            symbols,
            ty,
            environment: HashMap::new_in(source),
            line_starts,
            namespace: b"",
            reachable: true,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn infer_ir(&mut self, ir: IR<'source, SymbolId, S, E>) -> IR<'arena, SymbolId, Flow, Type<'arena>> {
        let (statements, _exit) = self.infer_block(ir.statements);

        IR { span: ir.span, statements, errors: self.arena.alloc_slice_copy(ir.errors) }
    }

    pub(crate) fn int_literal(&mut self, value: i64) -> Type<'arena> {
        self.ty.union_of(&[Atom::Int(IntAtom::Literal(value))])
    }

    pub(crate) fn float_literal(&mut self, value: f64) -> Type<'arena> {
        if !value.is_finite() {
            return TYPE_FLOAT;
        }

        self.ty.union_of(&[Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(value))))])
    }

    pub(crate) fn literal_string(&mut self, bytes: &[u8]) -> Type<'arena> {
        let literal = StringLiteral::Value(self.ty.intern(bytes));

        let mut flags = U8Flags::empty();
        if !bytes.is_empty() {
            flags = flags.with(StringRefinementFlag::NonEmpty);
            if bytes != b"0" {
                flags = flags.with(StringRefinementFlag::Truthy);
            }
        }

        let atom = self.ty.string(StringAtom { literal, casing: StringCasing::Unspecified, flags });

        self.ty.union_of(&[atom])
    }

    /// The union of two types.
    pub(crate) fn union(&mut self, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        let mut atoms = Vec::new_in(self.source);
        atoms.extend_from_slice(left.atoms);
        atoms.extend_from_slice(right.atoms);

        self.ty.union_of(&atoms)
    }

    /// Merges a conditionally-taken path back into the environment: keeps only
    /// the variables that existed in `before` (so a var introduced only on the
    /// conditional path, or by scoped narrowing, does not leak as definite), and
    /// unions each with its post-path value. Used after a short-circuit operand
    /// and at branch joins.
    pub(crate) fn merge_environment_with(&mut self, before: Environment<'source, 'arena, A>) {
        let mut merged = HashMap::new_in(self.source);
        for (variable, before_type) in &before {
            let value = match self.environment.get(variable).copied() {
                Some(current) => self.union(*before_type, current),
                None => *before_type,
            };

            merged.insert(*variable, value);
        }

        self.environment = merged;
    }

    /// Unions two environments over the union of their keys (a variable present
    /// in only one keeps its type). Used to join the two truth-paths of a
    /// short-circuit operand inside condition analysis.
    pub(crate) fn union_environments(
        &mut self,
        left: Environment<'source, 'arena, A>,
        right: Environment<'source, 'arena, A>,
    ) -> Environment<'source, 'arena, A> {
        let mut merged = left;
        for (variable, right_type) in &right {
            let value = match merged.get(variable).copied() {
                Some(left_type) => self.union(left_type, *right_type),
                None => *right_type,
            };

            merged.insert(*variable, value);
        }

        merged
    }

    /// Builds a sealed array type from fully-known entries: an ordered run of
    /// `0, 1, 2, ...` integer keys becomes a `list{...}`, anything else a keyed
    /// `array{...}`, and no entries the empty array.
    pub(crate) fn closed_array(&mut self, items: &[KnownItem<'arena>]) -> Type<'arena> {
        if items.is_empty() {
            return self.ty.union_of(&[EMPTY_ARRAY]);
        }

        let non_empty = items.iter().any(|item| !item.optional);
        let is_list = items.iter().enumerate().all(|(index, item)| item.key == ArrayKey::Int(index as i64));

        let atom = if is_list {
            let mut elements = Vec::new_in(self.source);
            for (index, item) in items.iter().enumerate() {
                elements.push(KnownElement { index: index as u32, value: item.value, optional: item.optional });
            }

            self.ty.sealed_list(&elements, non_empty)
        } else {
            self.ty.keyed_sealed(items, non_empty)
        };

        self.ty.union_of(&[atom])
    }
}
