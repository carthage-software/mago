#![allow(clippy::float_arithmetic)]

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::UnaryPostfix;
use mago_hir::ir::expression::UnaryPrefix;
use mago_hir::ir::expression::operator::UnaryPostfixOperator;
use mago_hir::ir::expression::operator::UnaryPrefixOperator;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::well_known::ARRAY_KEY_MIXED;
use mago_oracle::ty::well_known::TYPE_BOOL;
use mago_oracle::ty::well_known::TYPE_FALSE;
use mago_oracle::ty::well_known::TYPE_FLOAT;
use mago_oracle::ty::well_known::TYPE_INT;
use mago_oracle::ty::well_known::TYPE_INT_OR_FLOAT;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_oracle::ty::well_known::TYPE_NULL;
use mago_oracle::ty::well_known::TYPE_OBJECT;
use mago_oracle::ty::well_known::TYPE_STRING;
use mago_oracle::ty::well_known::TYPE_TRUE;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::semantics::Number;
use crate::semantics::append_string;
use crate::semantics::is_array_type;
use crate::semantics::literal_string_bytes;
use crate::semantics::number_of;
use crate::semantics::parse_php_number;
use crate::semantics::truthiness;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_unary_prefix(
        &mut self,
        span: Span,
        unary: &'source UnaryPrefix<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let operand = self.infer_expression(unary.operand);

        let meta = if operand.meta.is_never() {
            TYPE_NEVER
        } else {
            match unary.operator {
                UnaryPrefixOperator::PreIncrement => {
                    let incremented = self.increment_type(operand.meta);
                    self.write_back(unary.operand, incremented);

                    incremented
                }
                UnaryPrefixOperator::PreDecrement => {
                    let decremented = self.decrement_type(operand.meta);
                    self.write_back(unary.operand, decremented);

                    decremented
                }
                operator => self.unary_prefix_type(operator, operand.meta),
            }
        };

        let unary = UnaryPrefix { span: unary.span, operator: unary.operator, operand: self.arena.alloc(operand) };

        Expression { meta, span, kind: ExpressionKind::UnaryPrefix(self.arena.alloc(unary)) }
    }

    pub fn infer_unary_postfix(
        &mut self,
        span: Span,
        unary: &'source UnaryPostfix<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let operand = self.infer_expression(unary.operand);

        let meta = if operand.meta.is_never() {
            TYPE_NEVER
        } else {
            let updated = match unary.operator {
                UnaryPostfixOperator::PostIncrement => self.increment_type(operand.meta),
                UnaryPostfixOperator::PostDecrement => self.decrement_type(operand.meta),
            };
            self.write_back(unary.operand, updated);

            operand.meta
        };

        let unary = UnaryPostfix { span: unary.span, operand: self.arena.alloc(operand), operator: unary.operator };

        Expression { meta, span, kind: ExpressionKind::UnaryPostfix(self.arena.alloc(unary)) }
    }

    fn unary_prefix_type(&mut self, operator: UnaryPrefixOperator, operand: Type<'arena>) -> Type<'arena> {
        match operator {
            UnaryPrefixOperator::Negation => match number_of(operand) {
                Some(Number::Int(value)) => match value.checked_neg() {
                    Some(value) => self.int_literal(value),
                    None => self.float_literal(-(value as f64)),
                },
                Some(Number::Float(value)) => self.float_literal(-value),
                None => TYPE_INT_OR_FLOAT,
            },
            UnaryPrefixOperator::Plus => match number_of(operand) {
                Some(Number::Int(value)) => self.int_literal(value),
                Some(Number::Float(value)) => self.float_literal(value),
                None => TYPE_INT_OR_FLOAT,
            },
            UnaryPrefixOperator::Not => match truthiness(operand) {
                Some(true) => TYPE_FALSE,
                Some(false) => TYPE_TRUE,
                None => TYPE_BOOL,
            },
            UnaryPrefixOperator::BitwiseNot => {
                if let Some(bytes) = literal_string_bytes(operand) {
                    let mut result = Vec::new_in(self.source);
                    for byte in bytes {
                        result.push(!*byte);
                    }

                    self.literal_string(&result)
                } else {
                    match number_of(operand) {
                        Some(number) => self.int_literal(!number.to_int()),
                        None => TYPE_INT,
                    }
                }
            }
            UnaryPrefixOperator::IntCast => match cast_number(operand) {
                Some(number) => self.int_literal(number.to_int()),
                None => TYPE_INT,
            },
            UnaryPrefixOperator::FloatCast => match cast_number(operand) {
                Some(number) => self.float_literal(number.as_f64()),
                None => TYPE_FLOAT,
            },
            UnaryPrefixOperator::BoolCast => match truthiness(operand) {
                Some(true) => TYPE_TRUE,
                Some(false) => TYPE_FALSE,
                None => TYPE_BOOL,
            },
            UnaryPrefixOperator::StringCast => {
                let mut bytes = Vec::new_in(self.source);
                if append_string(&mut bytes, operand) { self.literal_string(&bytes) } else { TYPE_STRING }
            }
            UnaryPrefixOperator::ArrayCast => self.ty.union_of(&[ARRAY_KEY_MIXED]),
            UnaryPrefixOperator::ObjectCast => TYPE_OBJECT,
            UnaryPrefixOperator::UnsetCast | UnaryPrefixOperator::VoidCast => TYPE_NULL,
            UnaryPrefixOperator::ErrorControl
            | UnaryPrefixOperator::Reference
            | UnaryPrefixOperator::PreIncrement
            | UnaryPrefixOperator::PreDecrement => operand,
        }
    }

    fn increment_type(&mut self, ty: Type<'arena>) -> Type<'arena> {
        let [atom] = ty.atoms else {
            return ty;
        };

        match atom {
            Atom::Int(IntAtom::Literal(value)) => match value.checked_add(1) {
                Some(value) => self.int_literal(value),
                None => self.float_literal(*value as f64 + 1.0),
            },
            Atom::Float(FloatAtom::Literal(value)) => self.float_literal(value.0.into_inner() + 1.0),
            Atom::Null => self.int_literal(1),
            Atom::True | Atom::False | Atom::Bool => ty,
            Atom::String(string) => match string.literal {
                StringLiteral::Value(value) => match parse_php_number(value) {
                    Some(Number::Int(number)) => match number.checked_add(1) {
                        Some(number) => self.int_literal(number),
                        None => self.float_literal(number as f64 + 1.0),
                    },
                    Some(Number::Float(number)) => self.float_literal(number + 1.0),
                    None => {
                        let mut next = Vec::new_in(self.source);
                        if php_string_increment(value, &mut next) { self.literal_string(&next) } else { ty }
                    }
                },
                _ => TYPE_STRING,
            },
            Atom::Int(_) => TYPE_INT,
            Atom::Float(_) => TYPE_FLOAT,
            _ => ty,
        }
    }

    fn decrement_type(&mut self, ty: Type<'arena>) -> Type<'arena> {
        let [atom] = ty.atoms else {
            return ty;
        };

        match atom {
            Atom::Int(IntAtom::Literal(value)) => match value.checked_sub(1) {
                Some(value) => self.int_literal(value),
                None => self.float_literal(*value as f64 - 1.0),
            },
            Atom::Float(FloatAtom::Literal(value)) => self.float_literal(value.0.into_inner() - 1.0),
            Atom::Null | Atom::True | Atom::False | Atom::Bool => ty,
            Atom::String(string) => match string.literal {
                StringLiteral::Value(value) => match parse_php_number(value) {
                    Some(Number::Int(number)) => match number.checked_sub(1) {
                        Some(number) => self.int_literal(number),
                        None => self.float_literal(number as f64 - 1.0),
                    },
                    Some(Number::Float(number)) => self.float_literal(number - 1.0),
                    None => ty,
                },
                _ => TYPE_STRING,
            },
            Atom::Int(_) => TYPE_INT,
            Atom::Float(_) => TYPE_FLOAT,
            _ => ty,
        }
    }

    fn write_back(&mut self, operand: &'source Expression<'source, SymbolId, S, E>, ty: Type<'arena>) {
        if let ExpressionKind::Variable(Variable::Direct(direct)) = &operand.kind {
            let name = self.arena.alloc_slice_copy(direct.name);
            self.environment.set(Var::new(name), ty);
        }
    }
}

/// PHP coerces an array to a number through `(int)`/`(float)` by its emptiness:
/// an empty array is `0` and any non-empty array is `1`. Scalars keep their usual
/// numeric coercion.
fn cast_number(ty: Type<'_>) -> Option<Number> {
    if let Some(number) = number_of(ty) {
        return Some(number);
    }

    if is_array_type(ty) {
        return match truthiness(ty) {
            Some(true) => Some(Number::Int(1)),
            Some(false) => Some(Number::Int(0)),
            None => None,
        };
    }

    None
}

/// PHP's Perl-style string increment (`"a"`→`"b"`, `"Az"`→`"Ba"`, `"Zz"`→`"AAa"`,
/// `"a9"`→`"b0"`). Only applies to non-empty all-alphanumeric strings; returns
/// `false` (caller keeps the value unchanged) otherwise.
fn php_string_increment<A>(input: &[u8], output: &mut Vec<'_, u8, A>) -> bool
where
    A: Arena,
{
    if input.is_empty() || !input.iter().all(|byte| byte.is_ascii_alphanumeric()) {
        return false;
    }

    output.extend_from_slice(input);

    let mut index = output.len();
    while index > 0 {
        index -= 1;

        let (next, carry) = match output[index] {
            byte @ (b'0'..=b'8' | b'a'..=b'y' | b'A'..=b'Y') => (byte + 1, false),
            b'9' => (b'0', true),
            b'z' => (b'a', true),
            b'Z' => (b'A', true),
            byte => (byte, false),
        };

        output[index] = next;

        if !carry {
            return true;
        }

        if index == 0 {
            output.insert(
                0,
                match next {
                    b'0' => b'1',
                    b'a' => b'a',
                    _ => b'A',
                },
            );

            return true;
        }
    }

    true
}
