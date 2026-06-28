use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_flags::U8Flags;
use mago_hir::ir::expression::Binary;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::operator::BinaryOperator;
use mago_hir::ir::expression::operator::UnaryPrefixOperator;
use mago_hir::ir::variable::Variable;
use mago_oracle::assertion::Assertion;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::float::LiteralFloat;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known::EMPTY_ARRAY;
use mago_oracle::ty::well_known::EMPTY_STRING;
use mago_oracle::ty::well_known::FALSE;
use mago_oracle::ty::well_known::NULL;
use mago_oracle::ty::well_known::TYPE_NULL;
use mago_oracle::var::Var;
use ordered_float::OrderedFloat;

use crate::flow::Flow;
use crate::tdd::DecisionDiagram;
use crate::tdd::Literal;
use crate::tdd::Node;

/// Narrows `ty` under `assertion`, leaning on the oracle lattice.
///
/// An unhandled assertion leaves the type unchanged, and a narrowing with no inhabitants
/// yields `never` (the caller reads that as an impossible/unreachable path).
#[must_use]
pub fn reconcile<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    assertion: Assertion<'arena>,
    ty: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    match assertion {
        Assertion::IsIdentical(atom) | Assertion::IsType(atom) | Assertion::IsEqual(atom) => {
            let narrowing = builder.union_of(&[atom]);

            meet_with(builder, symbols, ty, narrowing)
        }
        Assertion::IsNotIdentical(atom) | Assertion::IsNotType(atom) | Assertion::IsNotEqual(atom) => {
            let removed = builder.union_of(&[atom]);

            subtract_with(builder, symbols, ty, removed)
        }
        Assertion::Truthy => filter_truthy(builder, symbols, ty),
        Assertion::Falsy => filter_falsy(builder, symbols, ty),
        Assertion::IsIsset => subtract_with(builder, symbols, ty, TYPE_NULL),
        Assertion::IsNotIsset => meet_with(builder, symbols, ty, TYPE_NULL),
        _ => ty,
    }
}

/// The lattice meet (intersection) of two types: the values inhabiting both.
#[must_use]
pub fn meet_with<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
    other: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut report = LatticeReport::new();

    meet::compute(ty, other, symbols, LatticeOptions::default(), &mut report, builder)
}

/// `ty` with every value also inhabiting `other` removed.
#[must_use]
pub fn subtract_with<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
    other: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut report = LatticeReport::new();

    subtract::compute(ty, other, symbols, LatticeOptions::default(), &mut report, builder)
}

fn filter_truthy<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let falsy = falsy_type(builder);

    subtract_with(builder, symbols, ty, falsy)
}

fn filter_falsy<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let falsy = falsy_type(builder);

    meet_with(builder, symbols, ty, falsy)
}

fn falsy_type<'arena, S, A>(builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let zero_string = builder.intern(b"0");
    let zero_string = builder.string(StringAtom {
        literal: StringLiteral::Value(zero_string),
        casing: StringCasing::Unspecified,
        flags: U8Flags::empty(),
    });

    builder.union_of(&[
        NULL,
        FALSE,
        Atom::Int(IntAtom::Literal(0)),
        Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(0.0)))),
        EMPTY_STRING,
        zero_string,
        EMPTY_ARRAY,
    ])
}

