//! Internal: the structural walker that backs every public function in
//! [`crate::transform`]. One implementation, four entry-point shapes
//! ([`map`](super::map), [`flat_map`](super::flat_map),
//! [`filter_map`](super::filter_map), [`filter`](super::filter)).
//!
//! The walker is post-order. For each atom:
//!
//! 1. Recurse into every nested [`Type`] carried in the atom's payload,
//!    transforming each via [`walk`] with the same closure.
//! 2. If any nested [`Type`] changed, rebuild the atom with the new
//!    payload through the builder.
//! 3. Run the per-atom closure on the (possibly rebuilt) atom. The
//!    closure decides whether to drop, replace with one atom, or expand
//!    to many.
//!
//! Each level commits with a single union construction. Nothing is
//! rebuilt redundantly between levels.
//!
//! The per-atom closure receives the builder alongside the atom so
//! constructive transforms (e.g. literal widening in [`crate::widen`])
//! can build replacement payloads mid-walk.

use mago_allocator::Arena;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Parameter;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::conditional::ConditionalAtom;
use crate::ty::atom::payload::derived::DerivedAtom;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::intersected::IntersectedAtom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::shape::KnownProperty;
use crate::ty::atom::payload::object::shape::ObjectShapeAtom;
use crate::ty::atom::payload::reference::SymbolReferenceAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::builder::TypeBuilder;

