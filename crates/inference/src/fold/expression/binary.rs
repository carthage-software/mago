#![allow(clippy::float_arithmetic)]

use std::cmp::Ordering;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Binary;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::operator::BinaryOperator;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::overlaps;
use mago_oracle::ty::well_known::TYPE_ARRAY_KEY;
use mago_oracle::ty::well_known::TYPE_BOOL;
use mago_oracle::ty::well_known::TYPE_FALSE;
use mago_oracle::ty::well_known::TYPE_INT;
use mago_oracle::ty::well_known::TYPE_INT_OR_FLOAT;
use mago_oracle::ty::well_known::TYPE_MINUS_ONE_ZERO_ONE;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_oracle::ty::well_known::TYPE_STRING;
use mago_oracle::ty::well_known::TYPE_TRUE;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::reconciler::condition_diagram;
use crate::reconciler::narrowing_assertions;
use crate::reconciler::reconcile;
use crate::semantics::Number;
use crate::semantics::append_string;
use crate::semantics::collect_array_values;
use crate::semantics::collect_closed_array;
use crate::semantics::could_be_array;
use crate::semantics::fold_identical;
use crate::semantics::is_array_type;
use crate::semantics::literal_string_bytes;
use crate::semantics::number_of;
use crate::semantics::numbers_of;
use crate::semantics::parse_php_number;
use crate::semantics::truthiness;
use crate::tdd::DecisionDiagram;

/// A fully-known operand for comparison, with numeric strings already coerced.
#[derive(Clone, Copy)]
enum Comparable<'arena> {
    Number(Number),
    String(&'arena [u8]),
    Bool(bool),
    Null,
}

