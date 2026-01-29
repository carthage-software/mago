use std::borrow::Cow;

use either::Either;

use mago_atom::AtomMap;
use mago_atom::empty_atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::template::TemplateBound;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::union::TUnion;

use crate::context::Context;
use crate::invocation::Invocation;

/// Resolves a type resulting from an invocation.
pub fn resolve_invocation_type<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
    invocation_type: TUnion,
) -> TUnion {
    let mut template_result = Cow::Borrowed(template_result);

    'populate_templates: {
        if let Some(function_like_identifier) = invocation.target.get_function_like_identifier() {
            let generic_parent = match function_like_identifier {
                FunctionLikeIdentifier::Method(class, method) => GenericParent::FunctionLike((*class, *method)),
                FunctionLikeIdentifier::Function(function) => GenericParent::FunctionLike((empty_atom(), *function)),
                _ => {
                    break 'populate_templates;
                }
            };

            let method_templates = invocation.target.get_template_types().unwrap_or(&[]);

            let all_template_names: Vec<_> = method_templates
                .iter()
                .map(|(name, _)| *name)
                .chain(template_result.template_types.keys().copied())
                .collect();

            for template_name in all_template_names {
                let has_bound_for_method = template_result
                    .lower_bounds
                    .get(&template_name)
                    .and_then(|bounds| bounds.get(&generic_parent))
                    .is_some_and(|bounds| !bounds.is_empty());

                let method_parents: Vec<_> = method_templates
                    .iter()
                    .filter(|(name, _)| name == &template_name)
                    .flat_map(|(_, constraints)| constraints.iter().map(|(parent, _)| parent))
                    .collect();

                let result_parents: Vec<_> = template_result
                    .template_types
                    .get(&template_name)
                    .map(|v| v.iter().map(|(parent, _)| parent).collect())
                    .unwrap_or_default();

                let has_bound_for_template_parent =
                    method_parents.iter().chain(result_parents.iter()).any(|constraint_parent| {
                        template_result
                            .lower_bounds
                            .get(&template_name)
                            .and_then(|bounds| bounds.get(*constraint_parent))
                            .is_some_and(|bounds| !bounds.is_empty())
                    });

                if !has_bound_for_method && !has_bound_for_template_parent {
                    let mut owned_template_result = template_result.into_owned();

                    owned_template_result
                        .lower_bounds
                        .entry(template_name)
                        .or_default()
                        .insert(generic_parent, vec![TemplateBound::new(get_never(), 1, None, None)]);

                    template_result = Cow::Owned(owned_template_result);
                }
            }
        }
    }

    resolve_union(context, invocation, &template_result, parameters, invocation_type)
}