/// What the per-atom closure returns. The walker translates this into
/// either a no-op, an in-place replacement, a 1→N expansion, or an
/// outright drop.
pub(super) enum Outcome<'arena> {
    Unchanged,
    Single(Atom<'arena>),
    Many(Vec<Atom<'arena>>),
    Drop,
}

/// Walk `ty` post-order, applying `transform` at every atom position
/// (deep through every nested-type carrier). Returns the original
/// [`Type`] when nothing changed; otherwise builds the rebuilt atom
/// list into a union once.
pub(super) fn walk<'scratch, 'arena, S, A, F>(
    ty: Type<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let mut new_atoms: Vec<Atom<'arena>> = Vec::with_capacity(ty.atoms.len());
    let mut changed = false;

    for &atom in ty.atoms {
        let rebuilt = walk_nested(atom, transform, builder);
        let target = match rebuilt {
            Some(replacement) => {
                changed = true;
                replacement
            }
            None => atom,
        };

        match transform(target, builder) {
            Outcome::Unchanged => new_atoms.push(target),
            Outcome::Single(replaced) => {
                changed = true;
                new_atoms.push(replaced);
            }
            Outcome::Many(replaced) => {
                changed = true;
                new_atoms.extend(replaced);
            }
            Outcome::Drop => {
                changed = true;
            }
        }
    }

    if !changed {
        return ty;
    }

    builder.union_of(&new_atoms)
}

/// Recurse into every nested [`Type`] carried by `atom`'s payload.
/// Returns `Some(rebuilt_atom)` when at least one nested type changed,
/// `None` otherwise.
#[inline]
fn walk_nested<'scratch, 'arena, S, A, F>(
    atom: Atom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    match atom {
        Atom::Object(payload) => walk_object(payload, transform, builder),
        Atom::List(payload) => walk_list(payload, transform, builder),
        Atom::Array(payload) => walk_keyed_array(payload, transform, builder),
        Atom::Iterable(payload) => walk_iterable(payload, transform, builder),
        Atom::ObjectShape(payload) => walk_object_shape(payload, transform, builder),
        Atom::ClassLikeString(payload) => walk_class_like_string(payload, transform, builder),
        Atom::GenericParameter(payload) => walk_generic_parameter(payload, transform, builder),
        Atom::Reference(payload) => walk_reference(payload, transform, builder),
        Atom::Conditional(payload) => walk_conditional(payload, transform, builder),
        Atom::Derived(payload) => walk_derived(payload, transform, builder),
        Atom::Callable(payload) => walk_callable(payload, transform, builder),
        Atom::Intersected(payload) => walk_intersected(payload, transform, builder),
        _ => None,
    }
}

#[inline]
fn walk_intersected<'scratch, 'arena, S, A, F>(
    payload: &'arena IntersectedAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let new_head = visit_conjunct(*payload.head, transform, builder);
    let walked_conjuncts: Vec<Atom<'arena>> =
        payload.conjuncts.iter().map(|&conjunct| visit_conjunct(conjunct, transform, builder)).collect();
    let conjuncts_changed =
        walked_conjuncts.iter().zip(payload.conjuncts.iter()).any(|(walked, original)| walked != original);
    if new_head == *payload.head && !conjuncts_changed {
        return None;
    }

    Some(builder.intersected(new_head, &walked_conjuncts))
}

/// Visit one intersection member. Conjunct positions hold exactly one
/// atom, so only a 1→1 replacement is honoured; expansion and drop
/// outcomes leave the member in place.
#[inline]
fn visit_conjunct<'scratch, 'arena, S, A, F>(
    atom: Atom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let target = walk_nested(atom, transform, builder).unwrap_or(atom);
    match transform(target, builder) {
        Outcome::Unchanged | Outcome::Many(_) | Outcome::Drop => target,
        Outcome::Single(replaced) => replaced,
    }
}

#[inline]
fn walk_object<'scratch, 'arena, S, A, F>(
    payload: &'arena ObjectAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let type_arguments = payload.type_arguments?;
    let walked: Vec<Type<'arena>> = type_arguments.iter().map(|&argument| walk(argument, transform, builder)).collect();
    if walked.iter().zip(type_arguments.iter()).all(|(new, original)| new == original) {
        return None;
    }

    let new_arguments = builder.types(&walked);

    Some(builder.object(ObjectAtom { type_arguments: Some(new_arguments), ..*payload }))
}

#[inline]
fn walk_list<'scratch, 'arena, S, A, F>(
    payload: &'arena ListAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let new_element_type = walk(payload.element_type, transform, builder);

    let new_known_elements = payload.known_elements.and_then(|entries| {
        let walked: Vec<KnownElement<'arena>> = entries
            .iter()
            .map(|entry| KnownElement { value: walk(entry.value, transform, builder), ..*entry })
            .collect();
        if walked.iter().zip(entries.iter()).all(|(new, original)| new.value == original.value) {
            None
        } else {
            Some(builder.known_elements(&walked))
        }
    });

    if new_element_type == payload.element_type && new_known_elements.is_none() {
        return None;
    }

    Some(builder.list(ListAtom {
        element_type: new_element_type,
        known_elements: new_known_elements.or(payload.known_elements),
        ..*payload
    }))
}

#[inline]
fn walk_keyed_array<'scratch, 'arena, S, A, F>(
    payload: &'arena ArrayAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let new_key = payload.key_param.map(|key_param| walk(key_param, transform, builder));
    let new_value = payload.value_param.map(|value_param| walk(value_param, transform, builder));

    let new_known_items = payload.known_items.and_then(|entries| {
        let walked: Vec<KnownItem<'arena>> =
            entries.iter().map(|entry| KnownItem { value: walk(entry.value, transform, builder), ..*entry }).collect();
        if walked.iter().zip(entries.iter()).all(|(new, original)| new.value == original.value) {
            None
        } else {
            Some(builder.known_items(&walked))
        }
    });

    let key_changed = new_key != payload.key_param;
    let value_changed = new_value != payload.value_param;
    if !key_changed && !value_changed && new_known_items.is_none() {
        return None;
    }

    Some(builder.array(ArrayAtom {
        key_param: new_key,
        value_param: new_value,
        known_items: new_known_items.or(payload.known_items),
        ..*payload
    }))
}

#[inline]
fn walk_iterable<'scratch, 'arena, S, A, F>(
    payload: &'arena IterableAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let new_key = walk(payload.key_type, transform, builder);
    let new_value = walk(payload.value_type, transform, builder);
    if new_key == payload.key_type && new_value == payload.value_type {
        return None;
    }

    Some(builder.iterable(IterableAtom { key_type: new_key, value_type: new_value }))
}

#[inline]
fn walk_object_shape<'scratch, 'arena, S, A, F>(
    payload: &'arena ObjectShapeAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let entries = payload.known_properties?;
    let walked: Vec<KnownProperty<'arena>> =
        entries.iter().map(|entry| KnownProperty { value: walk(entry.value, transform, builder), ..*entry }).collect();
    if walked.iter().zip(entries.iter()).all(|(new, original)| new.value == original.value) {
        return None;
    }

    let new_properties = builder.known_properties(&walked);

    Some(builder.object_shape(ObjectShapeAtom { known_properties: Some(new_properties), ..*payload }))
}

#[inline]
fn walk_class_like_string<'scratch, 'arena, S, A, F>(
    payload: &'arena ClassLikeStringAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let new_specifier = match payload.specifier {
        ClassLikeStringSpecifier::OfType { constraint } => {
            let walked = walk(constraint, transform, builder);
            if walked == constraint {
                return None;
            }

            ClassLikeStringSpecifier::OfType { constraint: walked }
        }
        ClassLikeStringSpecifier::Generic { constraint } => {
            let walked = walk(constraint, transform, builder);
            if walked == constraint {
                return None;
            }

            ClassLikeStringSpecifier::Generic { constraint: walked }
        }
        _ => return None,
    };

    Some(builder.class_like_string(ClassLikeStringAtom { specifier: new_specifier, ..*payload }))
}

#[inline]
fn walk_generic_parameter<'scratch, 'arena, S, A, F>(
    payload: &'arena GenericParameterAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let walked = walk(payload.constraint, transform, builder);
    if walked == payload.constraint {
        return None;
    }

    Some(builder.generic_parameter(GenericParameterAtom { constraint: walked, ..*payload }))
}

