mod comparator_common;

use std::collections::BTreeMap;
use std::sync::Arc;

use comparator_common::*;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::TType;
use mago_codex::ttype::TypeRef;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::callable::TCallableConstraint;
use mago_codex::ttype::atomic::callable::TCallableSignature;
use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
use mago_codex::ttype::atomic::conditional::TConditional;
use mago_codex::ttype::atomic::derived::TDerived;
use mago_codex::ttype::atomic::derived::key_of::TKeyOf;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use mago_codex::ttype::union::TUnion;
use mago_word::word;

fn type_trees() -> Vec<TUnion> {
    let generic = TAtomic::GenericParameter(TGenericParameter::new(
        word("T"),
        Arc::new(u(t_named("Base"))),
        GenericParent::ClassLike(word("Box")),
    ));
    let variable = TAtomic::Variable(word("$value"));
    let mut object = t_generic_named("Box", vec![u(generic.clone()), u(variable.clone())]);
    object.add_intersection_type(t_named("Countable"));

    let mut callable = TCallableSignature::new(false, true)
        .with_parameters(vec![TCallableParameter::new(Some(Arc::new(u(generic.clone()))), false, false, false)])
        .with_return_type(Some(Arc::new(u(object.clone()))));
    callable.constraints.push(TCallableConstraint::new(vec![], Arc::new(u(t_string())), Arc::new(u(variable.clone()))));

    vec![
        u(t_int()),
        u_many(vec![t_int(), t_string(), null()]),
        u(generic.clone()),
        u(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::generic(
            TClassLikeStringKind::Class,
            word("T"),
            GenericParent::ClassLike(word("Box")),
            t_named("Base"),
        )))),
        u(t_list(u(object.clone()), false)),
        u(t_sealed_list(BTreeMap::from([(0, (false, u(generic.clone()))), (1, (true, u(variable.clone())))]))),
        u(t_keyed_with_both(
            u(t_string()),
            u(generic.clone()),
            BTreeMap::from([(ak_str("item"), (false, u(object.clone())))]),
            false,
        )),
        u(t_iterable(u(t_int()), u(object.clone()))),
        u(object),
        u(TAtomic::Callable(TCallable::Signature(callable))),
        u(TAtomic::Conditional(TConditional::new(
            Arc::new(u(variable)),
            Arc::new(u(t_string())),
            Arc::new(u(generic.clone())),
            Arc::new(u(t_int())),
            false,
        ))),
        u(TAtomic::Derived(TDerived::KeyOf(TKeyOf::new(Arc::new(u(t_list(u(generic), false))))))),
    ]
}

fn node_address(node: TypeRef<'_>) -> (bool, usize) {
    match node {
        TypeRef::Union(union) => (true, std::ptr::from_ref(union) as usize),
        TypeRef::Atomic(atomic) => (false, std::ptr::from_ref(atomic) as usize),
    }
}

fn check_traversal(ttype: &impl TType) {
    let expected: Vec<_> = ttype.get_all_child_nodes().into_iter().map(node_address).collect();
    let mut visited = Vec::new();
    assert!(
        !ttype.any_child_node(|node| {
            visited.push(node_address(node));
            false
        }),
        "a predicate that is always false must visit every child"
    );
    assert_eq!(visited, expected, "both traversals must visit the same children in the same order");

    for stop in 0..expected.len() {
        let mut visited = Vec::new();
        assert!(
            ttype.any_child_node(|node| {
                visited.push(node_address(node));
                visited.len() == stop + 1
            }),
            "the predicate must find the chosen child"
        );
        assert_eq!(visited, expected[..=stop], "traversal must stop at the first matching child");
    }
}

#[test]
fn type_trait_remains_object_safe() {
    let union = u(t_int());
    let ttype: &dyn TType = &union;
    assert_eq!(ttype.get_id(), union.get_id());
    assert_eq!(ttype.get_all_child_nodes().len(), union.get_all_child_nodes().len());
}

#[test]
fn traversal_preserves_child_coverage_order_and_early_exit() {
    for union in type_trees() {
        check_traversal(&union);
        for atomic in union.types.iter() {
            check_traversal(atomic);
            check_traversal(&TypeRef::Atomic(atomic));
        }
        check_traversal(&TypeRef::Union(&union));
    }
}

#[test]
fn template_predicate_preserves_nested_template_coverage() {
    for union in type_trees() {
        let expected = union.get_all_child_nodes().into_iter().any(|node| {
            matches!(
                node,
                TypeRef::Atomic(
                    TAtomic::GenericParameter(_)
                        | TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic { .. }))
                )
            )
        });
        assert_eq!(union.has_template_types(), expected);
    }
}

#[test]
fn unspecified_template_predicate_checks_root_and_nested_unions() {
    let mut omitted = u(t_int());
    omitted.set_from_unspecified_template(true);
    assert!(omitted.contains_unspecified_template_arguments());
    assert!(u(t_list(omitted, false)).contains_unspecified_template_arguments());
    assert!(!u(t_list(u(t_int()), false)).contains_unspecified_template_arguments());
}