impl<'arena, 'source, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_binary(
        &mut self,
        span: Span,
        binary: &'source Binary<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let left = self.infer_expression(binary.left)?;

        let right = match binary.operator {
            BinaryOperator::And => self.infer_narrowed(binary.right, &left, true)?,
            BinaryOperator::Or => self.infer_narrowed(binary.right, &left, false)?,
            _ => self.infer_expression(binary.right)?,
        };

        let meta = match binary.operator {
            BinaryOperator::And | BinaryOperator::Or => self
                .boolean_fold(binary.operator, &left, &right)
                .unwrap_or_else(|| self.binary_type(binary.operator, left.meta, right.meta)),
            _ => self.binary_type(binary.operator, left.meta, right.meta),
        };

        let binary = Binary {
            span: binary.span,
            left: self.arena.alloc(left),
            operator: binary.operator,
            right: self.arena.alloc(right),
        };

        Ok(Expression { meta, span, kind: ExpressionKind::Binary(self.arena.alloc(binary)) })
    }

    /// Infers `right` after narrowing the environment by what `left` being
    /// `polarity` (true for `&&`, false for `||`) tells us about its variables.
    /// The narrowing is scoped: each touched variable is restored afterwards.
    fn infer_narrowed(
        &mut self,
        right: &'source Expression<'source, SymbolId, S, E>,
        left: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        polarity: bool,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut assertions = Vec::new_in(self.source);
        narrowing_assertions(left, polarity, &mut assertions);

        let before = self.environment.clone();

        for (variable, assertion) in &assertions {
            let base = self.environment.get(*variable);
            let narrowed = reconcile(&mut self.ty, self.symbols, *assertion, base);

            self.environment.set(*variable, narrowed);
        }

        let right = self.infer_expression(right)?;

        self.environment.merge_with(before, &mut self.ty);

        Ok(right)
    }

    /// Folds `&&`/`||` whose boolean structure is decided regardless of runtime
    /// values: a decision diagram over the operands collapses contradictions to
    /// `false` and tautologies to `true`, catching multi-variable cases the
    /// per-variable narrowing cannot (e.g. `($a || $b) && (!$a && !$b)`).
    fn boolean_fold(
        &self,
        operator: BinaryOperator,
        left: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        right: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<Type<'arena>> {
        let mut diagram = DecisionDiagram::new_in(self.source);
        let left_node = condition_diagram(&mut diagram, left)?;
        let right_node = condition_diagram(&mut diagram, right)?;

        let combined = match operator {
            BinaryOperator::And => diagram.and(left_node, right_node),
            _ => diagram.or(left_node, right_node),
        };

        if combined.is_false() {
            Some(TYPE_FALSE)
        } else if combined.is_true() {
            Some(TYPE_TRUE)
        } else {
            None
        }
    }

    fn binary_type(&mut self, operator: BinaryOperator, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        if left.is_never() {
            return TYPE_NEVER;
        }
        if right.is_never()
            && !matches!(operator, BinaryOperator::And | BinaryOperator::Or | BinaryOperator::NullCoalesce)
        {
            return TYPE_NEVER;
        }

        match operator {
            BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Modulo
            | BinaryOperator::Exponentiation => self.arithmetic(operator, left, right),
            BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOr | BinaryOperator::BitwiseXor => {
                self.bitwise_logical(operator, left, right)
            }
            BinaryOperator::LeftShift | BinaryOperator::RightShift => self.shift(operator, left, right),
            BinaryOperator::StringConcat => self.concat(left, right),
            BinaryOperator::Identical | BinaryOperator::NotIdentical => self.identical(operator, left, right),
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => comparison(operator, left, right),
            BinaryOperator::Spaceship => self.spaceship(left, right),
            BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => logical(operator, left, right),
            BinaryOperator::NullCoalesce => self.null_coalesce(left, right),
            BinaryOperator::Instanceof => TYPE_BOOL,
            BinaryOperator::Pipe => self.resolve_callable_call(right, &[left]),
        }
    }

    /// `===` / `!==`: fold two concrete literals exactly, and otherwise rule it
    /// `false`/`true` whenever the operand types are disjoint (a value can never
    /// be identical to one of a different type), e.g. `string === null`.
    fn identical(&mut self, operator: BinaryOperator, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        let identical = match fold_identical(left, right) {
            Some(value) => Some(value),
            None if !self.types_overlap(left, right) => Some(false),
            None => None,
        };

        let result = match operator {
            BinaryOperator::NotIdentical => identical.map(|value| !value),
            _ => identical,
        };

        match result {
            Some(true) => TYPE_TRUE,
            Some(false) => TYPE_FALSE,
            None => TYPE_BOOL,
        }
    }

    fn types_overlap(&mut self, left: Type<'arena>, right: Type<'arena>) -> bool {
        let mut report = LatticeReport::new();

        overlaps(left, right, self.symbols, LatticeOptions::default(), &mut report, &mut self.ty)
    }

    fn arithmetic(&mut self, operator: BinaryOperator, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        if is_array_type(left) || is_array_type(right) {
            if !matches!(operator, BinaryOperator::Addition) {
                return TYPE_NEVER;
            }
            if !could_be_array(left) || !could_be_array(right) {
                return TYPE_NEVER;
            }

            return self.add_arrays(left, right);
        }

        if is_non_numeric_literal_string(left) || is_non_numeric_literal_string(right) {
            return TYPE_NEVER;
        }

        let mut left_numbers = Vec::new_in(self.source);
        let mut right_numbers = Vec::new_in(self.source);
        if numbers_of(left, &mut left_numbers)
            && numbers_of(right, &mut right_numbers)
            && left_numbers.len().saturating_mul(right_numbers.len()) <= ARITHMETIC_DISTRIBUTION_LIMIT
        {
            let mut results = Vec::new_in(self.source);
            for &left in &left_numbers {
                for &right in &right_numbers {
                    let result = self.arithmetic_pair(operator, left, right);
                    results.extend_from_slice(result.atoms);
                }
            }

            if results.is_empty() {
                return TYPE_NEVER;
            }

            return self.ty.union_of(&results);
        }

        match operator {
            BinaryOperator::Modulo => TYPE_INT,
            _ => TYPE_INT_OR_FLOAT,
        }
    }

    /// Folds a single pair of fully-known numbers, exactly as PHP would, yielding
    /// the literal result (or `never` for the operations PHP throws on, such as
    /// division or modulo by zero).
    fn arithmetic_pair(&mut self, operator: BinaryOperator, left: Number, right: Number) -> Type<'arena> {
        match operator {
            BinaryOperator::Modulo => {
                if right.is_zero() {
                    TYPE_NEVER
                } else {
                    self.int_literal(left.to_int().checked_rem(right.to_int()).unwrap_or(0))
                }
            }
            BinaryOperator::Division => {
                if right.is_zero() {
                    return TYPE_NEVER;
                }

                match (left, right) {
                    (Number::Int(left), Number::Int(right)) if left.checked_rem(right) == Some(0) => {
                        match left.checked_div(right) {
                            Some(value) => self.int_literal(value),
                            None => self.float_literal(left as f64 / right as f64),
                        }
                    }
                    _ => self.float_literal(left.as_f64() / right.as_f64()),
                }
            }
            BinaryOperator::Exponentiation => match (left, right) {
                (Number::Int(base), Number::Int(exponent)) if exponent >= 0 => match u32::try_from(exponent) {
                    Ok(exponent) => match base.checked_pow(exponent) {
                        Some(value) => self.int_literal(value),
                        None => self.float_literal((base as f64).powf(exponent as f64)),
                    },
                    Err(_) => self.float_literal((base as f64).powf(exponent as f64)),
                },
                _ => self.float_literal(left.as_f64().powf(right.as_f64())),
            },
            _ => match (left, right) {
                (Number::Int(left), Number::Int(right)) => {
                    let folded = match operator {
                        BinaryOperator::Addition => left.checked_add(right),
                        BinaryOperator::Subtraction => left.checked_sub(right),
                        _ => left.checked_mul(right),
                    };

                    match folded {
                        Some(value) => self.int_literal(value),
                        None => self.float_literal(match operator {
                            BinaryOperator::Addition => left as f64 + right as f64,
                            BinaryOperator::Subtraction => left as f64 - right as f64,
                            _ => left as f64 * right as f64,
                        }),
                    }
                }
                _ => {
                    let value = match operator {
                        BinaryOperator::Addition => left.as_f64() + right.as_f64(),
                        BinaryOperator::Subtraction => left.as_f64() - right.as_f64(),
                        _ => left.as_f64() * right.as_f64(),
                    };

                    self.float_literal(value)
                }
            },
        }
    }

    fn bitwise_logical(&mut self, operator: BinaryOperator, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        if let (Some(left), Some(right)) = (literal_string_bytes(left), literal_string_bytes(right)) {
            return self.string_bitwise(operator, left, right);
        }

        let (Some(left), Some(right)) = (number_of(left), number_of(right)) else {
            return TYPE_INT;
        };

        let (left, right) = (left.to_int(), right.to_int());

        self.int_literal(match operator {
            BinaryOperator::BitwiseAnd => left & right,
            BinaryOperator::BitwiseOr => left | right,
            BinaryOperator::BitwiseXor => left ^ right,
            _ => unreachable!(),
        })
    }

    fn shift(&mut self, operator: BinaryOperator, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        let (Some(left), Some(right)) = (number_of(left), number_of(right)) else {
            return TYPE_INT;
        };

        let (left, right) = (left.to_int(), right.to_int());
        let Ok(shift) = u32::try_from(right) else {
            return TYPE_INT;
        };

        match operator {
            BinaryOperator::LeftShift => left.checked_shl(shift).map_or(TYPE_INT, |value| self.int_literal(value)),
            BinaryOperator::RightShift => left.checked_shr(shift).map_or(TYPE_INT, |value| self.int_literal(value)),
            _ => unreachable!(),
        }
    }

    fn string_bitwise(&mut self, operator: BinaryOperator, left: &[u8], right: &[u8]) -> Type<'arena> {
        let mut bytes = Vec::new_in(self.source);
        match operator {
            BinaryOperator::BitwiseOr => {
                let (longer, shorter) = if left.len() >= right.len() { (left, right) } else { (right, left) };
                for (index, byte) in longer.iter().enumerate() {
                    bytes.push(if index < shorter.len() { byte | shorter[index] } else { *byte });
                }
            }
            _ => {
                let length = left.len().min(right.len());
                for index in 0..length {
                    bytes.push(match operator {
                        BinaryOperator::BitwiseAnd => left[index] & right[index],
                        _ => left[index] ^ right[index],
                    });
                }
            }
        }

        self.literal_string(&bytes)
    }

    fn concat(&mut self, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        let mut bytes = Vec::new_in(self.source);

        if !append_string(&mut bytes, left) || !append_string(&mut bytes, right) {
            return TYPE_STRING;
        }

        self.literal_string(&bytes)
    }

    fn spaceship(&mut self, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        if let (Some(left), Some(right)) = (comparable_of(left), comparable_of(right))
            && let Some(ordering) = loose_compare(left, right)
        {
            return self.int_literal(match ordering {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            });
        }

        TYPE_MINUS_ONE_ZERO_ONE
    }

    fn null_coalesce(&mut self, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        if !left.atoms.iter().any(|atom| matches!(atom, Atom::Null)) {
            return left;
        }

        let mut atoms = Vec::new_in(self.source);
        for atom in left.atoms {
            if !matches!(atom, Atom::Null) {
                atoms.push(*atom);
            }
        }
        atoms.extend_from_slice(right.atoms);

        self.ty.union_of(&atoms)
    }

    /// PHP `+` on arrays: the result keeps every entry of `left`, plus the
    /// entries of `right` whose keys are not already present (left wins on
    /// collisions). When both operands are sealed shapes the merge is exact;
    /// otherwise the element types are unioned into an unsealed array.
    fn add_arrays(&mut self, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        let mut left_items = Vec::new_in(self.source);
        let mut right_items = Vec::new_in(self.source);
        if collect_closed_array(left, &mut left_items) && collect_closed_array(right, &mut right_items) {
            let mut merged = Vec::new_in(self.source);
            merged.extend_from_slice(&left_items);
            for item in &right_items {
                if !merged.iter().any(|existing| existing.key == item.key) {
                    merged.push(*item);
                }
            }

            return self.closed_array(&merged);
        }

        self.open_array_union(left, right)
    }

    fn open_array_union(&mut self, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
        let mut values = Vec::new_in(self.source);
        let left_non_empty = collect_array_values(left, &mut values);
        let right_non_empty = collect_array_values(right, &mut values);

        let value_type = if values.is_empty() { TYPE_MIXED } else { self.ty.union_of(&values) };
        let atom = self.ty.keyed_unsealed(TYPE_ARRAY_KEY, value_type, left_non_empty || right_non_empty);

        self.ty.union_of(&[atom])
    }
}

