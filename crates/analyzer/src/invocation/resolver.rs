use mago_allocator::Arena;
use std::borrow::Cow;

use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_never;
use mago_codex::ttype::template::TemplateBound;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::union::TUnion;
use mago_word::WordMap;
use mago_word::empty_word;

use crate::context::Context;
use crate::invocation::Invocation;

/// Resolves a type resulting from an invocation.
pub fn resolve_invocation_type<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &TemplateResult,
    parameters: &WordMap<TUnion>,
    invocation_type: TUnion,
) -> TUnion
where
    A: Arena,
{
    let mut template_result = Cow::Borrowed(template_result);

    'populate_templates: {
        if let Some(function_like_identifier) = invocation.target.get_function_like_identifier() {
            let generic_parent = match function_like_identifier {
                FunctionLikeIdentifier::Method(class, method) => GenericParent::FunctionLike((*class, *method)),
                FunctionLikeIdentifier::Function(function) => GenericParent::FunctionLike((empty_word(), *function)),
                _ => {
                    break 'populate_templates;
                }
            };

            let method_templates = invocation.target.get_template_types();

            let all_template_names: Vec<_> = method_templates
                .map(|m| m.keys().copied().collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .chain(template_result.template_types.keys().copied())
                .collect();

            for template_name in all_template_names {
                let has_bound_for_method = template_result
                    .lower_bounds
                    .get(&template_name)
                    .and_then(|bounds| bounds.get(&generic_parent))
                    .is_some_and(|bounds| !bounds.is_empty());

                let method_parents: Vec<_> = method_templates
                    .and_then(|m| m.get(&template_name))
                    .map(|t| vec![&t.defining_entity])
                    .unwrap_or_default();

                let result_parents: Vec<_> = template_result
                    .template_types
                    .get(&template_name)
                    .map(|v| v.iter().map(|t| &t.defining_entity).collect())
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

fn resolve_union<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &TemplateResult,
    parameters: &WordMap<TUnion>,
    union_to_resolve: TUnion,
) -> TUnion
where
    A: Arena,
{
    let mut resulting_union = union_to_resolve;
    let mut resulting_atomics = Vec::with_capacity(resulting_union.types.len());
    for atomic_to_resolve in resulting_union.types.into_owned() {
        let return_atomics = resolve_atomic(context, invocation, template_result, parameters, atomic_to_resolve);
        resulting_atomics.extend(return_atomics);
    }

    resulting_union.types = Cow::Owned(resulting_atomics);

    if !template_result.lower_bounds.is_empty() || resulting_union.has_template_types() {
        // Replace templates first so derived types (e.g. `template-type<T, ...>`)
        // see concrete object/target types before expansion runs their resolution logic.
        // Running expansion first would eagerly walk the abstract `T`'s constraint and lose the substitution site.
        resulting_union = inferred_type_replacer::replace(&resulting_union, template_result, context.codebase);

        expander::expand_union(
            context.codebase,
            &mut resulting_union,
            &TypeExpansionOptions { expand_templates: false, ..Default::default() },
        );
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
            if declaring_class_name != method_context.class_like_metadata.name
                && let Some(declaring_class_meta) = context.codebase.get_class_like(declaring_class_name.as_bytes())
                && declaring_class_meta.kind.is_trait()
            {
                let mut new_atomics = Vec::with_capacity(resulting_union.types.len());
                for atomic in resulting_union.types.as_ref() {
                    match atomic {
                        TAtomic::Object(TObject::Named(named_object))
                            if named_object.name.as_bytes().eq_ignore_ascii_case(declaring_class_name.as_bytes()) =>
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
        static_class_type = StaticClassType::default();
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

fn resolve_atomic<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &TemplateResult,
    parameters: &WordMap<TUnion>,
    atomic_to_resolve: TAtomic,
) -> Vec<TAtomic>
where
    A: Arena,
{
    if let TAtomic::Variable(variable) = atomic_to_resolve {
        if variable.as_bytes().eq_ignore_ascii_case(b"$this")
            && let Some(method_context) = invocation.target.get_method_context()
            && let StaticClassType::Object(this_type) = &method_context.class_type
        {
            return vec![TAtomic::Object(this_type.clone())];
        }

        return parameters
            .get(&variable)
            .map(|argument_type| {
                inferred_type_replacer::replace(argument_type, template_result, context.codebase).types.into_owned()
            })
            .unwrap_or_else(|| vec![TAtomic::Mixed(TMixed::new())]);
    }

    let TAtomic::Conditional(conditional) = atomic_to_resolve else {
        return vec![atomic_to_resolve];
    };

    let subject = resolve_union(context, invocation, template_result, parameters, (*conditional.subject).clone());
    let target = resolve_union(context, invocation, template_result, parameters, (*conditional.target).clone());
    let then_type = resolve_union(context, invocation, template_result, parameters, (*conditional.then).clone());
    let otherwise_type =
        resolve_union(context, invocation, template_result, parameters, (*conditional.otherwise).clone());
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
            true,
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
            return if negated { then_type.types.into_owned() } else { otherwise_type.types.into_owned() };
        }

        if subject_is_contained {
            return if negated { otherwise_type.types.into_owned() } else { then_type.types.into_owned() };
        }
    }

    add_union_type(then_type, &otherwise_type, context.codebase, CombinerOptions::default()).types.into_owned()
}
