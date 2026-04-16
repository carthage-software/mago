use std::rc::Rc;

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::context::ScopeContext;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::wrap_atomic;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::HookedProperty;
use mago_syntax::ast::PlainProperty;
use mago_syntax::ast::Property;
use mago_syntax::ast::PropertyConcreteItem;
use mago_syntax::ast::PropertyHook;
use mago_syntax::ast::PropertyHookBody;
use mago_syntax::ast::PropertyHookConcreteBody;
use mago_syntax::ast::PropertyItem;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::analyze_statements;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::function_like::add_properties_to_context;
use crate::statement::function_like::get_this_type;
use crate::statement::function_like::report_undefined_type_references;
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
        );

        for item in &self.items {
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
        self.value.analyze(context, block_context, artifacts)?;

        if let Some(class_metadata) = block_context.scope.get_class_like()
            && let Some(property_metadata) = class_metadata.properties.get(&atom(self.variable.name))
            && let Some(declared_type_metadata) = property_metadata.type_metadata.as_ref()
            && !declared_type_metadata.type_union.is_mixed()
            && !declared_type_metadata.type_union.has_template_types()
            && !declared_type_metadata.type_union.is_generic_parameter()
            && let Some(value_type) = artifacts.get_expression_type(&self.value)
            && !value_type.is_never()
        {
            let declared_type = &declared_type_metadata.type_union;

            let mut comparison_result = ComparisonResult::new();
            if !union_comparator::is_contained_by(
                context.codebase,
                value_type,
                declared_type,
                true,
                true,
                false,
                &mut comparison_result,
            ) {
                let value_type_str = value_type.get_id();
                let declared_type_str = declared_type.get_id();
                let class_name = class_metadata.original_name;
                let property_name = self.variable.name;

                let issue = Issue::error(format!(
                    "Default value for property `{class_name}::{property_name}` is not assignable to its declared type."
                ))
                .with_annotation(
                    Annotation::primary(self.value.span())
                        .with_message(format!("This default value has type `{value_type_str}`")),
                )
                .with_annotation(
                    Annotation::secondary(declared_type_metadata.span)
                        .with_message(format!("Property is declared with type `{declared_type_str}`")),
                )
                .with_note("A property's default value must be assignable to the property's declared type.")
                .with_help("Change the default value to match the declared type, or update the property type to accept the default.");

                context.collector.report_with_code(IssueCode::InvalidPropertyDefaultValue, issue);
            }
        }

        Ok(())
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
        );
        self.item.analyze(context, block_context, artifacts)?;

        let property_name = atom(self.item.variable().name);
        for hook in &self.hook_list.hooks {
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
    );

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

        if let Some(param) = &hook_meta.parameter
            && let Some(param_type) = param.get_type_metadata()
        {
            report_undefined_type_references(context, param_type);

            // Only check native declaration if effective type is from docblock
            if param_type.from_docblock
                && let Some(param_type_decl) = param.get_type_declaration_metadata()
            {
                report_undefined_type_references(context, param_type_decl);
            }
        }
    }

    let mut hook_block_context = BlockContext::new(scope, context.settings.register_super_globals);

    if let Some(class_like_metadata) = parent_block_context.scope.get_class_like() {
        hook_block_context.locals.insert(
            Atom::from("$this"),
            Rc::new(wrap_atomic(TAtomic::Object(get_this_type(context, class_like_metadata, None)))),
        );

        add_properties_to_context(context, &mut hook_block_context, class_like_metadata, None)?;
    }

    if hook.name.value == "set" {
        let value_type = get_value_type_for_set_hook(property_name, parent_block_context);
        let param_name = hook
            .parameter_list
            .as_ref()
            .and_then(|p| p.parameters.first())
            .map_or_else(|| Atom::from("$value"), |p| Atom::from(p.variable.name));

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
                    Some(expr_body.expression),
                    value_type,
                    expr_body.expression.span(),
                );
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

    property.type_metadata.as_ref().map_or_else(get_mixed, |t| t.type_union.clone())
}