/// The largest cartesian product of operand literals that arithmetic enumerates
/// exactly; beyond it the result widens to the sound `int|float` superset rather
/// than building an unbounded union.
const ARITHMETIC_DISTRIBUTION_LIMIT: usize = 256;

/// A statically-known non-numeric literal string, which PHP rejects with a
/// `TypeError` in every arithmetic operator.
fn is_non_numeric_literal_string(ty: Type<'_>) -> bool {
    let [Atom::String(string)] = ty.atoms else {
        return false;
    };

    matches!(string.literal, StringLiteral::Value(value) if parse_php_number(value).is_none())
}

fn comparable_of(ty: Type<'_>) -> Option<Comparable<'_>> {
    let [atom] = ty.atoms else {
        return None;
    };

    Some(match atom {
        Atom::Int(IntAtom::Literal(value)) => Comparable::Number(Number::Int(*value)),
        Atom::Float(FloatAtom::Literal(value)) => Comparable::Number(Number::Float(value.0.into_inner())),
        Atom::True => Comparable::Bool(true),
        Atom::False => Comparable::Bool(false),
        Atom::Null => Comparable::Null,
        Atom::String(string) => match string.literal {
            StringLiteral::Value(value) => match parse_php_number(value) {
                Some(number) => Comparable::Number(number),
                None => Comparable::String(value),
            },
            _ => return None,
        },
        _ => return None,
    })
}

fn comparison<'arena>(operator: BinaryOperator, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
    let folded = match operator {
        BinaryOperator::Identical => fold_identical(left, right),
        BinaryOperator::NotIdentical => fold_identical(left, right).map(|result| !result),
        BinaryOperator::Equal => fold_loose_equal(left, right),
        BinaryOperator::NotEqual => fold_loose_equal(left, right).map(|result| !result),
        BinaryOperator::LessThan => fold_ordering(left, right).map(|ordering| ordering == Ordering::Less),
        BinaryOperator::LessThanOrEqual => fold_ordering(left, right).map(|ordering| ordering != Ordering::Greater),
        BinaryOperator::GreaterThan => fold_ordering(left, right).map(|ordering| ordering == Ordering::Greater),
        BinaryOperator::GreaterThanOrEqual => fold_ordering(left, right).map(|ordering| ordering != Ordering::Less),
        _ => None,
    };

    match folded {
        Some(true) => TYPE_TRUE,
        Some(false) => TYPE_FALSE,
        None => TYPE_BOOL,
    }
}

