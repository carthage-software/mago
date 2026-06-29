use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_database::file::File;
use mago_flags::U8Flags;
use mago_hir::ir::IR;
use mago_hir::ir::expression::Expression;
use mago_oracle::assertion::Assertion;
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

use crate::error::InferenceResult;
use crate::extension::AssertionSink;
use crate::extension::AssertionTiming;
use crate::extension::ExtensionContext;
use crate::extension::Extensions;
use crate::flow::Flow;
use crate::reconciler::reconcile;

mod argument;
mod condition;
mod environment;
mod expression;
mod item;
mod statement;

pub(crate) use environment::Environment;

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
    extensions: Extensions<'arena, A>,
    _phantom: std::marker::PhantomData<(S, E)>,
}

impl<'source, 'symbol, 'arena, A, S, E> InferenceFolder<'source, 'symbol, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn new(
        source: &'source A,
        arena: &'arena A,
        symbols: &'symbol SymbolTable<'arena, A>,
        file: &File,
        extensions: Extensions<'arena, A>,
    ) -> Self {
        let ty = TypeBuilder::new(arena, source);
        let line_starts = source.alloc_slice_copy(file.lines.as_slice());

        Self {
            source,
            arena,
            symbols,
            ty,
            environment: Environment::new_in(source),
            line_starts,
            namespace: b"",
            reachable: true,
            extensions,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Lets the enabled [`ExtensionInference`] extensions override the type of an
    /// already-inferred expression, and applies any unconditional
    /// ([`AssertionTiming::Always`]) assertions the [`ExtensionAssertion`]
    /// extensions extract from it to the environment. Skipped entirely when no
    /// extensions are enabled.
    pub(crate) fn apply_extensions(&mut self, typed: &mut Expression<'arena, SymbolId, Flow, Type<'arena>>) {
        let inference = self.extensions.inference;
        if !inference.is_empty() {
            let mut context = ExtensionContext::new(&mut self.ty, self.symbols, self.namespace);
            for extension in inference {
                if let Some(ty) = extension.infer(&mut context, typed) {
                    typed.meta = ty;
                    break;
                }
            }
        }

        if !self.extensions.assertion.is_empty() {
            let assertions = self.extension_assertions(typed);
            for (variable, assertion, timing) in &assertions {
                if matches!(timing, AssertionTiming::Always) {
                    let base = self.environment.get(*variable);
                    let narrowed = reconcile(&mut self.ty, self.symbols, *assertion, base);
                    self.environment.set(*variable, narrowed);
                }
            }
        }
    }

    /// Collects the assertions every enabled [`ExtensionAssertion`] extracts from
    /// `expression`, each tagged with the timing under which it holds.
    pub(crate) fn extension_assertions(
        &mut self,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Vec<'source, (Var<'arena>, Assertion<'arena>, AssertionTiming), A> {
        let extensions = self.extensions.assertion;
        let mut entries = Vec::new_in(self.source);
        if !extensions.is_empty() {
            let mut context = ExtensionContext::new(&mut self.ty, self.symbols, self.namespace);
            let mut sink = AssertionSink::new(&mut entries);
            for extension in extensions {
                extension.assertions(&mut context, expression, &mut sink);
            }
        }

        entries
    }

    pub fn infer_ir(
        &mut self,
        ir: IR<'source, SymbolId, S, E>,
    ) -> InferenceResult<IR<'arena, SymbolId, Flow, Type<'arena>>> {
        let (statements, _exit) = self.infer_block(ir.statements)?;

        Ok(IR { span: ir.span, statements, errors: self.arena.alloc_slice_copy(ir.errors) })
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
        environment::union_types(&mut self.ty, left, right)
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