fn resolve_union<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
    union_to_resolve: TUnion,
) -> TUnion {
    let mut resulting_union = union_to_resolve;
    let mut resulting_atomics = Vec::with_capacity(resulting_union.types.len());
    for atomic_to_resolve in resulting_union.types.into_owned() {
        let return_atomic = resolve_atomic(context, invocation, template_result, parameters, atomic_to_resolve);

        match return_atomic {
            Either::Left(atomic) => resulting_atomics.push(atomic),
            Either::Right(union) => {
                resulting_atomics.extend(union.types.into_owned());
            }
        }
    }

    resulting_union.types = Cow::Owned(resulting_atomics);

    if !template_result.lower_bounds.is_empty() || resulting_union.has_template_types() {
        expander::expand_union(
            context.codebase,
            &mut resulting_union,
            &TypeExpansionOptions { expand_templates: false, ..Default::default() },
        );

        resulting_union = inferred_type_replacer::replace(&resulting_union, template_result, context.codebase);
    }

    let static_class_type;
    let parent_class;
    let self_class;
    let function_is_final;

    if let Some(method_context) = invocation.target.get_method_context() {
        static_class_type = method_context.class_type.clone();
        parent_class = method_context.class_like_metadata.direct_parent_class;
        self_class = Some(method_context.class_like_metadata.name);
        function_is_final = invocation
            .target
            .get_function_like_metadata()
            .and_then(|metadata| metadata.method_metadata.as_ref())
            .is_some_and(|metadata| metadata.is_final);

        if let Some(declaring_method_id) = &method_context.declaring_method_id {
            let declaring_class_name = declaring_method_id.get_class_name();
            if *declaring_class_name != method_context.class_like_metadata.name
                && let Some(declaring_class_meta) = context.codebase.get_class_like(declaring_class_name)
                && declaring_class_meta.kind.is_trait()
            {
                let mut new_atomics = Vec::with_capacity(resulting_union.types.len());
                for atomic in resulting_union.types.as_ref() {
                    match atomic {
                        TAtomic::Object(TObject::Named(named_object))
                            if named_object.name.eq_ignore_ascii_case(declaring_class_name) =>
                        {
                            let mut new_object = named_object.clone();
                            new_object.name = method_context.class_like_metadata.name;
                            new_atomics.push(TAtomic::Object(TObject::Named(new_object)));
                        }
                        _ => new_atomics.push(atomic.clone()),
                    }
                }

                resulting_union.types = Cow::Owned(new_atomics);
            }
        }
    } else {
        static_class_type = Default::default();
        parent_class = None;
        self_class = None;
        function_is_final = false;
    }

    expander::expand_union(
        context.codebase,
        &mut resulting_union,
        &TypeExpansionOptions {
            expand_templates: false,
            expand_generic: true,
            self_class,
            static_class_type,
            parent_class,
            function_is_final,
            ..Default::default()
        },
    );

    resulting_union
}

fn resolve_atomic<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
    atomic_to_resolve: TAtomic,
) -> Either<TAtomic, TUnion> {
    if let TAtomic::Variable(variable) = atomic_to_resolve {
        if variable.eq_ignore_ascii_case("$this")
            && let Some(method_context) = invocation.target.get_method_context()
            && let StaticClassType::Object(this_type) = &method_context.class_type
        {
            return Either::Left(TAtomic::Object(this_type.clone()));
        }

        return parameters.get(&variable).map_or(Either::Right(get_mixed()), |argument_type| {
            Either::Right(inferred_type_replacer::replace(argument_type, template_result, context.codebase))
        });
    }

    let TAtomic::Conditional(conditional) = atomic_to_resolve else {
        return Either::Left(atomic_to_resolve);
    };

    let subject = resolve_union(context, invocation, template_result, parameters, *conditional.subject);
    let target = resolve_union(context, invocation, template_result, parameters, *conditional.target);
    let then_type = resolve_union(context, invocation, template_result, parameters, *conditional.then);
    let otherwise_type = resolve_union(context, invocation, template_result, parameters, *conditional.otherwise);
    let negated = conditional.negated;

    let subject = inferred_type_replacer::replace(&subject, template_result, context.codebase);
    let target = inferred_type_replacer::replace(&target, template_result, context.codebase);

    if !subject.is_never() {
        let mut comparison_result = ComparisonResult::new();
        let subject_is_contained = union_comparator::is_contained_by(
            context.codebase,
            &subject,
            &target,
            false,
            false,
            false,
            &mut comparison_result,
        );

        let are_int_float_disjoint = if target.is_single() && subject.is_single() {
            matches!(
                (subject.effective_int_or_float(), target.effective_int_or_float()),
                (Some(true), Some(false)) | (Some(false), Some(true))
            )
        } else {
            false
        };

        let are_disjoint = are_int_float_disjoint
            || !union_comparator::can_expression_types_be_identical(context.codebase, &subject, &target, false, false);

        if are_disjoint {
            return if negated { Either::Right(then_type) } else { Either::Right(otherwise_type) };
        }

        if subject_is_contained {
            return if negated { Either::Right(otherwise_type) } else { Either::Right(then_type) };
        }
    }

    Either::Right(add_union_type(then_type, &otherwise_type, context.codebase, CombinerOptions::default()))
}
