use std::collections::BTreeMap;
use std::sync::Arc;

use mago_word::word;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::misc::GenericParent;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::atomic::callable::{TCallable, TCallableSignature};
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::{TClassLikeString, TClassLikeStringKind};
use crate::ttype::get_literal_string;

use super::*;

fn named(name: &str) -> TUnion {
    TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(word(name)))))
}

fn display(ty: &TUnion) -> String {
    NameDisplay::for_types(&CodebaseMetadata::new(), std::iter::once(ty)).union(ty)
}

#[test]
fn type_names_do_not_rewrite_shape_keys_or_literal_values() {
    let shape = |name: &str| {
        TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(word("a'user")), (false, get_literal_string(word("a'user")))),
            (ArrayKey::String(word("user")), (false, named(name))),
        ])))))
    };
    let source = shape("Domain\\User");
    let before = source.get_id();
    assert_eq!(display(&source), BytesDisplay(shape("User").get_id().as_bytes()).to_string());
    assert_eq!(source.get_id(), before, "hover must not change shared analysis types");
}

#[test]
fn type_names_keep_template_names_and_origins() {
    let ty = TUnion::from_atomic(TAtomic::GenericParameter(TGenericParameter::new(
        word("user"),
        Arc::new(named("Domain\\User")),
        GenericParent::ClassLike(word("Domain\\Owner")),
    )));
    assert_eq!(display(&ty), "('user.Domain\\Owner extends User)");
}

#[test]
fn type_names_keep_callable_aliases_but_shorten_signature_types() {
    let alias = TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Method(word("Domain\\User"), word("user"))));
    let signature = TCallableSignature::new(false, true)
        .with_parameters(vec![TCallableParameter::new(Some(Arc::new(named("Domain\\User"))), false, false, false)])
        .with_return_type(Some(Arc::new(TUnion::from_atomic(alias))));
    let ty = TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(signature)));
    assert_eq!(display(&ty), "(closure(User): Closure<Domain\\User::user>(...))");
}

#[test]
fn type_names_keep_class_string_values_but_shorten_constraints() {
    let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
        word("Domain\\Box"),
        Some(vec![
            TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Literal {
                value: word("Domain\\User"),
            }))),
            TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType {
                kind: TClassLikeStringKind::Class,
                constraint: Arc::new(TAtomic::Object(TObject::Named(TNamedObject::new(word("Domain\\User"))))),
            }))),
        ]),
    ))));
    assert_eq!(display(&ty), "Box<class-string('Domain\\User'), class-string<User>>");
}

#[test]
fn type_names_keep_reference_collisions_qualified() {
    let first = named("Left\\User");
    let second = TUnion::from_atomic(TAtomic::Reference(TReference::new_member(
        word("Right\\User"),
        TReferenceMemberSelector::Identifier(word("user")),
    )));
    let codebase = CodebaseMetadata::new();
    let names = NameDisplay::for_types(&codebase, [&first, &second].into_iter());
    assert_eq!(names.union(&first), "\\Left\\User");
    assert_eq!(names.union(&second), "unknown-ref(\\Right\\User::user)");
}

#[test]
fn type_names_shorten_intersections_without_changing_member_names() {
    let mut object = TAtomic::Object(TObject::Named(TNamedObject::new(word("Domain\\User"))));
    object.add_intersection_type(TAtomic::Object(TObject::new_enum_case(word("Domain\\State"), word("user"))));
    assert_eq!(display(&TUnion::from_atomic(object)), "User&State::user");
}

#[test]
fn enum_display_keeps_cases_collisions_and_nested_literal_values() {
    let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
        word("Domain\\Box"),
        Some(vec![
            TUnion::from_atomic(TAtomic::Object(TObject::new_enum(word("Left\\State")))),
            TUnion::from_atomic(TAtomic::Object(TObject::new_enum_case(word("Right\\State"), word("Open")))),
            get_literal_string(word("enum(Left\\State)")),
        ]),
    ))));
    let before = ty.clone();
    assert_eq!(display(&ty), "Box<\\Left\\State, \\Right\\State::Open, string('enum(Left\\State)')>");
    assert_eq!(ty, before, "hover must not change shared analysis types");
}

