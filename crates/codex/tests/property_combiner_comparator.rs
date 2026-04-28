#![cfg(feature = "property-tests")]

use std::sync::Arc;

use mago_atom::atom;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::callable::TCallableSignature;
use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
use mago_codex::ttype::atomic::iterable::TIterable;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::resource::TResource;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::combiner::combine;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::union::TUnion;

const CLASSES: &[&str] = &["A", "B", "C"];

fn empty_codebase() -> CodebaseMetadata {
    CodebaseMetadata::new()
}

fn is_contained(input: &TUnion, container: &TUnion, codebase: &CodebaseMetadata) -> bool {
    let mut r = ComparisonResult::new();
    union_comparator::is_contained_by(codebase, input, container, false, false, false, &mut r)
}

fn recanonicalise(union: &TUnion) -> TUnion {
    let mut combined = combine(union.types.as_ref().to_vec(), &empty_codebase(), CombinerOptions::default());
    if combined.is_empty() {
        combined.push(TAtomic::Never);
    }
    TUnion::from_vec(combined)
}

fn arb_primitive() -> impl Strategy<Value = TAtomic> {
    prop_oneof![
        Just(TAtomic::Null),
        Just(TAtomic::Mixed(TMixed::new())),
        Just(TAtomic::Mixed(TMixed::new().with_truthiness(TMixedTruthiness::Truthy))),
        Just(TAtomic::Mixed(TMixed::new().with_truthiness(TMixedTruthiness::Falsy))),
        Just(TAtomic::Mixed(TMixed::new().with_is_non_null(true))),
        Just(TAtomic::Resource(TResource::new(None))),
        Just(TAtomic::Resource(TResource::open())),
        Just(TAtomic::Resource(TResource::closed())),
    ]
}

