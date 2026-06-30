use mago_allocator::LocalArena;
use mago_inference::tdd::DecisionDiagram;
use mago_inference::tdd::Literal;
use mago_inference::tdd::Node;
use mago_oracle::assertion::Assertion;
use mago_oracle::ty::well_known::FALSE;
use mago_oracle::ty::well_known::NULL;
use mago_oracle::var::Var;

fn is_null(variable: &[u8]) -> Literal<'_> {
    Literal { variable: Var::new(variable), assertion: Assertion::IsIdentical(NULL) }
}

fn is_false(variable: &[u8]) -> Literal<'_> {
    Literal { variable: Var::new(variable), assertion: Assertion::IsIdentical(FALSE) }
}

#[test]
fn complementary_laws() {
    let arena = LocalArena::new();
    let mut diagram = DecisionDiagram::new_in(&arena);

    let a = diagram.literal(is_null(b"$a"));
    assert!(!a.is_terminal());

    let not_a = diagram.not(a);
    assert!(diagram.and(a, not_a).is_false(), "x AND not-x is a contradiction");
    assert!(diagram.or(a, not_a).is_true(), "x OR not-x is a tautology");
    assert_eq!(diagram.not(not_a), a, "double negation is canonical");
}

#[test]
fn operations_are_canonical() {
    let arena = LocalArena::new();
    let mut diagram = DecisionDiagram::new_in(&arena);

    let a = diagram.literal(is_null(b"$a"));
    let b = diagram.literal(is_false(b"$a"));

    assert_eq!(diagram.and(a, b), diagram.and(b, a), "AND is order-independent");
    assert_eq!(diagram.or(a, b), diagram.or(b, a), "OR is order-independent");
    assert_eq!(diagram.and(a, a), a, "AND is idempotent");
    assert_eq!(diagram.and(a, Node::TRUE), a);
    assert_eq!(diagram.or(a, Node::FALSE), a);
}

#[test]
fn elseif_chain_else_is_contradiction() {
    let arena = LocalArena::new();
    let mut diagram = DecisionDiagram::new_in(&arena);

    let l1 = diagram.literal(is_null(b"$a"));
    let l2 = diagram.literal(is_false(b"$a"));

    let not_l1 = diagram.not(l1);
    let not_l2 = diagram.not(l2);
    let not_cond3 = diagram.not(not_l1);

    let guard = diagram.and(not_l1, not_l2);
    let guard = diagram.and(guard, not_cond3);

    assert!(guard.is_false(), "the final else is unreachable");
}

#[test]
fn restrict_cofactors_a_literal() {
    let arena = LocalArena::new();
    let mut diagram = DecisionDiagram::new_in(&arena);

    let l1 = is_null(b"$a");
    let a = diagram.literal(l1);

    assert!(diagram.restrict(a, l1, true).is_true(), "$a===null restricted with $a===null is true");
    assert!(diagram.restrict(a, l1, false).is_false(), "$a===null restricted with $a!==null is false");
}