#[test]
fn owned_rendering_keeps_compound_type_spelling() {
    use crate::ttype::atomic::array::list::TList;
    use crate::ttype::atomic::conditional::TConditional;
    use crate::ttype::atomic::derived::{
        TDerived, index_access::TIndexAccess, int_mask::TIntMask, int_mask_of::TIntMaskOf,
        intersection::TDerivedIntersection, key_of::TKeyOf, new::TNew, properties_of::TPropertiesOf,
        template_type::TTemplateType, value_of::TValueOf,
    };
    use crate::ttype::atomic::iterable::TIterable;
    use crate::ttype::{get_arraykey, get_int, get_mixed};

    let value = Arc::new(named("User"));
    let key = Arc::new(get_int());
    let mut derived_intersection = TDerivedIntersection::new((*value).clone());
    derived_intersection.add_intersection_type(TAtomic::Object(TObject::new_named(word("Other"))));
    let derived = [
        TDerived::KeyOf(TKeyOf::new(Arc::clone(&value))),
        TDerived::ValueOf(TValueOf::new(Arc::clone(&value))),
        TDerived::IntMaskOf(TIntMaskOf::new(Arc::clone(&value))),
        TDerived::New(TNew::new(Arc::clone(&value))),
        TDerived::PropertiesOf(TPropertiesOf::private(Arc::clone(&value))),
        TDerived::IntMask(TIntMask::new(vec![(*key).clone(), (*value).clone()])),
        TDerived::IndexAccess(TIndexAccess::new((*value).clone(), (*key).clone())),
        TDerived::TemplateType(TTemplateType::new(Arc::clone(&value), Arc::clone(&value), Arc::clone(&key))),
        TDerived::Intersection(derived_intersection),
    ];
    let shape = BTreeMap::from([
        (ArrayKey::Integer(2), (false, (*value).clone())),
        (ArrayKey::String(word(b"a\xff'")), (true, get_literal_string(word(b"z\xff")))),
    ]);
    let mut list = TList::from_known_elements(BTreeMap::from([(2, (true, (*value).clone()))]));
    list.element_type = Arc::clone(&key);
    let mut iterable = TIterable::new(Arc::clone(&key), Arc::clone(&value));
    iterable.intersection_types = Some(vec![TAtomic::Object(TObject::new_has_method(word("get")))]);
    let samples = [
        TAtomic::Array(TArray::List(list)),
        TAtomic::Array(TArray::List(TList::new_non_empty(Arc::clone(&value)))),
        TAtomic::Array(TArray::Keyed(
            TKeyedArray::new().with_known_items(shape).with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())),
        )),
        TAtomic::Array(TArray::Keyed(
            TKeyedArray::new_with_parameters(Arc::clone(&key), Arc::clone(&value)).with_non_empty(true),
        )),
        TAtomic::Iterable(iterable),
        TAtomic::Conditional(TConditional::new(
            Arc::clone(&key),
            Arc::clone(&value),
            Arc::clone(&value),
            Arc::clone(&key),
            true,
        )),
        TAtomic::Object(TObject::Named(TNamedObject::new_this(word("User")))),
        TAtomic::Object(TObject::Named(TNamedObject::new_static(word("User")))),
        TAtomic::Object(TObject::new_with_properties(false, BTreeMap::from([(word("id"), (true, (*value).clone()))]))),
    ];

    for atomic in samples.into_iter().chain(derived.into_iter().map(TAtomic::Derived)) {
        let ty = TUnion::from_vec(vec![TAtomic::Null, atomic.clone(), atomic]);
        assert_eq!(display(&ty), BytesDisplay(ty.get_id().as_bytes()).to_string());
    }
}

#[test]
fn owned_rendering_keeps_literal_bytes_and_numbers() {
    use crate::ttype::atomic::scalar::int::TInteger;
    use crate::ttype::{get_literal_float, get_literal_int};

    let values = [
        get_literal_string(word(b"\xff'|\\xFF")),
        get_literal_int(i64::MIN),
        get_literal_float(1e-5),
        get_literal_float(1e20),
        get_literal_float(f64::INFINITY),
        TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::Range(-30, 40)))),
        TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::From(4)))),
        TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::To(-4)))),
    ];
    for ty in values {
        assert_eq!(display(&ty), BytesDisplay(ty.get_id().as_bytes()).to_string());
    }
}