fn arb_bool() -> impl Strategy<Value = TAtomic> {
    prop_oneof![
        Just(TAtomic::Scalar(TScalar::r#true())),
        Just(TAtomic::Scalar(TScalar::r#false())),
        Just(TAtomic::Scalar(TScalar::bool())),
    ]
}

fn arb_integer() -> impl Strategy<Value = TAtomic> {
    prop_oneof![
        Just(TAtomic::Scalar(TScalar::int())),
        Just(TAtomic::Scalar(TScalar::Integer(TInteger::positive()))),
        Just(TAtomic::Scalar(TScalar::Integer(TInteger::negative()))),
        Just(TAtomic::Scalar(TScalar::Integer(TInteger::non_negative()))),
        Just(TAtomic::Scalar(TScalar::Integer(TInteger::non_positive()))),
        (-50i64..=50i64).prop_map(|v| TAtomic::Scalar(TScalar::Integer(TInteger::literal(v)))),
        (-50i64..=0i64, 0i64..=50i64).prop_map(|(lo, hi)| TAtomic::Scalar(TScalar::Integer(TInteger::Range(lo, hi)))),
        (-50i64..=50i64).prop_map(|f| TAtomic::Scalar(TScalar::Integer(TInteger::From(f)))),
        (-50i64..=50i64).prop_map(|t| TAtomic::Scalar(TScalar::Integer(TInteger::To(t)))),
    ]
}

fn arb_float() -> impl Strategy<Value = TAtomic> {
    prop_oneof![
        Just(TAtomic::Scalar(TScalar::float())),
        Just(TAtomic::Scalar(TScalar::Float(TFloat::UnspecifiedLiteral))),
        (-50.0f64..=50.0f64).prop_map(|v| TAtomic::Scalar(TScalar::literal_float(v))),
    ]
}

fn arb_string() -> impl Strategy<Value = TAtomic> {
    prop_oneof![
        Just(TAtomic::Scalar(TScalar::string())),
        Just(TAtomic::Scalar(TScalar::non_empty_string())),
        Just(TAtomic::Scalar(TScalar::numeric_string())),
        Just(TAtomic::Scalar(TScalar::String(TString::lowercase()))),
        Just(TAtomic::Scalar(TScalar::String(TString::uppercase()))),
        Just(TAtomic::Scalar(TScalar::String(TString::truthy()))),
        Just(TAtomic::Scalar(TScalar::String(TString::callable()))),
        Just(TAtomic::Scalar(TScalar::class_string())),
        Just(TAtomic::Scalar(TScalar::interface_string())),
        Just(TAtomic::Scalar(TScalar::enum_string())),
        Just(TAtomic::Scalar(TScalar::trait_string())),
        select(vec!["foo", "bar", "baz", "", "0", "hello"])
            .prop_map(|s| TAtomic::Scalar(TScalar::literal_string(atom(s)))),
    ]
}

fn arb_named_object() -> impl Strategy<Value = TAtomic> {
    select(CLASSES.to_vec()).prop_map(|name| TAtomic::Object(TObject::Named(TNamedObject::new(atom(name)))))
}

fn arb_array_key() -> impl Strategy<Value = TAtomic> {
    prop_oneof![
        Just(TAtomic::Scalar(TScalar::array_key())),
        Just(TAtomic::Scalar(TScalar::numeric())),
        Just(TAtomic::Scalar(TScalar::generic())),
    ]
}

fn arb_leaf_atomic() -> impl Strategy<Value = TAtomic> {
    prop_oneof![arb_primitive(), arb_bool(), arb_integer(), arb_float(), arb_string(), arb_named_object(), arb_array_key()]
}

fn arb_atomic() -> impl Strategy<Value = TAtomic> {
    arb_leaf_atomic().prop_recursive(3, 16, 4, |inner| {
        let element_union = arb_union_with(inner.clone()).boxed();
        let _ = inner;

        prop_oneof![
            (element_union.clone(), any::<bool>()).prop_map(|(elem, non_empty)| {
                let list = TList::new(Arc::new(elem));
                let list = if non_empty { list.clone_non_empty() } else { list };
                TAtomic::Array(TArray::List(list))
            }),
            (element_union.clone(), element_union.clone(), any::<bool>()).prop_map(|(k, v, non_empty)| {
                TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: None,
                    parameters: Some((Arc::new(k), Arc::new(v))),
                    non_empty,
                }))
            }),
            (element_union.clone(), element_union.clone()).prop_map(|(k, v)| {
                TAtomic::Iterable(TIterable::new(Arc::new(k), Arc::new(v)))
            }),
            (element_union.clone(), element_union).prop_map(|(p, r)| {
                let signature = TCallableSignature::new(false, false)
                    .with_parameters(vec![TCallableParameter::new(Some(Arc::new(p)), false, false, false)])
                    .with_return_type(Some(Arc::new(r)));
                TAtomic::Callable(TCallable::Signature(signature))
            }),
        ]
    })
}

fn arb_union_with(atomic: impl Strategy<Value = TAtomic>) -> impl Strategy<Value = TUnion> {
    vec(atomic, 1..=3).prop_map(TUnion::from_vec)
}

fn arb_union() -> impl Strategy<Value = TUnion> {
    vec(arb_atomic(), 1..=3).prop_map(TUnion::from_vec)
}

fn arb_subtype_pair() -> impl Strategy<Value = (TUnion, TUnion)> {
    (vec(arb_atomic(), 1..=3), vec(arb_atomic(), 0..=2)).prop_map(|(a_atoms, extras)| {
        let a = TUnion::from_vec(a_atoms.clone());

        let mut b_atoms = a_atoms;
        b_atoms.extend(extras);
        let b = TUnion::from_vec(b_atoms);

        (a, b)
    })
}

fn proptest_config() -> ProptestConfig {
    let cases: u32 = std::env::var("MAGO_PROPTEST_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(96);

    ProptestConfig { cases, max_shrink_iters: 200, ..ProptestConfig::default() }
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn subtype_is_reflexive(a in arb_union()) {
        let cb = empty_codebase();
        prop_assert!(is_contained(&a, &a, &cb), "a </: a\n  a = {:?}", a);
    }

    #[test]
    fn recanonicalise_is_idempotent(a in arb_union()) {
        let cb = empty_codebase();
        let ra = recanonicalise(&a);
        let rra = recanonicalise(&ra);

        prop_assert!(
            is_contained(&ra, &rra, &cb),
            "ra </: rra\n  a = {:?}\n  ra = {:?}\n  rra = {:?}",
            a, ra, rra,
        );
        prop_assert!(
            is_contained(&rra, &ra, &cb),
            "rra </: ra\n  a = {:?}\n  ra = {:?}\n  rra = {:?}",
            a, ra, rra,
        );
    }
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    #[ignore = "combiner over-widens lists with float refinements and absorbs void in unions"]
    fn recanonicalise_is_widening(a in arb_union()) {
        let cb = empty_codebase();
        let ra = recanonicalise(&a);

        prop_assert!(is_contained(&a, &ra, &cb), "a </: ra\n  a = {:?}\n  ra = {:?}", a, ra);
    }

    #[test]
    #[ignore = "depends on `recanonicalise_is_widening`"]
    fn subtype_preserved_through_canonical_container((a, b) in arb_subtype_pair()) {
        let cb = empty_codebase();
        prop_assume!(is_contained(&a, &b, &cb));

        let rb = recanonicalise(&b);
        prop_assert!(
            is_contained(&a, &rb, &cb),
            "a </: rb\n  a = {:?}\n  b = {:?}\n  rb = {:?}",
            a, b, rb,
        );
    }
}
