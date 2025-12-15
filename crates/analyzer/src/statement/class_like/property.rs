use std::rc::Rc;

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::context::ScopeContext;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::wrap_atomic;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::analyze_statements;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::function_like::get_this_type;
use crate::statement::r#return::handle_return_value;

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

        let property_name = atom(self.item.variable().name);
        for hook in self.hook_list.hooks.iter() {
            analyze_property_hook(hook, property_name, context, block_context, artifacts)?;
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
        analyze_property_hook(self, atom(""), context, block_context, artifacts)
    }
}

fn analyze_property_hook<'ctx, 'arena>(
    hook: &PropertyHook<'arena>,
    property_name: mago_atom::Atom,
    context: &mut Context<'ctx, 'arena>,
    parent_block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    analyze_attributes(
        context,
        parent_block_context,
        artifacts,
        hook.attribute_lists.as_slice(),
        AttributeTarget::Method,
    )?;

    let PropertyHookBody::Concrete(body) = &hook.body else {
        return Ok(());
    };

    let mut scope = ScopeContext::new();
    scope.set_class_like(parent_block_context.scope.get_class_like());
    scope.set_static(false);

    if let Some(class_like) = parent_block_context.scope.get_class_like()
        && let Some(property) = class_like.properties.get(&property_name)
        && let Some(hook_meta) = property.hooks.get(&atom(hook.name.value))
    {
        scope.set_property_hook(Some((property_name, hook_meta)));
    }

    let mut hook_block_context = BlockContext::new(scope, context.settings.register_super_globals);

    if let Some(class_like_metadata) = parent_block_context.scope.get_class_like() {
        let this_type = wrap_atomic(TAtomic::Object(get_this_type(context, class_like_metadata, None)));
        hook_block_context.locals.insert(Atom::from("$this"), Rc::new(this_type));
        add_properties_to_hook_context(context, &mut hook_block_context, class_like_metadata)?;
    }

    if hook.name.value == "set" {
        let value_type = get_value_type_for_set_hook(property_name, parent_block_context);
        let param_name = hook
            .parameter_list
            .as_ref()
            .and_then(|p| p.parameters.first())
            .map(|p| Atom::from(p.variable.name))
            .unwrap_or_else(|| Atom::from("$value"));

        hook_block_context.locals.insert(param_name, Rc::new(value_type));
    }

    match body {
        PropertyHookConcreteBody::Block(block) => {
            analyze_statements(block.statements.as_slice(), context, &mut hook_block_context, artifacts)?;
        }
        PropertyHookConcreteBody::Expression(expr_body) => {
            expr_body.expression.analyze(context, &mut hook_block_context, artifacts)?;

            if hook.name.value == "get" {
                let value_type = artifacts
                    .get_rc_expression_type(&expr_body.expression)
                    .cloned()
                    .unwrap_or_else(|| Rc::new(get_mixed()));

                handle_return_value(
                    context,
                    &mut hook_block_context,
                    artifacts,
                    Some(&expr_body.expression),
                    value_type,
                    expr_body.expression.span(),
                )?;
            }
        }
    }

    Ok(())
}

fn get_value_type_for_set_hook(
    property_name: mago_atom::Atom,
    block_context: &BlockContext<'_>,
) -> mago_codex::ttype::union::TUnion {
    let Some(class_like) = block_context.scope.get_class_like() else {
        return get_mixed();
    };
    let Some(property) = class_like.properties.get(&property_name) else {
        return get_mixed();
    };

    if let Some(hook) = property.hooks.get(&atom("set"))
        && let Some(param) = &hook.parameter
        && let Some(type_metadata) = param.get_type_metadata()
    {
        return type_metadata.type_union.clone();
    }

    property.type_metadata.as_ref().map(|t| t.type_union.clone()).unwrap_or_else(get_mixed)
}

fn add_properties_to_hook_context<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    hook_block_context: &mut BlockContext<'ctx>,
    class_like_metadata: &mago_codex::metadata::class_like::ClassLikeMetadata,
) -> Result<(), AnalysisError> {
    for (property_name, declaring_class) in &class_like_metadata.declaring_property_ids {
        let Some(property_class) = context.codebase.get_class_like(declaring_class) else { continue };
        let Some(property) = property_class.properties.get(property_name) else { continue };

        if property.flags.is_static() {
            continue;
        }

        let property_type = property.type_metadata.as_ref().map(|t| t.type_union.clone()).unwrap_or_else(get_mixed);
        let raw_name = property_name.strip_prefix("$").unwrap_or(property_name);
        hook_block_context.locals.insert(Atom::from(&format!("$this->{raw_name}")), Rc::new(property_type));
    }

    Ok(())
}
