use std::borrow::Cow;

use mago_atom::atom;

use crate::get_class_like;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::object::TObject;
use crate::ttype::template::TemplateResult;

pub fn cast_atomic_to_callable<'a>(
    atomic: &'a TAtomic,
    codebase: &CodebaseMetadata,
    template_result: Option<&mut TemplateResult>,
) -> Option<Cow<'a, TCallable>> {
    if let TAtomic::Callable(callable) = atomic {
        return Some(Cow::Borrowed(callable));
    }

    if let Some(literal_string) = atomic.get_literal_string_value() {
        return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Function(atom(literal_string)))));
    }

    if let TAtomic::Object(TObject::Named(named_object)) = atomic {
        if let Some(template_result) = template_result
            && let Some(class_metadata) = get_class_like(codebase, named_object.get_name_ref())
        {
            for (index, parameter) in named_object.get_type_parameters().unwrap_or_default().iter().enumerate() {
                let Some(template_name) = class_metadata.get_template_name_for_index(index) else {
                    continue;
                };

                template_result
                    .template_types
                    .entry(template_name)
                    .or_default()
                    .push((GenericParent::ClassLike(named_object.get_name()), parameter.clone()));
            }
        }

        return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Method(
            named_object.get_name(),
            atom("__invoke"),
        ))));
    }

    if let TAtomic::Array(TArray::List(TList { known_elements: Some(known_elements), .. })) = atomic {
        let (optional, class_or_object) = known_elements.get(&0)?;
        if *optional {
            return None;
        }

        if !class_or_object.is_single() {
            return None;
        }

        let (optional, method) = known_elements.get(&1)?;
        if *optional {
            return None;
        }

        let class_or_object = class_or_object.get_single();
        let method_name = atom(method.get_single_literal_string_value()?);
        if let Some(class_name) = class_or_object.get_literal_string_value() {
            return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Method(atom(class_name), method_name))));
        }

        if let TAtomic::Object(TObject::Named(named_object)) = class_or_object {
            if let Some(template_result) = template_result
                && let Some(class_metadata) = get_class_like(codebase, named_object.get_name_ref())
            {
                for (index, parameter) in named_object.get_type_parameters().unwrap_or_default().iter().enumerate() {
                    let Some(template_name) = class_metadata.get_template_name_for_index(index) else {
                        continue;
                    };

                    template_result
                        .template_types
                        .entry(template_name)
                        .or_default()
                        .push((GenericParent::ClassLike(named_object.get_name()), parameter.clone()));
                }
            }

            return Some(Cow::Owned(TCallable::Alias(FunctionLikeIdentifier::Method(
                named_object.get_name(),
                method_name,
            ))));
        }
    }

    None
}
