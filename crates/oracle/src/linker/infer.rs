use mago_allocator::Arena;
use mago_hir::ir::expression::ArrayElement;
use mago_hir::ir::expression::ArrayElementKind;
use mago_hir::ir::expression::CompositeStringPart;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::MagicConstantKind;
use mago_hir::ir::expression::operator::BinaryOperator;
use mago_hir::ir::expression::operator::UnaryPrefixOperator;
use mago_hir::ir::literal::LiteralKind;

use crate::linker::lower::Lowerer;
use crate::ty::Atom;
use crate::ty::Type;
use crate::ty::atom::payload::array::ArrayKey;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::well_known;

impl<'arena, S, A> Lowerer<'_, '_, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// A simple, best-effort constant-expression inference: enough to type
    /// global-constant values and parameter defaults at link time without the
    /// full program inference pass. Literals fold to literal types, constant
    /// arithmetic/concatenation/bitwise operations fold when both operands are
    /// known, and array literals become shapes. Anything it cannot resolve
    /// returns `None`, deferring to the later inference pass.
    pub(crate) fn infer<I, St, Ex>(&mut self, expression: &Expression<'arena, I, St, Ex>) -> Option<Type<'arena>> {
        match &expression.kind {
            ExpressionKind::Parenthesized(inner) => self.infer(inner),
            ExpressionKind::Literal(literal) => Some(match literal.kind {
                LiteralKind::Integer(integer) => match integer.value {
                    Some(value) => self.atom_type(Atom::int_literal(value as i64)),
                    None => well_known::TYPE_INT_OR_FLOAT,
                },
                LiteralKind::Float(_) => well_known::TYPE_FLOAT,
                LiteralKind::String(string) => match string.value {
                    Some([]) => self.atom_type(well_known::EMPTY_STRING),
                    Some(value) => {
                        let atom = self.builder.string_literal_atom(value);
                        self.atom_type(atom)
                    }
                    None => well_known::TYPE_STRING,
                },
                LiteralKind::True => well_known::TYPE_TRUE,
                LiteralKind::False => well_known::TYPE_FALSE,
                LiteralKind::Null => well_known::TYPE_NULL,
            }),
            ExpressionKind::CompositeString(parts) => {
                let non_empty = parts.iter().any(|part| match part {
                    CompositeStringPart::Literal(raw) => !raw.is_empty(),
                    _ => true,
                });
                Some(if non_empty { self.atom_type(well_known::NON_EMPTY_STRING) } else { well_known::TYPE_STRING })
            }
            ExpressionKind::MagicConstant(magic) => Some(match magic.kind {
                MagicConstantKind::Line => well_known::TYPE_INT,
                _ => well_known::TYPE_STRING,
            }),
            ExpressionKind::Constant(_) | ExpressionKind::Identifier(_) => None,
            ExpressionKind::UnaryPrefix(unary) => self.infer_unary(unary.operator, unary.operand),
            ExpressionKind::Binary(binary) => self.infer_binary(binary.operator, binary.left, binary.right),
            ExpressionKind::Array(elements) | ExpressionKind::List(elements) => self.infer_array(elements.as_slice()),
            ExpressionKind::Print(_) => Some(self.atom_type(Atom::int_literal(1))),
            ExpressionKind::Isset(_) | ExpressionKind::Empty(_) => Some(well_known::TYPE_BOOL),
            ExpressionKind::Clone(inner) => self.infer(inner),
            _ => None,
        }
    }

    /// Folds a unary-prefix operation over an inferred operand.
    fn infer_unary<I, St, Ex>(
        &mut self,
        operator: UnaryPrefixOperator,
        operand: &Expression<'arena, I, St, Ex>,
    ) -> Option<Type<'arena>> {
        match operator {
            UnaryPrefixOperator::Plus => self.infer(operand),
            UnaryPrefixOperator::Negation => {
                let operand = self.infer(operand)?;
                Some(match single_int(operand) {
                    Some(value) => self.atom_type(Atom::int_literal(value.wrapping_neg())),
                    None => operand,
                })
            }
            UnaryPrefixOperator::BitwiseNot => {
                let operand = self.infer(operand)?;
                Some(match single_int(operand) {
                    Some(value) => self.atom_type(Atom::int_literal(!value)),
                    None => well_known::TYPE_INT,
                })
            }
            UnaryPrefixOperator::Not => Some(well_known::TYPE_BOOL),
            UnaryPrefixOperator::BoolCast => Some(well_known::TYPE_BOOL),
            UnaryPrefixOperator::IntCast => Some(well_known::TYPE_INT),
            UnaryPrefixOperator::FloatCast => Some(well_known::TYPE_FLOAT),
            UnaryPrefixOperator::StringCast => Some(well_known::TYPE_STRING),
            UnaryPrefixOperator::ObjectCast => Some(well_known::TYPE_OBJECT),
            UnaryPrefixOperator::ArrayCast => Some(self.atom_type(well_known::ARRAY_KEY_MIXED)),
            UnaryPrefixOperator::UnsetCast => Some(well_known::TYPE_NULL),
            UnaryPrefixOperator::VoidCast => Some(well_known::TYPE_VOID),
            _ => None,
        }
    }

    /// Folds a binary operation: string concatenation, bitwise, and arithmetic
    /// operators fold to a literal when both operands are known literals, and to
    /// the operator's result type otherwise.
    fn infer_binary<I, St, Ex>(
        &mut self,
        operator: BinaryOperator,
        left: &Expression<'arena, I, St, Ex>,
        right: &Expression<'arena, I, St, Ex>,
    ) -> Option<Type<'arena>> {
        match operator {
            BinaryOperator::StringConcat => Some(well_known::TYPE_STRING),
            BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::LeftShift
            | BinaryOperator::RightShift => {
                let folded = self.fold_int_pair(left, right, |left, right| match operator {
                    BinaryOperator::BitwiseAnd => Some(left & right),
                    BinaryOperator::BitwiseOr => Some(left | right),
                    BinaryOperator::BitwiseXor => Some(left ^ right),
                    BinaryOperator::LeftShift => u32::try_from(right).ok().and_then(|shift| left.checked_shl(shift)),
                    BinaryOperator::RightShift => u32::try_from(right).ok().and_then(|shift| left.checked_shr(shift)),
                    _ => None,
                });
                Some(folded.unwrap_or(well_known::TYPE_INT))
            }
            BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Modulo
            | BinaryOperator::Exponentiation
            | BinaryOperator::Division => {
                let folded = self.fold_int_pair(left, right, |left, right| match operator {
                    BinaryOperator::Addition => left.checked_add(right),
                    BinaryOperator::Subtraction => left.checked_sub(right),
                    BinaryOperator::Multiplication => left.checked_mul(right),
                    BinaryOperator::Modulo if right != 0 => left.checked_rem(right),
                    BinaryOperator::Exponentiation if right >= 0 => {
                        u32::try_from(right).ok().and_then(|exponent| left.checked_pow(exponent))
                    }
                    BinaryOperator::Division if right != 0 && left % right == 0 => left.checked_div(right),
                    _ => None,
                });
                Some(folded.unwrap_or(well_known::TYPE_INT_OR_FLOAT))
            }
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Identical
            | BinaryOperator::NotIdentical
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Xor
            | BinaryOperator::Instanceof => Some(well_known::TYPE_BOOL),
            _ => None,
        }
    }

    /// Folds two operands that both infer to literal integers through `combine`,
    /// returning the literal-int result type when both are known and `combine`
    /// succeeds.
    fn fold_int_pair<I, St, Ex>(
        &mut self,
        left: &Expression<'arena, I, St, Ex>,
        right: &Expression<'arena, I, St, Ex>,
        combine: impl Fn(i64, i64) -> Option<i64>,
    ) -> Option<Type<'arena>> {
        let left = self.infer(left).and_then(single_int)?;
        let right = self.infer(right).and_then(single_int)?;
        let value = combine(left, right)?;

        Some(self.atom_type(Atom::int_literal(value)))
    }

    /// Infers an array/list literal into a shape: a positional list becomes a
    /// sealed `list{…}`, a fully-keyed array a sealed `array{…}`.
    fn infer_array<I, St, Ex>(&mut self, elements: &[ArrayElement<'arena, I, St, Ex>]) -> Option<Type<'arena>> {
        use crate::ty::atom::payload::array::KnownElement;
        use crate::ty::atom::payload::array::KnownItem;

        if elements.iter().all(|element| matches!(element.kind, ArrayElementKind::Value(_))) {
            let mut known = self.builder.scratch_vec();
            for (index, element) in elements.iter().enumerate() {
                let ArrayElementKind::Value(value) = element.kind else {
                    return None;
                };
                let value = self.infer(value).unwrap_or(well_known::TYPE_MIXED);
                known.push(KnownElement { index: index as u32, value, optional: false });
            }
            let atom = self.builder.sealed_list_atom(&known, !elements.is_empty());

            return Some(self.atom_type(atom));
        }

        if elements.iter().all(|element| matches!(element.kind, ArrayElementKind::KeyValue(_, _))) {
            let mut known = self.builder.scratch_vec();
            for element in elements {
                let ArrayElementKind::KeyValue(key, value) = element.kind else {
                    return None;
                };
                let key = self.infer(key).and_then(array_key)?;
                let value = self.infer(value).unwrap_or(well_known::TYPE_MIXED);
                known.push(KnownItem { key, value, optional: false });
            }
            let atom = self.builder.sealed_keyed_array_atom(&known, !elements.is_empty());

            return Some(self.atom_type(atom));
        }

        Some(self.atom_type(well_known::ARRAY_KEY_MIXED))
    }

    /// Interns a single-atom type through the builder.
    fn atom_type(&mut self, atom: Atom<'arena>) -> Type<'arena> {
        self.builder.union_of(&[atom])
    }
}

/// The literal integer value of a single-atom integer type, if it is one.
fn single_int(ty: Type<'_>) -> Option<i64> {
    match ty.atoms {
        [Atom::Int(IntAtom::Literal(value))] => Some(*value),
        _ => None,
    }
}

/// The array key for a single-atom literal int or string type, if it is one.
fn array_key(ty: Type<'_>) -> Option<ArrayKey<'_>> {
    match ty.atoms {
        [Atom::Int(IntAtom::Literal(value))] => Some(ArrayKey::Int(*value)),
        [Atom::String(string)] => match string.literal {
            StringLiteral::Value(value) => Some(ArrayKey::String(value)),
            _ => None,
        },
        _ => None,
    }
}