fn fold_loose_equal(left: Type<'_>, right: Type<'_>) -> Option<bool> {
    match (comparable_of(left)?, comparable_of(right)?) {
        (Comparable::Number(left), Comparable::Number(right)) => Some(left.as_f64() == right.as_f64()),
        (Comparable::String(left), Comparable::String(right)) => Some(left == right),
        (Comparable::Bool(left), Comparable::Bool(right)) => Some(left == right),
        (Comparable::Null, Comparable::Null) => Some(true),
        _ => None,
    }
}

fn fold_ordering(left: Type<'_>, right: Type<'_>) -> Option<Ordering> {
    loose_compare(comparable_of(left)?, comparable_of(right)?)
}

fn loose_compare(left: Comparable<'_>, right: Comparable<'_>) -> Option<Ordering> {
    match (left, right) {
        (Comparable::Number(Number::Int(left)), Comparable::Number(Number::Int(right))) => Some(left.cmp(&right)),
        (Comparable::Number(left), Comparable::Number(right)) => left.as_f64().partial_cmp(&right.as_f64()),
        (Comparable::String(left), Comparable::String(right)) => Some(left.cmp(right)),
        _ => None,
    }
}

fn logical<'arena>(operator: BinaryOperator, left: Type<'arena>, right: Type<'arena>) -> Type<'arena> {
    let left = truthiness(left);
    let right = truthiness(right);

    let result = match operator {
        BinaryOperator::And => {
            if left == Some(false) || right == Some(false) {
                Some(false)
            } else if left == Some(true) && right == Some(true) {
                Some(true)
            } else {
                None
            }
        }
        BinaryOperator::Or => {
            if left == Some(true) || right == Some(true) {
                Some(true)
            } else if left == Some(false) && right == Some(false) {
                Some(false)
            } else {
                None
            }
        }
        BinaryOperator::Xor => match (left, right) {
            (Some(left), Some(right)) => Some(left ^ right),
            _ => None,
        },
        _ => None,
    };

    match result {
        Some(true) => TYPE_TRUE,
        Some(false) => TYPE_FALSE,
        None => TYPE_BOOL,
    }
}