/// Collects the per-variable assertions that definitely hold when `expr`
/// evaluates to `polarity`.
///
/// Only the conjunctive facts are kept: `A && B` under `true` contributes both sides;
/// the disjunctive directions (`A && B` false, `A || B` true) narrow nothing soundly and are skipped.
pub fn narrowing_assertions<'arena, A>(
    expr: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    polarity: bool,
    out: &mut Vec<'_, (Var<'arena>, Assertion<'arena>), A>,
) where
    A: Arena,
{
    match &expr.kind {
        ExpressionKind::Parenthesized(inner) => narrowing_assertions(inner, polarity, out),
        ExpressionKind::Variable(Variable::Direct(direct)) => {
            out.push((Var::new(direct.name), if polarity { Assertion::Truthy } else { Assertion::Falsy }));
        }
        ExpressionKind::UnaryPrefix(unary) if matches!(unary.operator, UnaryPrefixOperator::Not) => {
            narrowing_assertions(unary.operand, !polarity, out);
        }
        ExpressionKind::Empty(operand) => narrowing_assertions(operand, !polarity, out),
        ExpressionKind::Isset(delimited) if polarity => {
            for operand in delimited.items {
                if let ExpressionKind::Variable(Variable::Direct(direct)) = &operand.kind {
                    out.push((Var::new(direct.name), Assertion::IsIsset));
                }
            }
        }
        ExpressionKind::Binary(binary) => match binary.operator {
            BinaryOperator::And if polarity => {
                narrowing_assertions(binary.left, true, out);
                narrowing_assertions(binary.right, true, out);
            }
            BinaryOperator::Or if !polarity => {
                narrowing_assertions(binary.left, false, out);
                narrowing_assertions(binary.right, false, out);
            }
            BinaryOperator::Identical | BinaryOperator::NotIdentical => {
                if let Some((variable, atom)) = comparison_literal(binary) {
                    let positive = matches!(binary.operator, BinaryOperator::Identical) == polarity;
                    let assertion =
                        if positive { Assertion::IsIdentical(atom) } else { Assertion::IsNotIdentical(atom) };

                    out.push((variable, assertion));
                }
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if let Some((variable, atom)) = comparison_literal(binary)
                    && let Some(assertion) =
                        loose_equality_assertion(atom, matches!(binary.operator, BinaryOperator::Equal), polarity)
                {
                    out.push((variable, assertion));
                }
            }
            _ => {}
        },
        _ => {}
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

/// Builds the boolean-structure decision diagram of a condition expression.
///
/// Negatives are encoded as `not` over the canonical positive literal (so
/// `$x !== null` is `!($x === null)`, letting `$x === null && $x !== null`
/// collapse to `⊥`). Returns `None` for any sub-expression with no literal form
/// — the caller then cannot fold it structurally.
pub fn condition_diagram<'arena, A>(
    diagram: &mut DecisionDiagram<'_, 'arena, A>,
    expr: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Node>
where
    A: Arena,
{
    match &expr.kind {
        ExpressionKind::Parenthesized(inner) => condition_diagram(diagram, inner),
        ExpressionKind::Variable(Variable::Direct(direct)) => {
            Some(diagram.literal(Literal { variable: Var::new(direct.name), assertion: Assertion::Truthy }))
        }
        ExpressionKind::UnaryPrefix(unary) if matches!(unary.operator, UnaryPrefixOperator::Not) => {
            let inner = condition_diagram(diagram, unary.operand)?;

            Some(diagram.not(inner))
        }
        ExpressionKind::Empty(operand) => {
            let inner = condition_diagram(diagram, operand)?;

            Some(diagram.not(inner))
        }
        ExpressionKind::Isset(delimited) => {
            let mut node = Node::TRUE;
            for operand in delimited.items {
                let ExpressionKind::Variable(Variable::Direct(direct)) = &operand.kind else {
                    return None;
                };

                let literal =
                    diagram.literal(Literal { variable: Var::new(direct.name), assertion: Assertion::IsIsset });
                node = diagram.and(node, literal);
            }

            Some(node)
        }
        ExpressionKind::Binary(binary) => match binary.operator {
            BinaryOperator::And => {
                let left = condition_diagram(diagram, binary.left)?;
                let right = condition_diagram(diagram, binary.right)?;

                Some(diagram.and(left, right))
            }
            BinaryOperator::Or => {
                let left = condition_diagram(diagram, binary.left)?;
                let right = condition_diagram(diagram, binary.right)?;

                Some(diagram.or(left, right))
            }
            BinaryOperator::Identical | BinaryOperator::NotIdentical => {
                let (variable, atom) = comparison_literal(binary)?;
                let node = diagram.literal(Literal { variable, assertion: Assertion::IsIdentical(atom) });

                Some(if matches!(binary.operator, BinaryOperator::NotIdentical) { diagram.not(node) } else { node })
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                let (variable, atom) = comparison_literal(binary)?;
                let truthy = diagram.literal(Literal { variable, assertion: Assertion::Truthy });
                let is_equal = matches!(binary.operator, BinaryOperator::Equal);

                match atom {
                    Atom::True => Some(if is_equal { truthy } else { diagram.not(truthy) }),
                    Atom::False => Some(if is_equal { diagram.not(truthy) } else { truthy }),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

fn comparison_literal<'arena>(
    binary: &Binary<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<(Var<'arena>, Atom<'arena>)> {
    direct_and_atom(binary.left, binary.right).or_else(|| direct_and_atom(binary.right, binary.left))
}

fn direct_and_atom<'arena>(
    variable_side: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    atom_side: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<(Var<'arena>, Atom<'arena>)> {
    if let ExpressionKind::Variable(Variable::Direct(direct)) = &variable_side.kind
        && let [atom] = atom_side.meta.atoms
    {
        Some((Var::new(direct.name), *atom))
    } else {
        None
    }
}
