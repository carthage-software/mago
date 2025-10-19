use std::borrow::Cow;
use std::collections::BTreeMap;

use mago_atom::Atom;
use mago_atom::atom;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::object::TObject;
use crate::ttype::template::TemplateResult;
use crate::ttype::union::TUnion;

pub fn cast_atomic_to_callable<'a>(
    atomic: &'a TAtomic,
    codebase: &CodebaseMetadata,
    mut template_result: Option<&mut TemplateResult>,
) -> Option<Cow<'a, TCallable>> {
    if let Some(intersections) = atomic.get_intersection_types() {
        for intersection in intersections {
            if let Some(callable) = cast_atomic_to_callable(intersection, codebase, template_result.as_deref_mut()) {
                return Some(callable);
            }
        }
    }

    if let TAtomic::GenericParameter(generic_parameter) = atomic
        && generic_parameter.constraint.is_single()
    {
        return cast_atomic_to_callable(generic_parameter.constraint.get_single(), codebase, template_result);
    }

    if let TAtomic::Callable(callable) = atomic {
        return Some(Cow::Borrowed(callable));
    }

    if let Some(literal_string) = atomic.get_literal_string_value() {
        // Check if this is a static method callable in the format "ClassName::methodName"
        if let Some((class_part, method_part)) = literal_string.split_once("::") {
            return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Method(
                atom(class_part),
                atom(method_part),
            ))));
        }

        return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Function(atom(literal_string)))));
    }

    if let TAtomic::Object(TObject::Named(named_object)) = atomic {
        let method_identifier = MethodIdentifier::new(named_object.get_name(), atom("__invoke"));
        let method_identifier = codebase.get_declaring_method_identifier(&method_identifier);
        if codebase.method_exists(method_identifier.get_class_name(), method_identifier.get_method_name()) {
            populate_template_result(
                template_result.as_deref_mut(),
                codebase,
                named_object.get_name(),
                named_object.get_type_parameters(),
            );

            return Some(Cow::Owned(TCallable::Alias(method_identifier.into())));
        }
    }

    if let TAtomic::Array(TArray::List(TList { known_elements: Some(known_elements), .. })) = atomic {
        return handle_array_callable(known_elements, codebase, template_result);
    }

    None
}

fn handle_array_callable(
    known_elements: &BTreeMap<usize, (bool, TUnion)>,
    codebase: &CodebaseMetadata,
    mut template_result: Option<&mut TemplateResult>,
) -> Option<Cow<'static, TCallable>> {
    let (optional, class_or_object) = known_elements.get(&0)?;
    if *optional || !class_or_object.is_single() {
        return None;
    }

    let (optional, method) = known_elements.get(&1)?;
    if *optional {
        return None;
    }

    let class_or_object = class_or_object.get_single();
    let method_name = atom(method.get_single_literal_string_value()?);

    // Check if the first element is a literal string (e.g., 'ClassName')
    if let Some(class_name) = class_or_object.get_literal_string_value() {
        return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Method(atom(class_name), method_name))));
    }

    // Check if the first element is a class-string literal (e.g., ClassName::class)
    if let Some(class_name) = class_or_object.get_class_string_value() {
        return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Method(class_name, method_name))));
    }

    // Collect all atomics to check - handle intersections and generic parameters
    let atomics_to_check = collect_atomics_to_check(class_or_object);

    for atomic in atomics_to_check {
        if let Some(callable) = try_object_method(atomic, method_name, codebase, template_result.as_deref_mut()) {
            return Some(callable);
        }
    }

    None
}

fn collect_atomics_to_check(atomic: &TAtomic) -> Vec<&TAtomic> {
    let mut atomics = vec![atomic];

    if let Some(intersections) = atomic.get_intersection_types() {
        atomics.extend(intersections.iter().flat_map(|intersection| collect_atomics_to_check(intersection)));
    }

    if let TAtomic::GenericParameter(generic_parameter) = atomic
        && generic_parameter.constraint.is_single()
    {
        atomics.extend(collect_atomics_to_check(generic_parameter.constraint.get_single()));
    }

    atomics
}

fn try_object_method(
    atomic: &TAtomic,
    method_name: Atom,
    codebase: &CodebaseMetadata,
    template_result: Option<&mut TemplateResult>,
) -> Option<Cow<'static, TCallable>> {
    let (object_name, type_parameters) = match atomic {
        TAtomic::Object(TObject::Named(named_object)) => (named_object.get_name(), named_object.get_type_parameters()),
        TAtomic::Object(TObject::Enum(enum_object)) => (enum_object.get_name(), None),
        _ => return None,
    };

    let method_identifier = MethodIdentifier::new(object_name, method_name);
    let method_identifier = codebase.get_declaring_method_identifier(&method_identifier);

    if codebase.method_exists(method_identifier.get_class_name(), method_identifier.get_method_name()) {
        populate_template_result(template_result, codebase, object_name, type_parameters);
        return Some(Cow::Owned(TCallable::Alias(method_identifier.into())));
    }

    None
}

fn populate_template_result(
    template_result: Option<&mut TemplateResult>,
    codebase: &CodebaseMetadata,
    object_name: Atom,
    type_parameters: Option<&[TUnion]>,
) {
    let Some(template_result) = template_result else {
        return;
    };

    let Some(class_metadata) = codebase.get_class_like(&object_name) else {
        return;
    };

    let Some(type_parameters) = type_parameters else {
        return;
    };

    for (index, parameter) in type_parameters.iter().enumerate() {
        let Some(template_name) = class_metadata.get_template_name_for_index(index) else {
            continue;
        };

        template_result
            .template_types
            .entry(template_name)
            .or_default()
            .push((GenericParent::ClassLike(object_name), parameter.clone()));
    }
}
