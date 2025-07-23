use mago_codex::ttype::TType;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

impl Analyzable for EnumCase {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLikeConstant,
        )?;

        self.item.analyze(context, block_context, artifacts)
    }
}

impl Analyzable for EnumCaseItem {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            EnumCaseItem::Unit(_) => Ok(()),
            EnumCaseItem::Backed(item) => item.analyze(context, block_context, artifacts),
        }
    }
}

impl Analyzable for EnumCaseBackedItem {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let Some(current_enum) = block_context.scope.get_class_like() else {
            return Err(AnalysisError::InternalError(
                "Internal Error: Enum case must be analyzed within an enum scope.".to_string(),
                self.span(),
            ));
        };

        let enum_name = context.interner.lookup(&current_enum.original_name);
        let case_name = context.interner.lookup(&self.name.value);

        let Some(backing_type) = &current_enum.enum_type else {
            context.buffer.report(
                TypingIssueKind::InvalidEnumCaseValue,
                Issue::error(format!(
                    "Case `{case_name}` in pure enum `{enum_name}` cannot have a value."
                ))
                .with_annotation(Annotation::primary(self.value.span()).with_message("This value is not allowed"))
                .with_annotation(
                    Annotation::secondary(current_enum.name_span.unwrap_or(current_enum.span))
                        .with_message(format!("`{enum_name}` is a pure enum and does not have a backing type")),
                )
                .with_help(format!("Either declare a backing type for the enum (e.g., `enum {enum_name}: int`) or remove the value from this case.")),
            );

            return Ok(());
        };

        self.value.analyze(context, block_context, artifacts)?;

        let Some(value_type) = artifacts.get_rc_expression_type(&self.value).cloned() else {
            context.buffer.report(
                TypingIssueKind::InvalidEnumCaseValue,
                Issue::error(format!("Could not infer the type of the value for case `{enum_name}::{case_name}`."))
                    .with_annotation(Annotation::primary(self.value.span()).with_message("The type of this value could not be determined"))
                    .with_note("The value of a backed enum case must be a constant expression that resolves to either a string or an integer.")
                    .with_help("Please use a literal or a constant expression for the value."),
            );

            return Ok(());
        };

        let backing_type_str = backing_type.get_id(Some(context.interner));

        if (backing_type.is_int() && !value_type.is_int()) || (backing_type.is_string() && !value_type.is_string()) {
            let value_type_str = value_type.get_id(Some(context.interner));

            context.buffer.report(
                TypingIssueKind::InvalidEnumCaseValue,
                Issue::error(format!(
                    "Invalid case value for `{enum_name}::{case_name}`. Expected `{backing_type_str}`, but got `{value_type_str}`."
                ))
                .with_annotation(
                    Annotation::primary(self.value.span())
                        .with_message(format!("This value has the type `{value_type_str}`")),
                )
                .with_annotation(
                    Annotation::secondary(current_enum.name_span.unwrap_or(current_enum.span))
                        .with_message(format!("Enum `{enum_name}` is defined here with a `{backing_type_str}` backing type")),
                )
                .with_help(format!("Ensure the case value is a literal {backing_type_str} or a constant expression that resolves to a {backing_type_str}.")),
            );
        }

        Ok(())
    }
}
