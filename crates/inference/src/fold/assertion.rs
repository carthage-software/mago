use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Binary;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::operator::BinaryOperatorKind;
use mago_hir::ir::expression::operator::UnaryPrefixOperatorKind;
use mago_oracle::assertion::Assertion;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::var::Var;

use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::tdd::DecisionDiagram;
use crate::tdd::Literal;
use crate::tdd::Node;

/// One narrowing fact a condition implies about a place: the place it constrains,
/// the type the place currently holds (the narrowing base, taken from the operand's
/// own inference so a derived place keeps its container-shaped type rather than
/// defaulting to `mixed`), and the assertion to apply.
pub(crate) type PlaceAssertion<'arena> = (Var<'arena>, Type<'arena>, Assertion<'arena>);

impl<'arena, A, S, E> InferenceFolder<'_, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// Collects the place assertions that definitely hold when `expr` evaluates to
    /// `polarity`. Each constrained operand is keyed by its place id, so the same
    /// machinery narrows a variable, a property, or an array element uniformly.
    ///
    /// Only the conjunctive facts are kept: `A && B` under `true` contributes both
    /// sides; the disjunctive directions narrow nothing soundly and are skipped.
    pub(crate) fn narrowing_assertions(
        &mut self,
        expr: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        polarity: bool,
        out: &mut Vec<'_, PlaceAssertion<'arena>, A>,
    ) {
        match &expr.kind {
            ExpressionKind::Parenthesized(inner) => self.narrowing_assertions(inner, polarity, out),
            ExpressionKind::UnaryPrefix(unary) if matches!(unary.operator.kind, UnaryPrefixOperatorKind::Not) => {
                self.narrowing_assertions(unary.operand, !polarity, out);
            }
            ExpressionKind::Empty(operand) => self.narrowing_assertions(operand, !polarity, out),
            ExpressionKind::Isset(delimited) if polarity => {
                for operand in delimited.items {
                    if let Some(place) = self.place_id(operand) {
                        out.push((place, operand.meta, Assertion::IsIsset));
                    }
                }
            }
            ExpressionKind::Binary(binary) => match binary.operator.kind {
                BinaryOperatorKind::And(_) if polarity => {
                    self.narrowing_assertions(binary.left, true, out);
                    self.narrowing_assertions(binary.right, true, out);
                }
                BinaryOperatorKind::Or(_) if !polarity => {
                    self.narrowing_assertions(binary.left, false, out);
                    self.narrowing_assertions(binary.right, false, out);
                }
                BinaryOperatorKind::Identical | BinaryOperatorKind::NotIdentical => {
                    if let Some((place, base, atom)) = self.comparison_literal(binary) {
                        let positive = matches!(binary.operator.kind, BinaryOperatorKind::Identical) == polarity;
                        let assertion =
                            if positive { Assertion::IsIdentical(atom) } else { Assertion::IsNotIdentical(atom) };

                        out.push((place, base, assertion));
                    }
                }
                BinaryOperatorKind::Equal | BinaryOperatorKind::NotEqual(_) => {
                    if let Some((place, base, atom)) = self.comparison_literal(binary)
                        && let Some(assertion) = loose_equality_assertion(
                            atom,
                            matches!(binary.operator.kind, BinaryOperatorKind::Equal),
                            polarity,
                        )
                    {
                        out.push((place, base, assertion));
                    }
                }
                _ => {}
            },
            _ => {
                if let Some(place) = self.place_id(expr) {
                    out.push((place, expr.meta, if polarity { Assertion::Truthy } else { Assertion::Falsy }));
                }
            }
        }
    }

    /// Builds the boolean-structure decision diagram of a condition expression.
    ///
    /// Negatives are encoded as `not` over the canonical positive literal (so
    /// `$x !== null` is `!($x === null)`, letting `$x === null && $x !== null`
    /// collapse to `⊥`). Returns `None` for any sub-expression with no place form
    /// — the caller then cannot fold it structurally.
    pub(crate) fn condition_diagram(
        &mut self,
        diagram: &mut DecisionDiagram<'_, 'arena, A>,
        expr: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<Node> {
        match &expr.kind {
            ExpressionKind::Parenthesized(inner) => self.condition_diagram(diagram, inner),
            ExpressionKind::UnaryPrefix(unary) if matches!(unary.operator.kind, UnaryPrefixOperatorKind::Not) => {
                let inner = self.condition_diagram(diagram, unary.operand)?;

                Some(diagram.not(inner))
            }
            ExpressionKind::Empty(operand) => {
                let inner = self.condition_diagram(diagram, operand)?;

                Some(diagram.not(inner))
            }
            ExpressionKind::Isset(delimited) => {
                let mut node = Node::TRUE;
                for operand in delimited.items {
                    let place = self.place_id(operand)?;
                    let literal = diagram.literal(Literal { variable: place, assertion: Assertion::IsIsset });
                    node = diagram.and(node, literal);
                }

                Some(node)
            }
            ExpressionKind::Binary(binary) => match binary.operator.kind {
                BinaryOperatorKind::And(_) => {
                    let left = self.condition_diagram(diagram, binary.left)?;
                    let right = self.condition_diagram(diagram, binary.right)?;

                    Some(diagram.and(left, right))
                }
                BinaryOperatorKind::Or(_) => {
                    let left = self.condition_diagram(diagram, binary.left)?;
                    let right = self.condition_diagram(diagram, binary.right)?;

                    Some(diagram.or(left, right))
                }
                BinaryOperatorKind::Identical | BinaryOperatorKind::NotIdentical => {
                    let (place, _, atom) = self.comparison_literal(binary)?;
                    let node = diagram.literal(Literal { variable: place, assertion: Assertion::IsIdentical(atom) });

                    Some(if matches!(binary.operator.kind, BinaryOperatorKind::NotIdentical) {
                        diagram.not(node)
                    } else {
                        node
                    })
                }
                BinaryOperatorKind::Equal | BinaryOperatorKind::NotEqual(_) => {
                    let (place, _, atom) = self.comparison_literal(binary)?;
                    let truthy = diagram.literal(Literal { variable: place, assertion: Assertion::Truthy });
                    let is_equal = matches!(binary.operator.kind, BinaryOperatorKind::Equal);

                    match atom {
                        Atom::True => Some(if is_equal { truthy } else { diagram.not(truthy) }),
                        Atom::False => Some(if is_equal { diagram.not(truthy) } else { truthy }),
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => {
                let place = self.place_id(expr)?;

                Some(diagram.literal(Literal { variable: place, assertion: Assertion::Truthy }))
            }
        }
    }

    fn comparison_literal(
        &mut self,
        binary: &Binary<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<(Var<'arena>, Type<'arena>, Atom<'arena>)> {
        if let Some(result) = self.place_and_atom(binary.left, binary.right) {
            return Some(result);
        }

        self.place_and_atom(binary.right, binary.left)
    }

    fn place_and_atom(
        &mut self,
        place_side: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        atom_side: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<(Var<'arena>, Type<'arena>, Atom<'arena>)> {
        let place = self.place_id(place_side)?;
        let [atom] = atom_side.meta.atoms else {
            return None;
        };

        Some((place, place_side.meta, *atom))
    }
}

/// Loose `==`/`!=` against a literal narrows soundly only for `true`/`false`,
/// where `$x == true` is exactly "`$x` is truthy" and `$x == false` exactly
/// "`$x` is falsy". Every other loose comparison (`== 5`, `== null`, ...) holds
/// for values of multiple types, so narrowing to the literal would drop real
/// possibilities; those produce no assertion.
fn loose_equality_assertion(atom: Atom<'_>, is_equal_operator: bool, polarity: bool) -> Option<Assertion<'_>> {
    let positive = is_equal_operator == polarity;

    match atom {
        Atom::True => Some(if positive { Assertion::Truthy } else { Assertion::Falsy }),
        Atom::False => Some(if positive { Assertion::Falsy } else { Assertion::Truthy }),
        _ => None,
    }
}
