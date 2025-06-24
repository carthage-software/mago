use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

impl Analyzable for Property {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Property::Plain(plain) => plain.analyze(context, block_context, artifacts),
            Property::Hooked(hooked) => hooked.analyze(context, block_context, artifacts),
        }
    }
}

impl Analyzable for PlainProperty {
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
            AttributeTarget::Property,
        )?;

        for item in self.items.iter() {
            item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl Analyzable for PropertyItem {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if let PropertyItem::Concrete(property_concrete_item) = self {
            property_concrete_item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl Analyzable for PropertyConcreteItem {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.value.analyze(context, block_context, artifacts)
    }
}

impl Analyzable for HookedProperty {
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
            AttributeTarget::Property,
        )?;

        self.item.analyze(context, block_context, artifacts)?;

        for hook in self.hook_list.hooks.iter() {
            hook.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl Analyzable for PropertyHook {
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
            AttributeTarget::Method,
        )?;

        // TODO(azjezz): analyze the hook body, but currently we don't scan it in codex..

        Ok(())
    }
}
