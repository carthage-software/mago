//! Editor targets for receivers skipped by property memoization.

use mago_allocator::Arena;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::TypeRef;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_span::HasSpan;
use mago_syntax::cst::ClassConstantAccess;
use mago_syntax::cst::ClassLikeConstantSelector;
use mago_syntax::cst::ClassLikeMemberSelector;
use mago_syntax::cst::Expression;
use mago_syntax::cst::MethodCall;
use mago_word::word;

use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::editor::MemberKind;
use crate::artifacts::editor::MemberReference;
use crate::artifacts::editor::MethodReference;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::visibility::is_method_visible_by_id;

pub(super) fn constant<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    block: &BlockContext<'ctx>,
    access: &ClassConstantAccess<'arena>,
) -> Option<(MemberReference, TUnion)>
where
    A: Arena,
{
    let class = match access.class.unparenthesized() {
        Expression::Identifier(name) => word(context.resolved_names.get(name)),
        Expression::Self_(_) | Expression::Static(_) => block.scope.get_class_like_name()?,
        Expression::Parent(_) => block.scope.get_class_like()?.direct_parent_class?,
        _ => return None,
    };

    let ClassLikeConstantSelector::Identifier(name) = &access.constant else { return None };
    let name = word(name.value);
    let class = context.codebase.get_class_like_by_name(class)?;
    let mut reference = MemberReference::property(
        block.scope.get_reference_origin(),
        access.class.span(),
        access.constant.span(),
        name,
    );

    reference.lookup_class = Some(class.name);
    reference.static_access = true;
    let mut result = if let Some(constant) = class.constants.get(&name) {
        reference.kind = MemberKind::ClassConstant;
        reference.declaration = Some(constant.span.subspan(0, name.len() as u32));
        if let Some(metadata) = &constant.type_metadata
            && metadata.from_docblock
        {
            metadata.type_union.clone()
        } else {
            constant
                .inferred_type
                .clone()
                .map(wrap_atomic)
                .or_else(|| constant.type_metadata.as_ref().map(|metadata| metadata.type_union.clone()))
                .unwrap_or_else(get_mixed)
        }
    } else {
        let case = class.enum_cases.get(&name)?;
        reference.kind = MemberKind::EnumCase;
        reference.declaration = Some(case.name_span);
        TUnion::from_atomic(TAtomic::Object(TObject::Enum(TEnum::new_case(class.original_name, name))))
    };

    expand_union(
        context.codebase,
        &mut result,
        &TypeExpansionOptions {
            self_class: Some(class.name),
            static_class_type: StaticClassType::Name(class.name),
            function_is_final: class.flags.is_final(),
            ..TypeExpansionOptions::default()
        },
    );

    Some((reference, result))
}

pub(super) fn methods<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    block: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    call: &MethodCall<'arena>,
) -> Vec<MethodReference>
where
    A: Arena,
{
    let ClassLikeMemberSelector::Identifier(name) = &call.method else { return Vec::new() };
    let Some(receiver) = super::saved_receiver_type(context, block, artifacts, call.object) else { return Vec::new() };
    let mut references = Vec::new();
    let mut uncertain = false;
    for atomic in receiver.types.iter() {
        uncertain |= atomic.get_intersection_types().is_some_and(|types| !types.is_empty());
        let TAtomic::Object(object) = atomic else {
            uncertain = true;
            continue;
        };

        let Some(class) = object.get_name() else {
            uncertain = true;
            continue;
        };

        let lookup = MethodIdentifier::new(class, word(name.value));
        let method = context.codebase.get_declaring_method_identifier(&lookup);
        let Some(metadata) = context.codebase.get_method_by_id(&method).filter(|meta| !meta.flags.is_magic_method())
        else {
            uncertain = true;
            continue;
        };

        references.push(MethodReference {
            selector_span: (name.span.start.offset, name.span.end.offset),
            expression_span: (call.span().start.offset, call.span().end.offset),
            receiver_span: (call.object.span().start.offset, call.object.span().end.offset),
            origin: block.scope.get_reference_origin(),
            receiver_class: class,
            method: MethodIdentifier::new(
                method.get_class_name().to_ascii_lowercase(),
                method.get_method_name().to_ascii_lowercase(),
            ),
            declaration: metadata.name_span,
            static_access: false,
            dynamic: false,
            magic: false,
            uncertain: metadata.name_span.is_none() || !is_method_visible_by_id(context, block, &lookup),
        });
    }

    for reference in &mut references {
        reference.uncertain |= uncertain;
    }

    references
}

pub(super) fn method_return<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    block: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    call: &MethodCall<'arena>,
) -> Option<TUnion>
where
    A: Arena,
{
    let mut result = None;
    for reference in methods(context, block, artifacts, call) {
        if reference.uncertain {
            return None;
        }

        let metadata = context.codebase.get_method_by_id(&reference.method)?;
        let mut return_type = metadata.return_type_metadata.as_ref()?.type_union.clone();
        if return_type.has_template_types()
            || return_type.any_child_node(|node| {
                matches!(node, TypeRef::Atomic(TAtomic::Variable(_) | TAtomic::Conditional(_) | TAtomic::Derived(_)))
            })
        {
            return None;
        }

        let declaring = reference.method.get_class_name();
        if context.codebase.get_class_like_by_name(declaring).is_some_and(|class| class.kind.is_trait()) {
            for atomic in return_type.types.to_mut() {
                if let TAtomic::Object(TObject::Named(object)) = atomic
                    && object.name.as_bytes().eq_ignore_ascii_case(declaring.as_bytes())
                {
                    object.name = reference.receiver_class;
                }
            }
        }

        expand_union(
            context.codebase,
            &mut return_type,
            &TypeExpansionOptions {
                self_class: Some(reference.receiver_class),
                static_class_type: StaticClassType::Name(reference.receiver_class),
                function_is_final: metadata.method_metadata.as_ref().is_some_and(|method| method.is_final),
                ..TypeExpansionOptions::default()
            },
        );

        result = Some(add_optional_union_type(return_type, result.as_ref(), context.codebase));
    }

    result
}
