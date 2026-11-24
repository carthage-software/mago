use mago_allocator::Arena;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::TypeRef;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::reference::TReference;
use mago_codex::ttype::atomic::reference::TReferenceMemberSelector;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::ArrowFunction;
use mago_syntax::cst::ClassConstantAccess;
use mago_syntax::cst::ClassLikeConstant;
use mago_syntax::cst::ClassLikeConstantItem;
use mago_syntax::cst::ClassLikeConstantSelector;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::Closure;
use mago_syntax::cst::EnumCase;
use mago_syntax::cst::EnumCaseItem;
use mago_syntax::cst::Expression;
use mago_syntax::cst::LocalIdentifier;
use mago_syntax::walker::MutWalker;
use mago_word::Word;
use mago_word::word;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::function_like::check_type_metadata;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstant<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLikeConstant,
        )?;

        for item in &self.items {
            item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstantItem<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        self.value.analyze(context, block_context, artifacts)?;

        if let Some(class_metadata) = block_context.scope.get_class_like()
            && let Some(constant_metadata) = class_metadata.constants.get(&word(self.name.value))
        {
            let native_type = constant_metadata.type_declaration.as_ref().filter(|native| {
                constant_metadata.type_metadata.as_ref().is_some_and(|effective| effective.span != native.span)
            });

            for type_metadata in [constant_metadata.type_metadata.as_ref(), native_type].into_iter().flatten() {
                check_type_metadata(context, type_metadata);
            }

            if let Some(inferred_type) = constant_metadata.inferred_type.as_ref()
                && inferred_type.is_never()
            {
                context.collector.report_with_code(
                    IssueCode::UnresolvableClassConstant,
                    Issue::error(format!(
                        "Cannot resolve the value of class constant `{}::{}`.",
                        class_metadata.original_name,
                        mago_bytes::BytesDisplay(self.name.value),
                    ))
                    .with_annotation(
                        Annotation::primary(self.value.span()).with_message("This initializer could not be evaluated"),
                    )
                    .with_note("Mago could not determine a value for this constant from its initializer."),
                );
            } else if let Some(value_type) = artifacts.get_expression_type(&self.value) {
                for declared_type_metadata in
                    [constant_metadata.type_metadata.as_ref(), native_type].into_iter().flatten()
                {
                    if value_type.is_never()
                        || declared_type_metadata.type_union.is_mixed()
                        || declared_type_metadata.type_union.has_template_types()
                        || declared_type_metadata.type_union.is_generic_parameter()
                    {
                        continue;
                    }

                    let mut declared_type = declared_type_metadata.type_union.clone();
                    expand_union(
                        context.codebase,
                        &mut declared_type,
                        &TypeExpansionOptions {
                            self_class: Some(class_metadata.original_name),
                            static_class_type: StaticClassType::Name(class_metadata.original_name),
                            ..Default::default()
                        },
                    );

                    let mut comparison_result = ComparisonResult::new();
                    if !union_comparator::is_contained_by(
                        context.codebase,
                        value_type,
                        &declared_type,
                        true,
                        true,
                        false,
                        &mut comparison_result,
                    ) {
                        let value_type_str = value_type.get_id();
                        let declared_type_str = declared_type.get_id();
                        let class_name = class_metadata.original_name;
                        let constant_name = mago_bytes::BytesDisplay(self.name.value);

                        let issue = Issue::error(format!(
                        "Value for constant `{class_name}::{constant_name}` is not assignable to its declared type."
                    ))
                    .with_annotation(
                        Annotation::primary(self.value.span())
                            .with_message(format!("This value has type `{value_type_str}`")),
                    )
                    .with_annotation(
                        Annotation::secondary(declared_type_metadata.span)
                            .with_message(format!("Constant is declared with type `{declared_type_str}`")),
                    )
                    .with_note("A class constant's value must be assignable to its declared type.")
                    .with_help(
                        "Change the value to match the declared type, or update the declared type to accept the value.",
                    );

                        context.collector.report_with_code(IssueCode::InvalidConstantValue, issue);

                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

struct ClassConstantAccesses<'ast, 'arena>(Vec<&'ast ClassConstantAccess<'arena>>);

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for ClassConstantAccesses<'ast, 'arena> {
    fn walk_in_class_constant_access(&mut self, access: &'ast ClassConstantAccess<'arena>, _: &mut ()) {
        self.0.push(access);
    }

    fn walk_closure(&mut self, _: &'ast Closure<'arena>, _: &mut ()) {}

    fn walk_arrow_function(&mut self, _: &'ast ArrowFunction<'arena>, _: &mut ()) {}
}

type Member<'ctx> = (&'ctx ClassLikeMetadata, Word);

fn member_span(class: &ClassLikeMetadata, member: Word) -> Option<Span> {
    class
        .constants
        .get(&member)
        .map(|constant| constant.span)
        .or_else(|| class.enum_cases.get(&member).map(|case| case.span))
}

fn resolve_access<'ctx, A>(
    context: &Context<'ctx, '_, A>,
    current: &'ctx ClassLikeMetadata,
    access: &ClassConstantAccess<'_>,
) -> Option<Member<'ctx>>
where
    A: Arena,
{
    let ClassLikeConstantSelector::Identifier(member) = &access.constant else {
        return None;
    };

    let class = match access.class {
        Expression::Self_(_) | Expression::Static(_) => current,
        Expression::Parent(_) => context.codebase.get_class_like_by_name(current.direct_parent_class?)?,
        Expression::Identifier(identifier) => {
            context.codebase.get_class_like(context.resolved_names.get(identifier))?
        }
        _ => return None,
    };

    Some((class, word(member.value)))
}

/// Reports constants and enum cases whose value leads back to themselves, which PHP
/// rejects when it first evaluates them.
pub(super) fn check_self_referencing_constants<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    members: &[ClassLikeMember<'arena>],
) where
    A: Arena,
{
    let mut values = Vec::new();
    for member in members {
        let items: Vec<(&LocalIdentifier<'arena>, &Expression<'arena>)> = match member {
            ClassLikeMember::Constant(constant) => constant.items.iter().map(|item| (&item.name, item.value)).collect(),
            ClassLikeMember::EnumCase(EnumCase { item: EnumCaseItem::Backed(item), .. }) => {
                vec![(&item.name, item.value)]
            }
            _ => continue,
        };

        for (name, value) in items {
            let name = word(name.value);
            let mut accesses = ClassConstantAccesses(Vec::new());
            accesses.walk_expression(value, &mut ());
            if let Some(span) = member_span(class_like_metadata, name) {
                values.push((name, span, value, accesses.0));
            }
        }
    }

    if values.iter().all(|(.., accesses)| accesses.is_empty()) {
        return;
    }

    for (name, start, value, accesses) in &values {
        if class_like_metadata
            .constants
            .get(name)
            .and_then(|constant| constant.inferred_type.as_ref())
            .is_some_and(TAtomic::is_never)
        {
            continue;
        }

        let mut stack: Vec<(Member<'ctx>, Member<'ctx>)> = accesses
            .iter()
            .filter_map(|access| resolve_access(context, class_like_metadata, access))
            .map(|target| (target, target))
            .collect();
        let mut visited = vec![*start];
        let mut through = None;

        while let Some(((class, member), first)) = stack.pop() {
            let Some(span) = member_span(class, member) else {
                continue;
            };

            if span == *start {
                through = Some(first);
                break;
            }

            if visited.contains(&span) {
                continue;
            }

            visited.push(span);

            if let Some((.., accesses)) = values.iter().find(|(_, value_span, ..)| *value_span == span) {
                stack.extend(
                    accesses
                        .iter()
                        .filter_map(|access| resolve_access(context, class_like_metadata, access))
                        .map(|target| (target, first)),
                );

                continue;
            }

            let atom = class
                .constants
                .get(&member)
                .and_then(|constant| constant.inferred_type.as_ref())
                .or_else(|| class.enum_cases.get(&member).and_then(|case| case.value_type.as_ref()));

            for node in atom
                .into_iter()
                .flat_map(|atom| std::iter::once(TypeRef::Atomic(atom)).chain(atom.get_all_child_nodes()))
            {
                if let TypeRef::Atomic(TAtomic::Reference(TReference::Member {
                    class_like_name,
                    member_selector: TReferenceMemberSelector::Identifier(member),
                })) = node
                    && let Some(class) = context.codebase.get_class_like_by_name(*class_like_name)
                {
                    stack.push(((class, *member), first));
                }
            }
        }

        let Some((through_class, through_member)) = through else {
            continue;
        };

        let kind = if class_like_metadata.enum_cases.contains_key(name) { "Case" } else { "Constant" };
        let class_name = class_like_metadata.original_name;
        let through = if through_class.name == class_like_metadata.name && through_member == *name {
            String::new()
        } else {
            format!(" through `{}::{through_member}`", through_class.original_name)
        };

        context.collector.report_with_code(
            IssueCode::SelfReferencingConstant,
            Issue::error(format!("{kind} `{class_name}::{name}` refers back to itself{through}."))
                .with_annotation(
                    Annotation::primary(value.span())
                        .with_message(format!("Evaluating this value needs `{class_name}::{name}` itself")),
                )
                .with_note(
                    "PHP throws `Error: Cannot declare self-referencing constant` when it first evaluates this value.",
                )
                .with_help("Give one of these constants a value that does not depend on the others."),
        );
    }
}
