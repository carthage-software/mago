use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Property<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Property::Plain(plain) => plain.analyze(context, block_context, artifacts),
            Property::Hooked(hooked) => hooked.analyze(context, block_context, artifacts),
        }
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PlainProperty<'arena> {
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
            AttributeTarget::Property,
        )?;

        for item in self.items.iter() {
            item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PropertyItem<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if let PropertyItem::Concrete(property_concrete_item) = self {
            property_concrete_item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PropertyConcreteItem<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.value.analyze(context, block_context, artifacts)
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for HookedProperty<'arena> {
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
            AttributeTarget::Property,
        )?;

        self.item.analyze(context, block_context, artifacts)?;

        for hook in self.hook_list.hooks.iter() {
            hook.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PropertyHook<'arena> {
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
            AttributeTarget::Method,
        )?;

        // TODO(azjezz): analyze the hook body, but currently we don't scan it in codex..

        Ok(())
    }
}