#[inline]
fn walk_reference<'scratch, 'arena, S, A, F>(
    payload: &'arena SymbolReferenceAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let type_arguments = payload.type_arguments?;
    let walked: Vec<Type<'arena>> = type_arguments.iter().map(|&argument| walk(argument, transform, builder)).collect();
    if walked.iter().zip(type_arguments.iter()).all(|(new, original)| new == original) {
        return None;
    }

    let new_arguments = builder.types(&walked);

    Some(builder.reference(SymbolReferenceAtom { type_arguments: Some(new_arguments), ..*payload }))
}

#[inline]
fn walk_conditional<'scratch, 'arena, S, A, F>(
    payload: &'arena ConditionalAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let subject = walk(payload.subject, transform, builder);
    let target = walk(payload.target, transform, builder);
    let then = walk(payload.then, transform, builder);
    let otherwise = walk(payload.otherwise, transform, builder);
    if subject == payload.subject && target == payload.target && then == payload.then && otherwise == payload.otherwise
    {
        return None;
    }

    Some(builder.conditional(ConditionalAtom { subject, target, then, otherwise, negated: payload.negated }))
}

#[inline]
fn walk_derived<'scratch, 'arena, S, A, F>(
    payload: &'arena DerivedAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let walked = match *payload {
        DerivedAtom::KeyOf(target) => {
            let new_target = walk(target, transform, builder);
            if new_target == target {
                return None;
            }

            DerivedAtom::KeyOf(new_target)
        }
        DerivedAtom::ValueOf(target) => {
            let new_target = walk(target, transform, builder);
            if new_target == target {
                return None;
            }

            DerivedAtom::ValueOf(new_target)
        }
        DerivedAtom::IndexAccess { target, index } => {
            let new_target = walk(target, transform, builder);
            let new_index = walk(index, transform, builder);
            if new_target == target && new_index == index {
                return None;
            }

            DerivedAtom::IndexAccess { target: new_target, index: new_index }
        }
        DerivedAtom::PropertiesOf { target, visibility } => {
            let new_target = walk(target, transform, builder);
            if new_target == target {
                return None;
            }

            DerivedAtom::PropertiesOf { target: new_target, visibility }
        }
        DerivedAtom::IntMask(members) => {
            let walked_members: Vec<Type<'arena>> =
                members.iter().map(|&member| walk(member, transform, builder)).collect();
            if walked_members.iter().zip(members.iter()).all(|(new, original)| new == original) {
                return None;
            }

            DerivedAtom::IntMask(builder.types(&walked_members))
        }
        DerivedAtom::IntMaskOf(target) => {
            let new_target = walk(target, transform, builder);
            if new_target == target {
                return None;
            }

            DerivedAtom::IntMaskOf(new_target)
        }
        DerivedAtom::TemplateType { object, class_name, template_name } => {
            let new_object = walk(object, transform, builder);
            let new_class_name = walk(class_name, transform, builder);
            let new_template_name = walk(template_name, transform, builder);
            if new_object == object && new_class_name == class_name && new_template_name == template_name {
                return None;
            }

            DerivedAtom::TemplateType {
                object: new_object,
                class_name: new_class_name,
                template_name: new_template_name,
            }
        }
        DerivedAtom::New(target) => {
            let new_target = walk(target, transform, builder);
            if new_target == target {
                return None;
            }

            DerivedAtom::New(new_target)
        }
    };

    Some(builder.derived(walked))
}

#[inline]
fn walk_callable<'scratch, 'arena, S, A, F>(
    payload: CallableAtom<'arena>,
    transform: &mut F,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
    F: FnMut(Atom<'arena>, &mut TypeBuilder<'scratch, 'arena, S, A>) -> Outcome<'arena>,
{
    let (CallableAtom::Signature(signature) | CallableAtom::Closure(signature)) = payload else {
        return None;
    };

    let new_return_type = walk(signature.return_type, transform, builder);
    let new_throws = signature.throws.map(|throws| walk(throws, transform, builder));
    let new_parameters = signature.parameters.and_then(|parameters| {
        let walked: Vec<Parameter<'arena>> = parameters
            .iter()
            .map(|parameter| Parameter { r#type: walk(parameter.r#type, transform, builder), ..*parameter })
            .collect();
        if walked.iter().zip(parameters.iter()).all(|(new, original)| new.r#type == original.r#type) {
            None
        } else {
            Some(builder.parameters(&walked))
        }
    });

    let return_type_changed = new_return_type != signature.return_type;
    let throws_changed = new_throws != signature.throws;
    if !return_type_changed && !throws_changed && new_parameters.is_none() {
        return None;
    }

    let new_signature = builder.signature(Signature {
        return_type: new_return_type,
        throws: new_throws.or(signature.throws),
        parameters: new_parameters.or(signature.parameters),
        ..*signature
    });

    let rebuilt = match payload {
        CallableAtom::Signature(_) => CallableAtom::Signature(new_signature),
        CallableAtom::Closure(_) => CallableAtom::Closure(new_signature),
        _ => return None,
    };

    Some(Atom::Callable(rebuilt))
}
