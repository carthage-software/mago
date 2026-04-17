use mago_atom::atom;
use mago_codex::ttype::TType;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeConstant;
use mago_syntax::ast::ClassLikeConstantItem;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstant<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLikeConstant,
        );

        for item in &self.items {
            item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstantItem<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.value.analyze(context, block_context, artifacts)?;

        if let Some(class_metadata) = block_context.scope.get_class_like()
            && let Some(constant_metadata) = class_metadata.constants.get(&atom(self.name.value))
        {
            if let Some(inferred_type) = constant_metadata.inferred_type.as_ref()
                && inferred_type.is_never()
            {
                context.collector.report_with_code(
                    IssueCode::UnresolvableClassConstant,
                    Issue::error(format!(
                        "Cannot resolve the value of class constant `{}::{}`.",
                        class_metadata.original_name, self.name.value,
                    ))
                    .with_annotation(
                        Annotation::primary(self.value.span()).with_message("This initializer could not be evaluated"),
                    )
                    .with_note("Mago could not determine a value for this constant from its initializer."),
                );
            } else if let Some(declared_type_metadata) = constant_metadata.type_metadata.as_ref()
                && let Some(value_type) = artifacts.get_expression_type(&self.value)
                && !value_type.is_never()
                && !declared_type_metadata.type_union.is_mixed()
                && !declared_type_metadata.type_union.has_template_types()
                && !declared_type_metadata.type_union.is_generic_parameter()
            {
                let mut declared_type = declared_type_metadata.type_union.clone();
                expand_union(
                    context.codebase,
                    &mut declared_type,
                    &TypeExpansionOptions {
                        self_class: Some(class_metadata.original_name),
                        static_class_type: StaticClassType::Name(class_metadata.original_name),
                        evaluate_class_constants: true,
                        evaluate_conditional_types: true,
                        expand_generic: true,
                        expand_templates: true,
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
                    let constant_name = self.name.value;

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
                }
            }
        }

        Ok(())
    }
}
