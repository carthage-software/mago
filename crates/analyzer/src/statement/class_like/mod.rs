use ahash::RandomState;
use indexmap::IndexMap;
use itertools::Itertools;

use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_codex::context::ScopeContext;

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::template::standin_type_replacer;
use mago_codex::ttype::template::standin_type_replacer::StandinOptions;
use mago_codex::ttype::union::TUnion;
use mago_codex::visibility::Visibility;
use mago_names::kind::NameKind;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Class;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::Enum;
use mago_syntax::ast::Extends;
use mago_syntax::ast::Implements;
use mago_syntax::ast::Interface;
use mago_syntax::ast::Property;
use mago_syntax::ast::Trait;
use mago_syntax::ast::TraitUse;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::plugin::context::HookContext;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::class_like::method_signature::SignatureCompatibilityIssue;
use crate::statement::function_like::report_undefined_type_references;
use crate::utils::missing_type_hints;

pub mod constant;
pub mod enum_case;
pub mod initialization;
pub mod method;
pub mod method_signature;
pub mod override_attribute;
pub mod property;
pub mod unused_members;

/// Reports a duplicate definition issue for class-like types.
fn report_duplicate_definition(
    context: &mut Context<'_, '_>,
    title: &str,
    kind: &str,
    name: &str,
    duplicate_span: Span,
    original_span: Span,
) {
    context.collector.report_with_code(
        IssueCode::DuplicateDefinition,
        Issue::error(format!("{title} `{name}` is already defined elsewhere."))
            .with_annotation(
                Annotation::primary(duplicate_span).with_message(format!("Duplicate {kind} definition here")),
            )
            .with_annotation(Annotation::secondary(original_span).with_message(format!("Original {kind} defined here")))
            .with_note("Each class, interface, trait, or enum must have a unique name within the same namespace.")
            .with_note("The duplicate definition will be ignored during analysis.")
            .with_help(
                "Consider using namespaces to avoid naming conflicts, or remove one of the duplicate definitions.",
            ),
    );
}

/// Helper function to check if a child type is compatible with (contained by) a parent type.
///
/// This is a convenience wrapper around `union_comparator::is_contained_by` with standard
/// settings for inheritance checks (no null/false ignoring, not inside assertion).
#[inline]
fn is_type_compatible(codebase: &mago_codex::metadata::CodebaseMetadata, child: &TUnion, parent: &TUnion) -> bool {
    union_comparator::is_contained_by(codebase, child, parent, false, false, false, &mut ComparisonResult::default())
}

/// Represents different types of property conflicts between traits
#[derive(Debug)]
enum PropertyConflict {
    Visibility(Visibility, Visibility, Visibility, Visibility),
    Static(bool, bool),
    Readonly(bool, bool),
    Type(Option<String>, Option<String>),
    Default(Option<String>, Option<String>),
    HookedProperty,
}

impl PropertyConflict {
    fn describe(&self) -> String {
        match self {
            PropertyConflict::Visibility(r1, w1, r2, w2) => {
                let p1_vis = if r1 == w1 { r1.to_string() } else { format!("{r1} {w1}(set)") };
                let p2_vis = if r2 == w2 { r2.to_string() } else { format!("{r2} {w2}(set)") };
                format!("visibility differs ({p1_vis} vs {p2_vis})")
            }
            PropertyConflict::Static(s1, s2) => {
                let p1_mod = if *s1 { "static" } else { "instance" };
                let p2_mod = if *s2 { "static" } else { "instance" };
                format!("static modifier differs ({p1_mod} vs {p2_mod})")
            }
            PropertyConflict::Readonly(r1, r2) => {
                let p1_mod = if *r1 { "readonly" } else { "not readonly" };
                let p2_mod = if *r2 { "readonly" } else { "not readonly" };
                format!("readonly modifier differs ({p1_mod} vs {p2_mod})")
            }
            PropertyConflict::Type(t1, t2) => match (t1, t2) {
                (Some(type1), Some(type2)) => format!("type declaration differs ({type1} vs {type2})"),
                (Some(type1), None) => format!("type declaration differs ({type1} vs untyped)"),
                (None, Some(type2)) => format!("type declaration differs (untyped vs {type2})"),
                (None, None) => unreachable!(),
            },
            PropertyConflict::Default(d1, d2) => match (d1, d2) {
                (Some(def1), Some(def2)) => format!("default value differs ({def1} vs {def2})"),
                (Some(def1), None) => format!("default value differs ({def1} vs no default)"),
                (None, Some(def2)) => format!("default value differs (no default vs {def2})"),
                (None, None) => unreachable!(),
            },
            PropertyConflict::HookedProperty => {
                "conflict resolution between hooked properties is not supported".to_string()
            }
        }
    }

    fn get_issue_code(&self) -> IssueCode {
        match self {
            PropertyConflict::Visibility(_, _, _, _) => IssueCode::IncompatiblePropertyVisibility,
            PropertyConflict::Static(_, _) => IssueCode::IncompatiblePropertyStatic,
            PropertyConflict::Readonly(_, _) => IssueCode::IncompatiblePropertyReadonly,
            PropertyConflict::Type(_, _) => IssueCode::IncompatiblePropertyType,
            PropertyConflict::Default(_, _) => IssueCode::IncompatiblePropertyDefault,
            PropertyConflict::HookedProperty => IssueCode::IncompatiblePropertyOverride,
        }
    }
}

/// Checks if a type union contains a reference to a specific template parameter.
fn type_contains_template_param(type_union: &TUnion, param_name: Atom, defining_class: Atom) -> bool {
    use mago_codex::ttype::TypeRef;

    type_union.types.iter().any(|atomic| {
        if let TAtomic::GenericParameter(TGenericParameter {
            parameter_name,
            defining_entity: GenericParent::ClassLike(class_name),
            ..
        }) = atomic
            && *parameter_name == param_name
            && *class_name == defining_class
        {
            return true;
        }

        if let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
            parameter_name,
            defining_entity: GenericParent::ClassLike(class_name),
            ..
        })) = atomic
            && *parameter_name == param_name
            && *class_name == defining_class
        {
            return true;
        }

        atomic.get_all_child_nodes().iter().any(|node| match node {
            TypeRef::Atomic(TAtomic::GenericParameter(gp)) => {
                gp.parameter_name == param_name
                    && matches!(gp.defining_entity, GenericParent::ClassLike(c) if c == defining_class)
            }
            TypeRef::Atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                parameter_name,
                defining_entity,
                ..
            }))) => {
                *parameter_name == param_name
                    && matches!(defining_entity, GenericParent::ClassLike(c) if *c == defining_class)
            }
            TypeRef::Union(u) => type_contains_template_param(u, param_name, defining_class),
            _ => false,
        })
    })
}

/// Checks for unused template parameters in a class-like declaration.
///
/// A template parameter is considered "used" if it appears in:
/// - A property type
/// - A method parameter type
/// - A method return type
/// - An `@extends`, `@implements`, or `@use` annotation
fn check_unused_template_parameters<'ctx>(
    context: &mut Context<'ctx, '_>,
    class_like_metadata: &'ctx ClassLikeMetadata,
) {
    if !context.settings.find_unused_definitions {
        return;
    }

    if class_like_metadata.template_types.is_empty() {
        return;
    }

    let class_name = class_like_metadata.name;
    let class_original_name = class_like_metadata.original_name;
    let class_kind_str = class_like_metadata.kind.as_str();
    let class_name_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);

    for (template_name, _) in &class_like_metadata.template_types {
        if template_name.as_str().starts_with('_') {
            continue;
        }

        let mut is_used = false;

        for extended_params in class_like_metadata.template_extended_parameters.values() {
            for (_param_name, param_type) in extended_params {
                if type_contains_template_param(param_type, *template_name, class_name) {
                    is_used = true;
                    break;
                }
            }

            if is_used {
                break;
            }
        }

        if is_used {
            continue;
        }

        for property_metadata in class_like_metadata.properties.values() {
            if let Some(type_metadata) = &property_metadata.type_metadata
                && type_contains_template_param(&type_metadata.type_union, *template_name, class_name)
            {
                is_used = true;
                break;
            }
        }

        if is_used {
            continue;
        }

        for method_id in class_like_metadata.declaring_method_ids.values() {
            let Some(function_like) =
                context.codebase.get_method(method_id.get_class_name(), method_id.get_method_name())
            else {
                continue;
            };

            // Check parameters
            for param in &function_like.parameters {
                if let Some(type_metadata) = &param.type_metadata
                    && type_contains_template_param(&type_metadata.type_union, *template_name, class_name)
                {
                    is_used = true;
                    break;
                }
            }

            if is_used {
                break;
            }

            // Check return type
            if let Some(return_type_metadata) = &function_like.return_type_metadata
                && type_contains_template_param(&return_type_metadata.type_union, *template_name, class_name)
            {
                is_used = true;
                break;
            }
        }

        if is_used {
            continue;
        }

        // Report warning if template parameter is unused
        context.collector.report_with_code(
            IssueCode::UnusedTemplateParameter,
            Issue::warning(format!(
                "Template parameter `{template_name}` is never used in {class_kind_str} `{class_original_name}`."
            ))
            .with_annotation(
                Annotation::primary(class_name_span)
                    .with_message(format!("Template `{template_name}` is defined on this {class_kind_str} but never referenced")),
            )
            .with_help(format!(
                "Remove the unused `@template {template_name}` from the docblock, or use it in a property, method signature, or inherited type."
            )),
        );
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Class<'arena> {
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
            AttributeTarget::ClassLike,
        );

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = context.codebase.get_class_like(name) else {
            tracing::warn!("Class {} not found in codebase", name);

            return Ok(());
        };

        if class_like_metadata.span != self.span() {
            report_duplicate_definition(context, "Class", "class", name, self.span(), class_like_metadata.span);
            return Ok(());
        }

        // Call plugin on_enter_class hooks
        if context.plugin_registry.has_class_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_enter_class(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        analyze_class_like(
            context,
            artifacts,
            Some(self.name.span),
            self.span(),
            self.extends.as_ref(),
            self.implements.as_ref(),
            class_like_metadata,
            self.members.as_slice(),
        )?;

        if context.settings.check_missing_override {
            override_attribute::check_override_attribute(class_like_metadata, self.members.as_slice(), context);
        }

        if context.settings.find_unused_definitions {
            let unused_members = unused_members::check_unused_members_with_transitivity(
                class_like_metadata.name,
                self.span(),
                class_like_metadata,
                &artifacts.symbol_references,
                context,
            );

            unused_members::check_write_only_properties(
                class_like_metadata.name,
                self.span(),
                class_like_metadata,
                &artifacts.symbol_references,
                &unused_members,
                context,
            );
        }

        // Call plugin on_leave_class hooks
        if context.plugin_registry.has_class_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_leave_class(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Interface<'arena> {
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
            AttributeTarget::ClassLike,
        );

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = context.codebase.get_class_like(name) else {
            tracing::warn!("Interface {name} not found in codebase");

            return Ok(());
        };

        if class_like_metadata.span != self.span() {
            report_duplicate_definition(context, "Interface", "interface", name, self.span(), class_like_metadata.span);
            return Ok(());
        }

        // Call plugin on_enter_interface hooks
        if context.plugin_registry.has_interface_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_enter_interface(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        analyze_class_like(
            context,
            artifacts,
            Some(self.name.span),
            self.span(),
            self.extends.as_ref(),
            None,
            class_like_metadata,
            self.members.as_slice(),
        )?;

        if context.settings.check_missing_override {
            override_attribute::check_override_attribute(class_like_metadata, self.members.as_slice(), context);
        }

        // Call plugin on_leave_interface hooks
        if context.plugin_registry.has_interface_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_leave_interface(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Trait<'arena> {
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
            AttributeTarget::ClassLike,
        );

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = context.codebase.get_class_like(name) else {
            tracing::warn!("Trait {} not found in codebase", name);

            return Ok(());
        };

        if class_like_metadata.span != self.span() {
            report_duplicate_definition(context, "Trait", "trait", name, self.span(), class_like_metadata.span);
            return Ok(());
        }

        // Call plugin on_enter_trait hooks
        if context.plugin_registry.has_trait_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_enter_trait(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        analyze_class_like(
            context,
            artifacts,
            Some(self.name.span),
            self.span(),
            None,
            None,
            class_like_metadata,
            self.members.as_slice(),
        )?;

        if context.settings.check_missing_override {
            override_attribute::check_override_attribute(class_like_metadata, self.members.as_slice(), context);
        }

        // Call plugin on_leave_trait hooks
        if context.plugin_registry.has_trait_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_leave_trait(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Enum<'arena> {
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
            AttributeTarget::ClassLike,
        );

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = context.codebase.get_class_like(name) else {
            tracing::warn!("Enum {} not found in codebase", name);

            return Ok(());
        };

        if class_like_metadata.span != self.span() {
            report_duplicate_definition(context, "Enum", "enum", name, self.span(), class_like_metadata.span);
            return Ok(());
        }

        // Call plugin on_enter_enum hooks
        if context.plugin_registry.has_enum_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_enter_enum(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        analyze_class_like(
            context,
            artifacts,
            Some(self.name.span),
            self.span(),
            None,
            self.implements.as_ref(),
            class_like_metadata,
            self.members.as_slice(),
        )?;

        if context.settings.check_missing_override {
            override_attribute::check_override_attribute(class_like_metadata, self.members.as_slice(), context);
        }

        if context.settings.find_unused_definitions {
            unused_members::check_unused_members_with_transitivity(
                class_like_metadata.name,
                self.span(),
                class_like_metadata,
                &artifacts.symbol_references,
                context,
            );
        }

        // Call plugin on_leave_enum hooks
        if context.plugin_registry.has_enum_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_leave_enum(self, class_like_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        Ok(())
    }
}

pub(crate) fn analyze_class_like<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    artifacts: &mut AnalysisArtifacts,
    name_span: Option<Span>,
    declaration_span: Span,
    extends_ast: Option<&'ast Extends<'arena>>,
    implements_ast: Option<&'ast Implements<'arena>>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    members: &'ast [ClassLikeMember<'arena>],
) -> Result<(), AnalysisError> {
    if context.settings.diff && context.codebase.safe_symbols.contains(&class_like_metadata.name) {
        return Ok(());
    }

    for parent_class in &class_like_metadata.all_parent_classes {
        artifacts.symbol_references.add_symbol_reference_to_symbol(class_like_metadata.name, *parent_class, true);
    }

    for parent_interface in &class_like_metadata.all_parent_interfaces {
        artifacts.symbol_references.add_symbol_reference_to_symbol(class_like_metadata.name, *parent_interface, true);
    }

    for trait_name in &class_like_metadata.used_traits {
        artifacts.symbol_references.add_symbol_reference_to_symbol(class_like_metadata.name, *trait_name, true);
    }

    if class_like_metadata.flags.is_unchecked() {
        return Ok(());
    }

    let name = &class_like_metadata.original_name;

    check_class_like_extends(context, class_like_metadata, extends_ast);
    check_class_like_implements(context, class_like_metadata, implements_ast);

    for member in members {
        if let ClassLikeMember::TraitUse(used_trait) = member {
            check_class_like_use(context, class_like_metadata, used_trait);
        }
    }

    if !class_like_metadata.invalid_dependencies.is_empty() {
        return Ok(());
    }

    if !class_like_metadata.kind.is_trait() && !class_like_metadata.flags.is_abstract() {
        for (method_name, method_id) in &class_like_metadata.declaring_method_ids {
            if class_like_metadata.kind.is_enum() {
                if method_name.eq_ignore_ascii_case("cases") {
                    continue;
                }

                if class_like_metadata.enum_type.is_some()
                    && (method_name.eq_ignore_ascii_case("from") || method_name.eq_ignore_ascii_case("tryFrom"))
                {
                    continue;
                }
            }

            let Some(declaring_class_like_metadata) = context.codebase.get_class_like(method_id.get_class_name())
            else {
                continue;
            };

            let Some(function_like) =
                context.codebase.get_method(method_id.get_class_name(), method_id.get_method_name())
            else {
                continue;
            };

            let Some(method_metadata) = function_like.method_metadata.as_ref() else {
                continue;
            };

            if method_metadata.is_abstract {
                let fqcn = declaring_class_like_metadata.original_name;
                let method_span = function_like.name_span.unwrap_or(function_like.span);

                context.collector.report_with_code(
                    IssueCode::UnimplementedAbstractMethod,
                    Issue::error(format!(
                        "Class `{name}` does not implement the abstract method `{method_name}`.",
                    ))
                    .with_annotation(
                        Annotation::primary(name_span.unwrap_or(declaration_span))
                            .with_message(format!("`{name}` is not abstract and must implement this method")),
                    )
                    .with_annotation(
                        Annotation::secondary(method_span).with_message(
                            format!("`{fqcn}::{method_name}` is defined as abstract here")
                        ),
                    )
                    .with_note("When a concrete class extends an abstract class or implements an interface, it must provide an implementation for all inherited abstract methods.".to_string())
                    .with_help(format!(
                        "You can either implement the `{method_name}` method in `{name}`, or declare `{name}` as an abstract class.",
                    )),
                );
            }
        }

        for property_name in class_like_metadata.declaring_property_ids.keys() {
            let current_property = class_like_metadata.properties.get(property_name);

            for parent_fqcn in class_like_metadata
                .all_parent_classes
                .iter()
                .chain(class_like_metadata.all_parent_interfaces.iter())
                .chain(class_like_metadata.used_traits.iter())
            {
                let Some(parent_metadata) = context.codebase.get_class_like(parent_fqcn) else {
                    continue;
                };

                let Some(parent_property) = parent_metadata.properties.get(property_name) else {
                    continue;
                };

                for (hook_name, hook_metadata) in &parent_property.hooks {
                    if !hook_metadata.is_abstract {
                        continue;
                    }

                    let is_implemented = current_property
                        .map(|p| {
                            if p.hooks.is_empty() {
                                !p.flags.is_virtual_property()
                            } else {
                                // Property has hooks - check if the specific hook is implemented
                                if p.hooks.get(hook_name).is_some_and(|h| !h.is_abstract) {
                                    return true;
                                }

                                p.hooks.values().any(|h| !h.is_abstract)
                            }
                        })
                        .unwrap_or_else(|| {
                            class_like_metadata.all_parent_classes.iter().any(|parent_class_fqcn| {
                                context
                                    .codebase
                                    .get_class_like(parent_class_fqcn)
                                    .and_then(|parent| parent.properties.get(property_name))
                                    .is_some_and(|prop| {
                                        if prop.hooks.get(hook_name).is_some_and(|h| !h.is_abstract) {
                                            return true;
                                        }

                                        !prop.flags.is_virtual_property() || prop.hooks.values().any(|h| !h.is_abstract)
                                    })
                            })
                        });

                    if !is_implemented {
                        let fqcn = parent_metadata.original_name;
                        let hook_span = hook_metadata.span;

                        context.collector.report_with_code(
                            IssueCode::UnimplementedAbstractPropertyHook,
                            Issue::error(format!(
                                "Class `{name}` does not implement the abstract property hook `{property_name}::{hook_name}()`.",
                            ))
                            .with_annotation(
                                Annotation::primary(name_span.unwrap_or(declaration_span))
                                    .with_message(format!("`{name}` is not abstract and must implement this hook")),
                            )
                            .with_annotation(
                                Annotation::secondary(hook_span).with_message(
                                    format!("`{fqcn}::{property_name}::{hook_name}()` is defined as abstract here")
                                ),
                            )
                            .with_note("When a concrete class extends an abstract class or implements an interface, it must provide an implementation for all inherited abstract property hooks.".to_string())
                            .with_help(format!(
                                "You can either implement the `{hook_name}` hook for property `{property_name}` in `{name}`, or declare `{name}` as an abstract class.",
                            )),
                        );
                    }
                }
            }
        }
    }

    if !class_like_metadata.kind.is_trait() {
        check_abstract_method_signatures(context, class_like_metadata);
        check_trait_method_conflicts(context, class_like_metadata, members);
    }

    check_trait_property_conflicts(context, class_like_metadata, members);
    check_readonly_class_trait_properties(context, class_like_metadata, members);

    if !class_like_metadata.template_types.is_empty() {
        for (template_name, _) in &class_like_metadata.template_types {
            let (resolved_template_name, _) = context.scope.resolve(NameKind::Default, template_name);
            if let Some(conflicting_class) = context.codebase.get_class_like(&resolved_template_name) {
                let conflicting_name = &conflicting_class.name;
                let conflicting_class_span = conflicting_class.name_span.unwrap_or(conflicting_class.span);

                context.collector.report_with_code(
                    IssueCode::NameAlreadyInUse,
                    Issue::error(format!(
                        "In class `{name}`, the template parameter `{template_name}` conflicts with an existing class.",
                    ))
                    .with_annotation(
                        Annotation::primary(name_span.unwrap_or(declaration_span))
                            .with_message("The docblock for this class defines the conflicting template parameter"),
                    )
                    .with_annotation(
                        Annotation::secondary(conflicting_class_span)
                            .with_message(format!("The conflicting type `{conflicting_name}` is defined here")),
                    )
                    .with_note("Template parameter names (from `@template`) must not conflict with existing classes, interfaces, enums, or traits in the same scope.")
                    .with_help(format!(
                        "In the docblock for the `{name}` type, rename the `@template {template_name}` parameter to avoid this naming collision.",
                    )),
                );
            }
        }
    }

    check_unused_template_parameters(context, class_like_metadata);
    check_class_like_properties(context, class_like_metadata);

    let mut scope = ScopeContext::new();
    scope.set_class_like(Some(class_like_metadata));
    scope.set_static(true);

    let mut block_context = BlockContext::new(scope, context.settings.register_super_globals);

    for member in members {
        match member {
            ClassLikeMember::Constant(class_like_constant) => {
                missing_type_hints::check_constant_type_hint(context, class_like_constant);

                class_like_constant.analyze(context, &mut block_context, artifacts)?;
            }
            ClassLikeMember::Property(property) => {
                missing_type_hints::check_property_type_hint(context, class_like_metadata, property);

                let property_names: Vec<Atom> = match property {
                    Property::Plain(plain) => plain.items.iter().map(|item| atom(item.variable().name)).collect(),
                    Property::Hooked(hooked) => {
                        vec![atom(hooked.item.variable().name)]
                    }
                };

                for property_name in property_names {
                    if let Some(prop_meta) = class_like_metadata.properties.get(&property_name)
                        && let Some(type_meta) = &prop_meta.type_metadata
                    {
                        report_undefined_type_references(context, type_meta);

                        if type_meta.from_docblock
                            && let Some(type_decl_meta) = &prop_meta.type_declaration_metadata
                        {
                            report_undefined_type_references(context, type_decl_meta);
                        }
                    }
                }

                property.analyze(context, &mut block_context, artifacts)?;
            }
            ClassLikeMember::EnumCase(enum_case) => {
                enum_case.analyze(context, &mut block_context, artifacts)?;
            }
            ClassLikeMember::Method(method) => {
                method.analyze(context, &mut block_context, artifacts)?;
            }
            _ => {}
        }
    }

    // Check trait constant overrides AFTER constants have been analyzed
    // so we can compare their inferred values
    check_class_like_constants(context, class_like_metadata, members);

    initialization::check_property_initialization(context, artifacts, class_like_metadata, declaration_span, name_span);

    Ok(())
}

fn check_class_like_extends<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    extends_ast: Option<&Extends<'arena>>,
) {
    // This check only applies to classes and interfaces, which can use `extends`.
    if !class_like_metadata.kind.is_class() && !class_like_metadata.kind.is_interface() {
        return;
    }

    let Some(extends) = extends_ast else {
        return;
    };

    let using_kind_str = class_like_metadata.kind.as_str();
    let using_kind_capitalized =
        format!("{}{}", using_kind_str.chars().next().unwrap().to_uppercase(), &using_kind_str[1..]);
    let using_name = class_like_metadata.original_name;
    let using_class_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);

    for extended_type in &extends.types {
        let extended_type_str = context.resolved_names.get(&extended_type);
        let extended_class_metadata = context.codebase.get_class_like(extended_type_str);

        // Case: The extended type does not exist.
        let Some(extended_class_metadata) = extended_class_metadata else {
            let extended_name = extended_type.value();

            context.collector.report_with_code(
                IssueCode::NonExistentClassLike,
                Issue::error(format!("{using_kind_capitalized} `{using_name}` cannot extend unknown type `{extended_name}`"))
                    .with_annotation(Annotation::primary(extended_type.span()).with_message("This type could not be found"))
                    .with_note("Mago could not find a definition for this class, interface, or trait.")
                    .with_help("Ensure the name is correct, including its namespace, and that it is properly defined and autoloadable."),
            );
            continue;
        };

        let extended_name = extended_class_metadata.original_name;
        let extended_kind_str = extended_class_metadata.kind.as_str();
        let extended_kind_prefix =
            if extended_class_metadata.kind.is_class() || extended_class_metadata.kind.is_trait() { "a" } else { "an" };
        let extended_class_span = extended_class_metadata.name_span.unwrap_or(extended_class_metadata.span);

        if extended_class_metadata.flags.is_deprecated() {
            context.collector.report_with_code(
                IssueCode::DeprecatedClass,
                Issue::warning(format!("Use of deprecated class `{extended_name}` in `extends` clause"))
                    .with_annotation(Annotation::primary(extended_type.span()).with_message("This class is marked as deprecated"))
                    .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("`{extended_name}` was marked deprecated here")))
                    .with_note("The parent type is deprecated and may be removed in a future version, which would break this child type.")
                    .with_help("Consider refactoring to avoid extending this type, or consult its documentation for alternatives."),
            );
        }

        if class_like_metadata.kind.is_interface() {
            if !extended_class_metadata.kind.is_interface() {
                context.collector.report_with_code(
                    IssueCode::InvalidExtend,
                    Issue::error(format!("Interface `{using_name}` cannot extend non-interface type `{extended_name}`"))
                        .with_annotation(Annotation::primary(extended_type.span())
                            .with_message(format!("...because it is {extended_kind_prefix} {extended_kind_str}, not an interface")))
                        .with_annotation(Annotation::secondary(extended_class_span)
                            .with_message(format!("`{extended_name}` is defined as {extended_kind_prefix} {extended_kind_str} here")))
                        .with_note("In PHP, an interface can only extend other interfaces.")
                        .with_help(format!("To resolve this, change `{extended_name}` to be an interface, or change `{using_name}` to a class if you intended to extend a class.")),
                );

                continue;
            }

            if extended_class_metadata.flags.is_enum_interface() && !class_like_metadata.flags.is_enum_interface() {
                context.collector.report_with_code(
                    IssueCode::InvalidExtend,
                    Issue::error(format!("Interface `{using_name}` cannot extend enum-interface `{extended_name}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message("This interface is not an `@enum-interface`..."))
                        .with_annotation(Annotation::secondary(extended_type.span()).with_message("...but it extends an `@enum-interface`"))
                        .with_note("An interface marked with `@enum-interface` can only be extended by other interfaces that are also marked with `@enum-interface`.")
                        .with_help(format!("To resolve this, add the `@enum-interface` PHPDoc tag to `{using_name}`, or extend a regular, non-enum interface.")),
                );
            }
        }

        if class_like_metadata.kind.is_class() {
            if !extended_class_metadata.kind.is_class() {
                context.collector.report_with_code(
                    IssueCode::InvalidExtend,
                    Issue::error(format!("Class `{using_name}` cannot extend non-class type `{extended_name}`"))
                        .with_annotation(Annotation::primary(extended_type.span()).with_message(format!(
                            "...because it is {extended_kind_prefix} {extended_kind_str}, not a class"
                        )))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!(
                            "`{extended_name}` is defined as {extended_kind_prefix} {extended_kind_str} here"
                        )))
                        .with_note("In PHP, a class can only extend another class.")
                        .with_help("To inherit from an interface, use `implements`. To use a trait, use `use`."),
                );

                continue;
            }

            if extended_class_metadata.flags.is_final() {
                context.collector.report_with_code(
                    IssueCode::ExtendFinalClass,
                    Issue::error(format!("Class `{using_name}` cannot extend final class `{extended_name}`"))
                        .with_annotation(Annotation::primary(extended_type.span()).with_message("This inheritance is not allowed"))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("`{extended_name}` is declared 'final' here")))
                        .with_note("A class marked as `final` cannot be extended by any other class.")
                        .with_help(format!("To resolve this, either remove the `final` keyword from `{extended_name}`, or choose a different class to extend.")),
                );
            }

            if extended_class_metadata.flags.is_readonly() && !class_like_metadata.flags.is_readonly() {
                context.collector.report_with_code(
                    IssueCode::InvalidExtend,
                    Issue::error(format!("Non-readonly class `{using_name}` cannot extend readonly class `{extended_name}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message("This class is not `readonly`..."))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("...but it extends `{extended_name}`, which is `readonly`")))
                        .with_note("A `readonly` class can only be extended by another `readonly` class.")
                        .with_help(format!("To resolve this, either make the `{using_name}` class `readonly`, or extend a different, non-readonly class.")),
                );
            } else if !extended_class_metadata.flags.is_readonly() && class_like_metadata.flags.is_readonly() {
                context.collector.report_with_code(
                    IssueCode::InvalidExtend,
                    Issue::error(format!("Readonly class `{using_name}` cannot extend non-readonly class `{extended_name}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message("This class is `readonly`..."))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("...but it extends `{extended_name}`, which is not `readonly`")))
                        .with_note("A non-`readonly` class can only be extended by another non-`readonly` class.")
                        .with_help(format!("To resolve this, either make the `{using_name}` class non-`readonly`, or extend a different, readonly class.")),
                );
            }

            if let Some(required_interface) =
                class_like_metadata.get_missing_required_interface(extended_class_metadata)
            {
                context.collector.report_with_code(
                    IssueCode::MissingRequiredInterface,
                    Issue::error(format!("Class `{using_name}` must implement required interface `{required_interface}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message(format!("...because its parent `{extended_name}` requires it")))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message("Requirement declared here (likely via `@require-implements`)"))
                        .with_note("When a class uses `@require-implements`, all of its concrete child classes must implement the specified interface.")
                        .with_help(format!("Add `implements {required_interface}` to the `{using_name}` definition, or declare `{using_name}` as `abstract`.")),
                );
            }

            if !class_like_metadata.is_permitted_to_inherit(extended_class_metadata) {
                context.collector.report_with_code(
                    IssueCode::InvalidExtend,
                    Issue::error(format!("Class `{using_name}` is not permitted to extend `{extended_name}`"))
                        .with_annotation(Annotation::primary(extended_type.span()).with_message("This inheritance is restricted"))
                        .with_annotation(Annotation::secondary(extended_class_span)
                            .with_message(format!("The `@inheritors` annotation on this class does not include `{using_name}`")))
                        .with_note("The `@inheritors` annotation on a class or interface restricts which types are allowed to extend it.")
                        .with_help(format!("To allow this, add `{using_name}` to the list in the `@inheritors` PHPDoc tag for `{extended_name}`.")),
                );
            }

            let actual_parameters_count = class_like_metadata
                .template_type_extends_count
                .get(&extended_class_metadata.name)
                .copied()
                .unwrap_or(0);

            check_template_parameters(
                context,
                class_like_metadata,
                extended_class_metadata,
                actual_parameters_count,
                InheritanceKind::Extends(extended_type.span()),
            );
        }
    }
}

fn check_class_like_implements<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    implements_ast: Option<&Implements<'arena>>,
) {
    // This check only applies to classes and enums, which can use `implements`.
    if !class_like_metadata.kind.is_class() && !class_like_metadata.kind.is_enum() {
        // A separate check in the semantic analyzer will catch `implements` on an invalid type like a trait or interface.
        return;
    }

    let Some(implements) = implements_ast else {
        return;
    };

    let using_kind_str = class_like_metadata.kind.as_str();
    let using_kind_capitalized =
        format!("{}{}", using_kind_str.chars().next().unwrap().to_uppercase(), &using_kind_str[1..]);
    let using_name = class_like_metadata.original_name;
    let using_class_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);

    for implemented_type in &implements.types {
        let implemented_type_str = context.resolved_names.get(&implemented_type);
        let implemented_interface_metadata = context.codebase.get_class_like(implemented_type_str);

        if let Some(implemented_metadata) = implemented_interface_metadata {
            let implemented_name = implemented_metadata.original_name;
            let implemented_kind_str = implemented_metadata.kind.as_str();
            let implemented_class_span = implemented_metadata.name_span.unwrap_or(implemented_metadata.span);
            let implemented_kind_prefix =
                if implemented_metadata.kind.is_class() || implemented_metadata.kind.is_trait() { "a" } else { "an" };

            if !implemented_metadata.kind.is_interface() {
                context.collector.report_with_code(
                    IssueCode::InvalidImplement,
                    Issue::error(format!("{using_kind_capitalized} `{using_name}` cannot implement non-interface type `{implemented_name}`"))
                        .with_annotation(Annotation::primary(implemented_type.span())
                            .with_message(format!("...because it is {implemented_kind_prefix} {implemented_kind_str}, not an interface")))
                        .with_annotation(Annotation::secondary(implemented_class_span)
                            .with_message(format!("`{implemented_name}` is defined as {implemented_kind_prefix} {implemented_kind_str} here")))
                        .with_note("The `implements` keyword is exclusively for implementing interfaces.")
                        .with_help("To inherit from a class, use `extends`. To use a trait, use `use`."),
                );

                continue;
            }

            if implemented_metadata.flags.is_enum_interface() && !class_like_metadata.kind.is_enum() {
                context.collector.report_with_code(
                    IssueCode::InvalidImplement,
                    Issue::error(format!("{using_kind_capitalized} `{using_name}` cannot implement enum-only interface `{implemented_name}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message(format!("This {using_kind_str} is not an enum...")))
                        .with_annotation(Annotation::secondary(implemented_type.span()).with_message("...but it implements an interface restricted to enums"))
                        .with_annotation(Annotation::secondary(implemented_class_span).with_message("This interface is marked with `@enum-interface` here"))
                        .with_note("An interface marked with `@enum-interface` can only be implemented by enums.")
                        .with_help(format!("To resolve this, either change `{using_name}` to be an enum, or implement a different, non-enum interface.")),
                );
            }

            if !class_like_metadata.is_permitted_to_inherit(implemented_metadata) {
                context.collector.report_with_code(
                    IssueCode::InvalidImplement,
                    Issue::error(format!("{using_kind_capitalized} `{using_name}` is not permitted to implement `{implemented_name}`"))
                         .with_annotation(Annotation::primary(implemented_type.span()).with_message("This implementation is restricted"))
                        .with_annotation(Annotation::secondary(implemented_class_span)
                            .with_message(format!("The `@inheritors` annotation on this interface does not include `{using_name}`")))
                        .with_note("The `@inheritors` annotation on an interface restricts which types are allowed to implement it.")
                        .with_help(format!("To allow this, add `{using_name}` to the list in the `@inheritors` PHPDoc tag for `{implemented_name}`.")),
                );
            }

            let actual_parameters_count = class_like_metadata
                .template_type_implements_count
                .get(&implemented_metadata.name)
                .copied()
                .unwrap_or(0);

            check_template_parameters(
                context,
                class_like_metadata,
                implemented_metadata,
                actual_parameters_count,
                InheritanceKind::Implements(implemented_type.span()),
            );

            check_interface_method_signatures(context, class_like_metadata, implemented_metadata);
        } else {
            let implemented_name = implemented_type.value();

            context.collector.report_with_code(
                IssueCode::NonExistentClassLike,
                Issue::error(format!("{using_kind_capitalized} `{using_name}` cannot implement unknown type `{implemented_name}`"))
                    .with_annotation(Annotation::primary(implemented_type.span()).with_message("This type could not be found"))
                    .with_note("Mago could not find a definition for this interface. The `implements` keyword is for interfaces only.")
                    .with_help("Ensure the name is correct, including its namespace, and that it is properly defined and autoloadable."),
            );
        }
    }
}

fn check_class_like_use<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    trait_use: &TraitUse<'arena>,
) {
    let using_kind_str = class_like_metadata.kind.as_str();
    let using_kind_capitalized =
        format!("{}{}", using_kind_str.chars().next().unwrap().to_uppercase(), &using_kind_str[1..]);
    let using_name = class_like_metadata.original_name;
    let using_class_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);

    for used_type in &trait_use.trait_names {
        let used_type_str = context.resolved_names.get(&used_type);
        let used_trait_metadata = context.codebase.get_class_like(used_type_str);

        let Some(used_trait_metadata) = used_trait_metadata else {
            let used_name = used_type.value();

            context.collector.report_with_code(
                IssueCode::NonExistentClassLike,
                Issue::error(format!("{using_kind_capitalized} `{using_name}` cannot use unknown type `{used_name}`"))
                    .with_annotation(Annotation::primary(used_type.span()).with_message("This type could not be found"))
                    .with_note("Mago could not find a definition for this trait. The `use` keyword is for traits only.")
                    .with_help("Ensure the name is correct, including its namespace, and that it is properly defined and autoloadable."),
            );

            continue;
        };

        let used_name = used_trait_metadata.original_name;
        let used_kind_str = used_trait_metadata.kind.as_str();
        let used_kind_prefix =
            if used_trait_metadata.kind.is_class() || used_trait_metadata.kind.is_trait() { "a" } else { "an" };
        let used_class_span = used_trait_metadata.name_span.unwrap_or(used_trait_metadata.span);

        // Case: Using something that is not a trait.
        if !used_trait_metadata.kind.is_trait() {
            context.collector.report_with_code(
                IssueCode::InvalidTraitUse,
                Issue::error(format!(
                    "{using_kind_capitalized} `{using_name}` cannot use non-trait type `{used_name}`"
                ))
                .with_annotation(
                    Annotation::primary(used_type.span())
                        .with_message(format!("...because it is {used_kind_prefix} {used_kind_str}, not a trait")),
                )
                .with_annotation(
                    Annotation::secondary(used_class_span)
                        .with_message(format!("`{used_name}` is defined as {used_kind_prefix} {used_kind_str} here")),
                )
                .with_note("The `use` keyword is exclusively for including traits in classes, enums, or other traits.")
                .with_help("To inherit from a class, use `extends`. To implement an interface, use `implements`."),
            );

            continue;
        }

        if used_trait_metadata.flags.is_deprecated() {
            context.collector.report_with_code(
                IssueCode::DeprecatedTrait,
                Issue::error(format!("Use of deprecated trait `{used_name}` in `{using_name}`"))
                    .with_annotation(Annotation::primary(used_type.span()).with_message("This trait is marked as deprecated"))
                    .with_annotation(Annotation::secondary(used_class_span).with_message(format!("`{used_name}` was marked as deprecated here")))
                    .with_note("This trait is deprecated and may be removed in a future version, which would break the consuming type.")
                    .with_help("Consider refactoring to avoid using this trait, or consult its documentation for alternatives."),
            );
        }

        if let Some(required_interface) = class_like_metadata.get_missing_required_interface(used_trait_metadata) {
            context.collector.report_with_code(
                IssueCode::MissingRequiredInterface,
                Issue::error(format!("{using_kind_capitalized} `{using_name}` must implement required interface `{required_interface}`"))
                    .with_annotation(Annotation::primary(using_class_span).with_message(format!("...because the trait `{used_name}` requires it")))
                    .with_annotation(Annotation::secondary(used_type.span()).with_message(format!("The requirement is introduced by using `{used_name}` here")))
                    .with_note("When a trait uses `@require-implements`, any concrete class using that trait must implement the specified interface.")
                    .with_help(format!("Add `implements {required_interface}` to the `{using_name}` definition, or declare it as `abstract`.")),
            );
        }

        if let Some(required_class) = class_like_metadata.get_missing_required_extends(used_trait_metadata) {
            context.collector.report_with_code(
                IssueCode::MissingRequiredParent,
                Issue::error(format!(
                    "{using_kind_capitalized} `{using_name}` must extend required class `{required_class}`"
                ))
                .with_annotation(
                    Annotation::primary(using_class_span)
                        .with_message(format!("...because the trait `{used_name}` requires it")),
                )
                .with_annotation(
                    Annotation::secondary(used_type.span())
                        .with_message(format!("The requirement is introduced by using `{used_name}` here")),
                )
                .with_note(
                    "When a trait uses `@require-extends`, any class using that trait must extend the specified class.",
                )
                .with_help(format!(
                    "Add `extends {required_class}` to the `{using_name}` definition, or ensure it is a parent class."
                )),
            );
        }

        if !class_like_metadata.is_permitted_to_inherit(used_trait_metadata) {
            context.collector.report_with_code(
                IssueCode::InvalidTraitUse,
                Issue::error(format!(
                    "{using_kind_capitalized} `{using_name}` is not permitted to use trait `{used_name}`"
                ))
                .with_annotation(Annotation::primary(used_type.span()).with_message("This usage is restricted"))
                .with_annotation(Annotation::secondary(used_class_span).with_message(format!(
                    "The `@inheritors` annotation on this trait does not include `{using_name}`"
                )))
                .with_note("The `@inheritors` annotation on a trait restricts which types are allowed to use it.")
                .with_help(format!(
                    "To allow this, add `{using_name}` to the list in the `@inheritors` PHPDoc tag for `{used_name}`."
                )),
            );
        }

        check_template_parameters(
            context,
            class_like_metadata,
            used_trait_metadata,
            class_like_metadata.template_type_uses_count.get(&used_trait_metadata.name).copied().unwrap_or(0),
            InheritanceKind::Use(used_type.span()),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InheritanceKind {
    Extends(Span),
    Implements(Span),
    Use(Span),
}

impl HasSpan for InheritanceKind {
    fn span(&self) -> Span {
        match self {
            InheritanceKind::Extends(span) => *span,
            InheritanceKind::Implements(span) => *span,
            InheritanceKind::Use(span) => *span,
        }
    }
}

fn check_template_parameters<'ctx>(
    context: &mut Context<'ctx, '_>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    parent_metadata: &'ctx ClassLikeMetadata,
    actual_parameters_count: usize,
    inheritance: InheritanceKind,
) {
    let expected_parameters_count = parent_metadata.template_types.len();

    let class_name = class_like_metadata.original_name;
    let class_kind_str = class_like_metadata.kind.as_str();
    let parent_name = parent_metadata.original_name;
    let class_name_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);
    let parent_definition_span = parent_metadata.name_span.unwrap_or(parent_metadata.span);
    let primary_annotation_span = inheritance.span();
    let (inheritance_keyword, inheritance_tag) = match inheritance {
        InheritanceKind::Extends(_) => ("extends", "@extends"),
        InheritanceKind::Implements(_) => ("implements", "@implements"),
        InheritanceKind::Use(_) => ("uses", "@use"),
    };

    if expected_parameters_count > actual_parameters_count {
        let issue = Issue::error(format!(
            "Too few template arguments for `{parent_name}`: expected {expected_parameters_count}, but found {actual_parameters_count}."
        ))
        .with_annotation(
            Annotation::primary(primary_annotation_span)
                .with_message(format!("Too few template arguments provided here when `{class_name}` {inheritance_keyword} `{parent_name}`")),
        )
        .with_annotation(
            Annotation::secondary(class_name_span)
                .with_message(format!("Declaration of `{class_name}` is here")),
        )
        .with_annotation(
            Annotation::secondary(parent_definition_span)
                .with_message(format!("`{parent_name}` is defined with {expected_parameters_count} template parameters")),
        )
        .with_help(format!("Provide all {expected_parameters_count} required template arguments in the `{inheritance_tag}` docblock tag for `{class_name}`."));

        context.collector.report_with_code(IssueCode::MissingTemplateParameter, issue);
    } else if expected_parameters_count < actual_parameters_count {
        let issue = Issue::error(format!(
            "Too many template arguments for `{parent_name}`: expected {expected_parameters_count}, but found {actual_parameters_count}."
        ))
        .with_annotation(
            Annotation::primary(primary_annotation_span)
                .with_message(format!("Too many template arguments provided here when `{class_name}` {inheritance_keyword} `{parent_name}`")),
        )
        .with_annotation(
            Annotation::secondary(class_name_span)
                .with_message(format!("Declaration of `{class_name}` is here")),
        )
        .with_annotation(
            Annotation::secondary(parent_definition_span)
                .with_message(format!("`{parent_name}` is defined with {expected_parameters_count} template parameters")),
        )
        .with_help(format!("Remove the extra arguments from the `{inheritance_tag}` tag for `{class_name}`."));

        context.collector.report_with_code(IssueCode::ExcessTemplateParameter, issue);
    }

    let own_template_parameters_len = class_like_metadata.template_types.len();
    if parent_metadata.flags.has_consistent_templates() && own_template_parameters_len != expected_parameters_count {
        context.collector.report_with_code(
            IssueCode::InconsistentTemplate,
            Issue::error(format!(
                "Template parameter count mismatch: `{class_name}` must have {expected_parameters_count} template parameters to match `{parent_name}`."
            ))
            .with_annotation(Annotation::primary(class_name_span).with_message(format!("This {class_kind_str} defines {own_template_parameters_len} template parameters...")))
            .with_annotation(Annotation::secondary(parent_definition_span).with_message(format!("...but parent `{parent_name}` is marked `@consistent-templates` and expects {expected_parameters_count}.")))
            .with_help("Ensure the number of template parameters on this {class_kind_str} matches its parent."),
        );
    }

    if expected_parameters_count > 0
        && let Some(extended_parameters) = class_like_metadata.template_extended_parameters.get(&parent_metadata.name)
    {
        let mut i = 0;
        let mut previous_extended_types: IndexMap<Atom, Vec<(GenericParent, TUnion)>, RandomState> =
            IndexMap::default();

        for (template_name, _) in &parent_metadata.template_types {
            if let Some(extended_type) = extended_parameters.get(template_name) {
                previous_extended_types
                    .entry(*template_name)
                    .or_default()
                    .push((GenericParent::ClassLike(parent_metadata.name), extended_type.clone()));
            }
        }

        for (template_name, template_type_map) in &parent_metadata.template_types {
            let Some(mut extended_type) = extended_parameters.get(template_name).cloned() else {
                i += 1;
                continue;
            };

            let Some(mut template_type) = template_type_map.last().map(|(_, template_type)| template_type).cloned()
            else {
                i += 1;
                continue;
            };

            expand_union(
                context.codebase,
                &mut extended_type,
                &TypeExpansionOptions { self_class: Some(class_like_metadata.original_name), ..Default::default() },
            );

            expand_union(
                context.codebase,
                &mut template_type,
                &TypeExpansionOptions { self_class: Some(class_like_metadata.original_name), ..Default::default() },
            );

            let extended_type_str = extended_type.get_id();

            if parent_metadata
                .template_variance
                .get(&i)
                .is_some_and(mago_codex::ttype::template::variance::Variance::is_invariant)
            {
                for extended_type_atomic in extended_type.types.as_ref() {
                    let TAtomic::GenericParameter(generic_parameter) = extended_type_atomic else {
                        continue;
                    };

                    let Some(local_offset) = class_like_metadata
                        .template_types
                        .iter()
                        .position(|(name, _)| *name == generic_parameter.parameter_name)
                    else {
                        continue;
                    };

                    if class_like_metadata
                        .template_variance
                        .get(&local_offset)
                        .is_some_and(mago_codex::ttype::template::variance::Variance::is_covariant)
                    {
                        let child_template_name = generic_parameter.parameter_name;

                        context.collector.report_with_code(
                            IssueCode::InvalidTemplateParameter,
                            Issue::error("Invalid template variance: cannot use a covariant template to satisfy an invariant one.")
                                .with_annotation(Annotation::primary(class_name_span).with_message(format!("In the definition of `{class_name}`")))
                                .with_note(format!("The parent `{parent_name}` defines template `{template_name}` as invariant (`@template`)."))
                                .with_note(format!("But it is being satisfied by the covariant template `{child_template_name}` (`@template-covariant`) from `{class_name}`."))
                                .with_help("Make the child template parameter invariant as well (`@template`), or change the parent's variance if appropriate."),
                        );
                    }
                }
            }

            if parent_metadata.flags.has_consistent_templates() {
                for extended_type_atomic in extended_type.types.as_ref() {
                    let extended_as_template = extended_type_atomic.get_generic_parameter_name();
                    if extended_as_template.is_none() {
                        context.collector.report_with_code(
                            IssueCode::InvalidTemplateParameter,
                            Issue::error("Inconsistent template: expected a template parameter, but found a concrete type.")
                                .with_annotation(Annotation::primary(parent_definition_span).with_message(format!(
                                    "Expected a template parameter, but got `{}`",
                                    extended_type.get_id(),
                                )))
                                .with_note(format!("Because `{parent_name}` is marked `@consistent-templates`, its template parameters must be extended with other template parameters, not concrete types."))
                                .with_help(format!("Change this to a template parameter defined on `{class_name}`.")),
                        );
                    } else if let Some(child_template_name) = extended_as_template
                        && let Some(child_template_map) = class_like_metadata.get_template_type(&child_template_name)
                        && let Some((_, child_template_type)) = child_template_map.last()
                        && child_template_type.get_id() != template_type.get_id()
                    {
                        context.collector.report_with_code(
                            IssueCode::InvalidTemplateParameter,
                            Issue::error("Inconsistent template: template parameter constraints do not match.")
                                .with_annotation(Annotation::primary(class_name_span).with_message(format!("This template parameter has constraint `{}`...", child_template_type.get_id())))
                                .with_annotation(Annotation::secondary(parent_definition_span).with_message(format!("...but parent `{parent_name}` requires a constraint of `{}` for this template.", template_type.get_id())))
                                .with_note(format!("Because `{parent_name}` is marked `@consistent-templates`, the constraints of its template parameters must be identical in child classes."))
                                .with_help("Adjust the constraint on the child template parameter to match the parent's."),
                        );
                    }
                }
            }

            if template_type.is_mixed() {
                previous_extended_types
                    .entry(*template_name)
                    .or_default()
                    .push((GenericParent::ClassLike(parent_metadata.name), extended_type));
            } else {
                let mut template_result = TemplateResult::new(previous_extended_types.clone(), Default::default());
                let mut replaced_template_type = standin_type_replacer::replace(
                    &template_type,
                    &mut template_result,
                    context.codebase,
                    &None,
                    None,
                    None,
                    StandinOptions::default(),
                );

                expand_union(
                    context.codebase,
                    &mut replaced_template_type,
                    &TypeExpansionOptions { self_class: Some(class_like_metadata.original_name), ..Default::default() },
                );

                if is_type_compatible(context.codebase, &extended_type, &replaced_template_type) {
                    previous_extended_types
                        .entry(*template_name)
                        .or_default()
                        .push((GenericParent::ClassLike(parent_metadata.name), extended_type));
                } else {
                    let replaced_type_str = replaced_template_type.get_id();

                    context.collector.report_with_code(
                        IssueCode::InvalidTemplateParameter,
                        Issue::error(format!(
                            "Template argument for `{parent_name}` is not compatible with its constraint."
                        ))
                        .with_annotation(
                            Annotation::primary(class_name_span)
                                .with_message(format!("In the definition of `{class_name}`")),
                        )
                        .with_note(format!("The type `{extended_type_str}` provided for template `{template_name}`..."))
                        .with_note(format!(
                            "...does not satisfy the required constraint of `{replaced_type_str}` from `{parent_name}`."
                        ))
                        .with_help("Change the provided type to be compatible with the template constraint."),
                    );
                }
            }

            i += 1;
        }
    }
}

/// Checks if this is the same method that was inherited (not overridden).
/// Example: `StringBox` extends Box and inherits `Box::setValue` without overriding it.
#[inline]
fn should_skip_same_method(appearing_fqcn: &str, overridden_fqcn: &str) -> bool {
    ascii_lowercase_atom(appearing_fqcn) == ascii_lowercase_atom(overridden_fqcn)
}

/// Checks if this is a trait-to-trait abstract method conflict that should be handled
/// by `check_trait_method_conflicts` instead of here.
///
/// We skip when BOTH methods are:
/// - From different traits
/// - Both are abstract
/// - Both traits are used by the current class
#[inline]
fn should_skip_trait_to_trait_conflict(
    appearing_class: &ClassLikeMetadata,
    appearing_method: &FunctionLikeMetadata,
    overridden_class: &ClassLikeMetadata,
    overridden_method: &FunctionLikeMetadata,
    class_like_metadata: &ClassLikeMetadata,
) -> bool {
    if !appearing_class.kind.is_trait() || !overridden_class.kind.is_trait() {
        return false;
    }

    let appearing_is_abstract = appearing_method.method_metadata.as_ref().is_some_and(|m| m.is_abstract);
    let overridden_is_abstract = overridden_method.method_metadata.as_ref().is_some_and(|m| m.is_abstract);

    if !appearing_is_abstract || !overridden_is_abstract {
        return false;
    }

    let appearing_lowercase = ascii_lowercase_atom(&appearing_class.name);
    let overridden_lowercase = ascii_lowercase_atom(&overridden_class.name);

    if !class_like_metadata.used_traits.contains(&appearing_lowercase)
        || !class_like_metadata.used_traits.contains(&overridden_lowercase)
    {
        return false;
    }

    appearing_lowercase != overridden_lowercase
}

/// Checks if this is an enum implementing BackedEnum/UnitEnum, which is allowed
/// to narrow the method signatures (e.g., `from(string)` instead of `from(int|string)`).
#[inline]
fn should_skip_enum_builtin_interface(class_like_metadata: &ClassLikeMetadata, interface_fqcn: &str) -> bool {
    class_like_metadata.kind.is_enum() && (interface_fqcn == "backedenum" || interface_fqcn == "unitenum")
}

fn check_abstract_method_signatures<'ctx>(
    context: &mut Context<'ctx, '_>,
    class_like_metadata: &'ctx ClassLikeMetadata,
) {
    for (method_name_atom, overridden_method_ids) in &class_like_metadata.overridden_method_ids {
        let method_name_str = method_name_atom.as_ref();

        let Some(declaring_method_id) = class_like_metadata.declaring_method_ids.get(method_name_atom) else {
            continue;
        };

        let declaring_fqcn_str = declaring_method_id.get_class_name().as_ref();
        let declaring_method_opt = context.codebase.get_method(declaring_fqcn_str, method_name_str);

        let (method_fqcn_str, appearing_method) = if let Some(method) = declaring_method_opt {
            (declaring_fqcn_str, method)
        } else if let Some(appearing_method_id) = class_like_metadata.appearing_method_ids.get(method_name_atom) {
            let appearing_fqcn_str = appearing_method_id.get_class_name().as_ref();
            let Some(method) = context.codebase.get_method(appearing_fqcn_str, method_name_str) else {
                continue;
            };

            (appearing_fqcn_str, method)
        } else {
            continue;
        };

        for (parent_fqcn, parent_declaring_method_id) in overridden_method_ids {
            let parent_fqcn_str = parent_fqcn.as_ref();

            let declaring_class_name = parent_declaring_method_id.get_class_name();
            let declaring_class_name_str = declaring_class_name.as_ref();

            if should_skip_same_method(method_fqcn_str, parent_fqcn_str) {
                continue;
            }

            let Some(overridden_method) =
                context.codebase.get_declaring_method(declaring_class_name_str, method_name_str)
            else {
                continue;
            };

            let Some(overridden_class) = context.codebase.get_class_like(declaring_class_name_str) else {
                continue;
            };

            if !class_like_metadata.kind.is_interface()
                && overridden_class.kind.is_interface()
                && class_like_metadata.direct_parent_interfaces.contains(&overridden_class.name)
            {
                continue;
            }

            let Some(appearing_class) = context.codebase.get_class_like(method_fqcn_str) else {
                continue;
            };

            if should_skip_trait_to_trait_conflict(
                appearing_class,
                appearing_method,
                overridden_class,
                overridden_method,
                class_like_metadata,
            ) {
                continue;
            }

            if should_skip_enum_builtin_interface(class_like_metadata, declaring_class_name_str) {
                continue;
            }

            let substituted_overridden_method =
                get_substituted_method(overridden_method, class_like_metadata, *declaring_class_name, context.codebase);

            let issues = method_signature::validate_method_signature_compatibility(
                context.codebase,
                class_like_metadata.name,
                appearing_method,
                &substituted_overridden_method,
            );

            if issues.is_empty() {
                continue;
            }

            let error_span = if appearing_class.kind.is_trait() {
                class_like_metadata.name_span.unwrap_or(class_like_metadata.span)
            } else {
                appearing_method.name_span.unwrap_or(appearing_method.span)
            };

            for incompatibility in issues {
                report_signature_compatibility_issue(
                    context,
                    class_like_metadata,
                    overridden_class,
                    *method_name_atom,
                    appearing_method,
                    incompatibility,
                    error_span,
                );
            }
        }
    }
}

fn check_trait_method_conflicts<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    members: &'ast [ClassLikeMember<'arena>],
) {
    let mut trait_uses: Vec<(&'ast TraitUse<'arena>, Vec<Atom>)> = Vec::new();

    for member in members {
        if let ClassLikeMember::TraitUse(trait_use) = member {
            let mut trait_names = Vec::new();
            for trait_name_id in &trait_use.trait_names {
                let (trait_fqcn, _) = context.scope.resolve(NameKind::Default, trait_name_id.value());
                trait_names.push(Atom::from(trait_fqcn.as_str()));
            }
            trait_uses.push((trait_use, trait_names));
        }
    }

    for i in 0..trait_uses.len() {
        let (first_trait_use, first_traits) = &trait_uses[i];

        for k in 0..first_traits.len() {
            for l in (k + 1)..first_traits.len() {
                let first_trait_fqcn = &first_traits[k];
                let second_trait_fqcn = &first_traits[l];

                let Some(first_trait_metadata) = context.codebase.get_class_like(first_trait_fqcn.as_ref()) else {
                    continue;
                };
                let Some(second_trait_metadata) = context.codebase.get_class_like(second_trait_fqcn.as_ref()) else {
                    continue;
                };

                for (method_name, first_method_id) in &first_trait_metadata.declaring_method_ids {
                    if let Some(second_method_id) = second_trait_metadata.declaring_method_ids.get(method_name) {
                        let first_method_str = method_name.as_ref();
                        let Some(first_method) = context
                            .codebase
                            .get_declaring_method(first_method_id.get_class_name().as_ref(), first_method_str)
                        else {
                            continue;
                        };
                        let Some(second_method) = context
                            .codebase
                            .get_declaring_method(second_method_id.get_class_name().as_ref(), first_method_str)
                        else {
                            continue;
                        };

                        let first_is_abstract = first_method.method_metadata.as_ref().is_some_and(|m| m.is_abstract);
                        let second_is_abstract = second_method.method_metadata.as_ref().is_some_and(|m| m.is_abstract);

                        if first_is_abstract || second_is_abstract {
                            let issues = method_signature::validate_method_signature_compatibility(
                                context.codebase,
                                class_like_metadata.name,
                                second_method,
                                first_method,
                            );

                            for incompatibility in issues {
                                let trait_use_span = first_trait_use.span();

                                report_signature_compatibility_issue(
                                    context,
                                    class_like_metadata,
                                    first_trait_metadata,
                                    *method_name,
                                    second_method,
                                    incompatibility,
                                    trait_use_span,
                                );
                            }
                        }
                    }
                }
            }
        }

        for (second_trait_use, second_traits) in trait_uses.iter().skip(i + 1) {
            for first_trait_fqcn in first_traits {
                let Some(first_trait_metadata) = context.codebase.get_class_like(first_trait_fqcn.as_ref()) else {
                    continue;
                };

                for second_trait_fqcn in second_traits {
                    let Some(second_trait_metadata) = context.codebase.get_class_like(second_trait_fqcn.as_ref())
                    else {
                        continue;
                    };

                    for (method_name, first_method_id) in &first_trait_metadata.declaring_method_ids {
                        if let Some(second_method_id) = second_trait_metadata.declaring_method_ids.get(method_name) {
                            let first_method_str = method_name.as_ref();
                            let Some(first_method) = context
                                .codebase
                                .get_declaring_method(first_method_id.get_class_name().as_ref(), first_method_str)
                            else {
                                continue;
                            };
                            let Some(second_method) = context
                                .codebase
                                .get_declaring_method(second_method_id.get_class_name().as_ref(), first_method_str)
                            else {
                                continue;
                            };

                            let first_is_abstract =
                                first_method.method_metadata.as_ref().is_some_and(|m| m.is_abstract);
                            let second_is_abstract =
                                second_method.method_metadata.as_ref().is_some_and(|m| m.is_abstract);

                            if first_is_abstract || second_is_abstract {
                                let issues = method_signature::validate_method_signature_compatibility(
                                    context.codebase,
                                    class_like_metadata.name,
                                    second_method,
                                    first_method,
                                );

                                for incompatibility in issues {
                                    let second_trait_use_span = second_trait_use.span();

                                    report_signature_compatibility_issue(
                                        context,
                                        class_like_metadata,
                                        first_trait_metadata,
                                        *method_name,
                                        second_method,
                                        incompatibility,
                                        second_trait_use_span,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_trait_property_conflicts<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    members: &'ast [ClassLikeMember<'arena>],
) {
    let mut trait_uses: Vec<(&'ast TraitUse<'arena>, Vec<Atom>)> = Vec::new();

    for member in members {
        if let ClassLikeMember::TraitUse(trait_use) = member {
            let mut trait_names = Vec::new();
            for trait_name_id in &trait_use.trait_names {
                let (trait_fqcn, _) = context.scope.resolve(NameKind::Default, trait_name_id.value());
                trait_names.push(Atom::from(trait_fqcn.as_str()));
            }
            trait_uses.push((trait_use, trait_names));
        }
    }

    let mut class_properties: IndexMap<Atom, &PropertyMetadata> = IndexMap::new();
    for (property_name, property_metadata) in class_like_metadata.properties.iter().sorted_by_key(|(k, _)| *k) {
        if let Some(declaring_class) = class_like_metadata.declaring_property_ids.get(property_name)
            && declaring_class == &class_like_metadata.name
        {
            class_properties.insert(*property_name, property_metadata);
        }
    }

    for i in 0..trait_uses.len() {
        let (first_trait_use, first_traits) = &trait_uses[i];

        for k in 0..first_traits.len() {
            for l in (k + 1)..first_traits.len() {
                let first_trait_fqcn = &first_traits[k];
                let second_trait_fqcn = &first_traits[l];

                let Some(first_trait_metadata) = context.codebase.get_class_like(first_trait_fqcn.as_ref()) else {
                    continue;
                };
                let Some(second_trait_metadata) = context.codebase.get_class_like(second_trait_fqcn.as_ref()) else {
                    continue;
                };

                for (property_name, first_property) in first_trait_metadata.properties.iter().sorted_by_key(|(k, _)| *k)
                {
                    let Some(second_property) = second_trait_metadata.properties.get(property_name) else {
                        continue;
                    };

                    if properties_are_compatible(first_property, second_property) {
                        continue;
                    }

                    report_trait_property_conflict(
                        context,
                        class_like_metadata.name,
                        *property_name,
                        *first_trait_fqcn,
                        *second_trait_fqcn,
                        first_trait_use.span(),
                        first_property,
                        second_property,
                    );
                }
            }
        }

        for (second_trait_use, second_traits) in trait_uses.iter().skip(i + 1) {
            for first_trait_fqcn in first_traits {
                let Some(first_trait_metadata) = context.codebase.get_class_like(first_trait_fqcn.as_ref()) else {
                    continue;
                };

                for second_trait_fqcn in second_traits {
                    let Some(second_trait_metadata) = context.codebase.get_class_like(second_trait_fqcn.as_ref())
                    else {
                        continue;
                    };

                    for (property_name, first_property) in
                        first_trait_metadata.properties.iter().sorted_by_key(|(k, _)| *k)
                    {
                        let Some(second_property) = second_trait_metadata.properties.get(property_name) else {
                            continue;
                        };

                        if properties_are_compatible(first_property, second_property) {
                            continue;
                        }

                        report_trait_property_conflict(
                            context,
                            class_like_metadata.name,
                            *property_name,
                            *first_trait_fqcn,
                            *second_trait_fqcn,
                            second_trait_use.span(),
                            first_property,
                            second_property,
                        );
                    }
                }
            }
        }

        for first_trait_fqcn in first_traits {
            let Some(first_trait_metadata) = context.codebase.get_class_like(first_trait_fqcn.as_ref()) else {
                continue;
            };

            for (property_name, trait_property) in first_trait_metadata.properties.iter().sorted_by_key(|(k, _)| *k) {
                let Some(class_property) = class_properties.get(property_name) else {
                    continue;
                };

                if properties_are_compatible(trait_property, class_property) {
                    continue;
                }

                let conflict_span = members
                    .iter()
                    .find_map(|member| {
                        if let ClassLikeMember::Property(prop) = member {
                            match prop {
                                Property::Plain(plain_prop) => {
                                    for item in &plain_prop.items {
                                        let var_name = Atom::from(item.variable().name);
                                        if var_name == *property_name {
                                            return Some(prop.span());
                                        }
                                    }
                                }
                                Property::Hooked(hooked_prop) => {
                                    let var_name = Atom::from(hooked_prop.item.variable().name);
                                    if var_name == *property_name {
                                        return Some(prop.span());
                                    }
                                }
                            }
                        }
                        None
                    })
                    .unwrap_or_else(|| first_trait_use.span());

                report_trait_property_conflict(
                    context,
                    class_like_metadata.name,
                    *property_name,
                    *first_trait_fqcn,
                    class_like_metadata.name,
                    conflict_span,
                    trait_property,
                    class_property,
                );
            }
        }
    }
}

fn properties_are_compatible(prop1: &PropertyMetadata, prop2: &PropertyMetadata) -> bool {
    // PHP 8.4: Conflict resolution between hooked properties is not supported
    if !prop1.hooks.is_empty() || !prop2.hooks.is_empty() {
        return false;
    }

    if prop1.read_visibility != prop2.read_visibility {
        return false;
    }
    if prop1.write_visibility != prop2.write_visibility {
        return false;
    }

    if prop1.flags.is_static() != prop2.flags.is_static() {
        return false;
    }

    if prop1.flags.is_readonly() != prop2.flags.is_readonly() {
        return false;
    }

    match (&prop1.type_declaration_metadata, &prop2.type_declaration_metadata) {
        (Some(t1), Some(t2)) => {
            if t1.type_union.get_id() != t2.type_union.get_id() {
                return false;
            }
        }
        (None, None) => {}
        _ => return false,
    }

    match (&prop1.default_type_metadata, &prop2.default_type_metadata) {
        (Some(d1), Some(d2)) => {
            if d1.type_union.get_id() != d2.type_union.get_id() {
                return false;
            }
        }
        (None, None) => {}
        _ => return false,
    }

    true
}

/// Check if two properties are compatible, returns Err with specific conflict type if not
fn check_property_compatibility(prop1: &PropertyMetadata, prop2: &PropertyMetadata) -> Result<(), PropertyConflict> {
    // PHP 8.4: Conflict resolution between hooked properties is not supported
    if !prop1.hooks.is_empty() || !prop2.hooks.is_empty() {
        return Err(PropertyConflict::HookedProperty);
    }

    if prop1.read_visibility != prop2.read_visibility || prop1.write_visibility != prop2.write_visibility {
        return Err(PropertyConflict::Visibility(
            prop1.read_visibility,
            prop1.write_visibility,
            prop2.read_visibility,
            prop2.write_visibility,
        ));
    }

    if prop1.flags.is_static() != prop2.flags.is_static() {
        return Err(PropertyConflict::Static(prop1.flags.is_static(), prop2.flags.is_static()));
    }

    if prop1.flags.is_readonly() != prop2.flags.is_readonly() {
        return Err(PropertyConflict::Readonly(prop1.flags.is_readonly(), prop2.flags.is_readonly()));
    }

    match (&prop1.type_declaration_metadata, &prop2.type_declaration_metadata) {
        (Some(t1), Some(t2)) => {
            if t1.type_union.get_id() != t2.type_union.get_id() {
                return Err(PropertyConflict::Type(
                    Some(format!("{:?}", t1.type_union)),
                    Some(format!("{:?}", t2.type_union)),
                ));
            }
        }
        (Some(t1), None) => {
            return Err(PropertyConflict::Type(Some(format!("{:?}", t1.type_union)), None));
        }
        (None, Some(t2)) => {
            return Err(PropertyConflict::Type(None, Some(format!("{:?}", t2.type_union))));
        }
        (None, None) => {}
    }

    match (&prop1.default_type_metadata, &prop2.default_type_metadata) {
        (Some(d1), Some(d2)) => {
            if d1.type_union.get_id() != d2.type_union.get_id() {
                return Err(PropertyConflict::Default(
                    Some(format!("{:?}", d1.type_union)),
                    Some(format!("{:?}", d2.type_union)),
                ));
            }
        }
        (Some(d1), None) => {
            return Err(PropertyConflict::Default(Some(format!("{:?}", d1.type_union)), None));
        }
        (None, Some(d2)) => {
            return Err(PropertyConflict::Default(None, Some(format!("{:?}", d2.type_union))));
        }
        (None, None) => {}
    }

    Ok(())
}

fn report_trait_property_conflict(
    context: &mut Context,
    class_name: Atom,
    property_name: Atom,
    trait1_name: Atom,
    trait2_name: Atom,
    conflict_span: Span,
    prop1: &PropertyMetadata,
    prop2: &PropertyMetadata,
) {
    let conflict = match check_property_compatibility(prop1, prop2) {
        Ok(()) => {
            PropertyConflict::Type(None, None) // Dummy value
        }
        Err(conflict) => conflict,
    };

    let conflict_description = conflict.describe();
    let issue_code = conflict.get_issue_code();

    context.collector.report_with_code(
        issue_code,
        Issue::error(format!(
            "Property `{property_name}` is defined differently in `{trait1_name}` and `{trait2_name}` used by `{class_name}`: {conflict_description}"
        ))
        .with_annotation(Annotation::primary(conflict_span).with_message("Conflicting property definitions"))
        .with_note(format!("In PHP, this will cause a fatal error: '{trait1_name} and {trait2_name} define the same property ({property_name}) in the composition of {class_name}. However, the definition differs and is considered incompatible.'"))
        .with_help("Ensure both sources define the property identically (same visibility, type, default value, and modifiers), or use only one source."),
    );
}

/// Apply template parameter substitution to a method's parameter and return types
///
/// For example, if interface has `K` and `V` template parameters, and the implementation
/// maps them to `TKey` and `TValue`, this function replaces all occurrences of `K` with `TKey`
/// and `V` with `TValue` in the method signature.
///
/// Gets the substituted method by applying template parameter mapping from the class.
/// Returns the original method if no template substitution is needed.
#[inline]
fn get_substituted_method(
    method: &FunctionLikeMetadata,
    class_like_metadata: &ClassLikeMetadata,
    parent_class_name: Atom,
    codebase: &mago_codex::metadata::CodebaseMetadata,
) -> FunctionLikeMetadata {
    let template_mapping =
        class_like_metadata.template_extended_parameters.get(&parent_class_name).cloned().unwrap_or_default();

    if template_mapping.is_empty() {
        method.clone()
    } else {
        let mut template_result = TemplateResult::default();
        for (template_name, concrete_type) in template_mapping {
            template_result.add_lower_bound(template_name, GenericParent::ClassLike(parent_class_name), concrete_type);
        }

        apply_template_substitution_to_method(method, &template_result, codebase)
    }
}

fn apply_template_substitution_to_method(
    method: &FunctionLikeMetadata,
    template_result: &TemplateResult,
    codebase: &mago_codex::metadata::CodebaseMetadata,
) -> FunctionLikeMetadata {
    let mut substituted_method = method.clone();

    for param in &mut substituted_method.parameters {
        if let Some(type_metadata) = &mut param.type_metadata {
            type_metadata.type_union =
                inferred_type_replacer::replace(&type_metadata.type_union, template_result, codebase);
        }
    }

    if let Some(return_type) = &mut substituted_method.return_type_declaration_metadata {
        return_type.type_union = inferred_type_replacer::replace(&return_type.type_union, template_result, codebase);
    }

    substituted_method
}

fn check_interface_method_signatures<'ctx>(
    context: &mut Context<'ctx, '_>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    interface_metadata: &'ctx ClassLikeMetadata,
) {
    let interface_fqcn_str: &str = interface_metadata.name.as_ref();
    if should_skip_enum_builtin_interface(class_like_metadata, interface_fqcn_str) {
        return;
    }

    for (method_name_atom, interface_method_id) in &interface_metadata.declaring_method_ids {
        let method_name_str = method_name_atom.as_ref();
        let interface_fqcn_str = interface_method_id.get_class_name().as_ref();

        let Some(interface_method) = context.codebase.get_declaring_method(interface_fqcn_str, method_name_str) else {
            continue;
        };

        let Some(class_method_id) = class_like_metadata.declaring_method_ids.get(method_name_atom) else {
            continue;
        };

        let class_fqcn_str = class_method_id.get_class_name().as_ref();
        let Some(class_method) = context.codebase.get_declaring_method(class_fqcn_str, method_name_str) else {
            continue;
        };

        if should_skip_same_method(class_fqcn_str, interface_fqcn_str) {
            continue;
        }

        let substituted_interface_method = get_substituted_method(
            interface_method,
            class_like_metadata,
            *interface_method_id.get_class_name(),
            context.codebase,
        );

        let issues = method_signature::validate_method_signature_compatibility(
            context.codebase,
            class_like_metadata.name,
            class_method,
            &substituted_interface_method,
        );

        for incompatibility in issues {
            // Use the method span as primary location (where the issue actually is)
            let method_span = class_method.name_span.unwrap_or(class_method.span);

            // Get the actual declaring class for error reporting
            let declaring_class = context.codebase.get_class_like(interface_fqcn_str).unwrap_or(interface_metadata);

            report_signature_compatibility_issue(
                context,
                class_like_metadata,
                declaring_class,
                *method_name_atom,
                class_method,
                incompatibility,
                method_span,
            );
        }
    }
}

fn report_signature_compatibility_issue<'ctx>(
    context: &mut Context<'ctx, '_>,
    child_class: &'ctx ClassLikeMetadata,
    parent_class: &'ctx ClassLikeMetadata,
    method_name: Atom,
    parent_method: &FunctionLikeMetadata,
    incompatibility: SignatureCompatibilityIssue,
    primary_span: Span,
) {
    let child_name = child_class.original_name;
    let parent_name = parent_class.original_name;
    let child_class_span = child_class.name_span.unwrap_or(child_class.span);
    let parent_class_span = parent_class.name_span.unwrap_or(parent_class.span);

    use method_signature::SignatureCompatibilityIssue;

    match incompatibility {
        SignatureCompatibilityIssue::FinalMethodOverride => {
            context.collector.report_with_code(
                IssueCode::OverrideFinalMethod,
                Issue::error(format!("Cannot override final method `{parent_name}::{method_name}()`"))
                    .with_annotation(
                        Annotation::primary(primary_span).with_message("Attempting to override final method here"),
                    )
                    .with_annotation(
                        Annotation::secondary(parent_class_span)
                            .with_message(format!("Method `{parent_name}::{method_name}()` is declared as final")),
                    )
                    .with_annotation(
                        Annotation::secondary(child_class_span).with_message(format!("In class `{child_name}`")),
                    )
                    .with_note("Final methods cannot be overridden in child classes or traits.")
                    .with_help(format!(
                        "Remove the method `{method_name}()` from `{child_name}`, or remove the final modifier from the parent method."
                    )),
            );
        }
        SignatureCompatibilityIssue::StaticModifierMismatch { child_is_static, parent_is_static: _ } => {
            let (child_modifier, parent_modifier) =
                if child_is_static { ("static", "non-static") } else { ("non-static", "static") };

            context.collector.report_with_code(
                IssueCode::IncompatibleStaticModifier,
                Issue::error(format!(
                    "Cannot make {parent_modifier} method `{parent_name}::{method_name}()` {child_modifier} in class `{child_name}`"
                ))
                .with_annotation(
                    Annotation::primary(primary_span)
                        .with_message(format!("This method is {child_modifier} but should be {parent_modifier}")),
                )
                .with_annotation(Annotation::secondary(parent_class_span).with_message(format!(
                    "`{parent_name}::{method_name}()` is defined as {parent_modifier} here"
                )))
                .with_annotation(
                    Annotation::secondary(child_class_span).with_message(format!("In class `{child_name}`")),
                )
                .with_note("The static modifier must match exactly between parent and child methods.")
                .with_help(format!("Change the method in `{child_name}` to be {parent_modifier} like the parent.")),
            );
        }
        SignatureCompatibilityIssue::VisibilityNarrowed { child_visibility, parent_visibility } => {
            context.collector.report_with_code(
                IssueCode::IncompatibleVisibility,
                Issue::error(format!(
                    "Visibility of `{child_name}::{method_name}()` must not be narrowed from {parent_visibility} to {child_visibility}"
                ))
                .with_annotation(Annotation::primary(primary_span).with_message(format!(
                    "Method declared as {child_visibility} but should be {parent_visibility} or wider"
                )))
                .with_annotation(Annotation::secondary(parent_class_span).with_message(format!(
                    "Parent method `{parent_name}::{method_name}()` is declared as {parent_visibility} here"
                )))
                .with_annotation(
                    Annotation::secondary(child_class_span).with_message(format!("In class `{child_name}`")),
                )
                .with_note("Visibility can only be widened (e.g., protected → public) not narrowed.")
                .with_help(format!("Change the visibility to {parent_visibility} or wider.")),
            );
        }
        SignatureCompatibilityIssue::ParameterCountMismatch { child_required_count, parent_required_count } => {
            context.collector.report_with_code(
                IssueCode::IncompatibleParameterCount,
                Issue::error(format!(
                    "`{child_name}::{method_name}()` must accept at least {parent_required_count} required parameters like `{parent_name}::{method_name}()`"
                ))
                .with_annotation(Annotation::primary(primary_span).with_message(format!(
                    "Method requires {child_required_count} parameters but parent requires {parent_required_count}"
                )))
                .with_annotation(Annotation::secondary(parent_class_span).with_message(format!(
                    "Parent method `{parent_name}::{method_name}()` requires {parent_required_count} parameters"
                )))
                .with_annotation(
                    Annotation::secondary(child_class_span).with_message(format!("In class `{child_name}`")),
                )
                .with_note("Child methods must accept at least as many required parameters as the parent.")
                .with_help("Add optional parameters or reduce the number of required parameters in the child method."),
            );
        }
        SignatureCompatibilityIssue::IncompatibleParameterType { parameter_index, child_type, parent_type } => {
            let param_name = parent_method.parameters.get(parameter_index).map_or("unknown", |p| p.name.0.as_ref());

            context.collector.report_with_code(
                IssueCode::IncompatibleParameterType,
                Issue::error(format!(
                    "Parameter `{param_name}` of `{child_name}::{method_name}()` expects type `{child_type}` but parent `{parent_name}::{method_name}()` expects type `{parent_type}`"
                ))
                .with_annotation(Annotation::primary(primary_span).with_message(format!(
                    "Parameter `{param_name}` expects type `{child_type}` but parent expects `{parent_type}`"
                )))
                .with_annotation(
                    Annotation::secondary(parent_class_span).with_message(format!(
                        "Parent method `{parent_name}::{method_name}()` parameter defined here"
                    )),
                )
                .with_annotation(
                    Annotation::secondary(child_class_span).with_message(format!("In class `{child_name}`")),
                )
                .with_note("Parameter types must be contravariant: child must accept equal or wider types than parent.")
                .with_help("Change the parameter type to be compatible with the parent method."),
            );
        }
        SignatureCompatibilityIssue::IncompatibleReturnType { child_type, parent_type } => {
            context.collector.report_with_code(
                IssueCode::IncompatibleReturnType,
                Issue::error(format!(
                    "Return type `{child_type}` of `{child_name}::{method_name}()` is incompatible with parent return type `{parent_type}` of `{parent_name}::{method_name}()`"
                ))
                .with_annotation(
                    Annotation::primary(primary_span)
                        .with_message(format!("Returns type `{child_type}` but parent expects `{parent_type}`")),
                )
                .with_annotation(Annotation::secondary(parent_class_span).with_message(format!(
                    "Parent method `{parent_name}::{method_name}()` return type defined here"
                )))
                .with_annotation(
                    Annotation::secondary(child_class_span).with_message(format!("In class `{child_name}`")),
                )
                .with_note("Return types must be covariant: child must return equal or narrower types than parent.")
                .with_help("Change the return type to be compatible with the parent method."),
            );
        }
        SignatureCompatibilityIssue::ParameterNameMismatch {
            parameter_index,
            child_name: child_param_name,
            parent_name: parent_param_name,
        } => {
            context.collector.report_with_code(
                IssueCode::IncompatibleParameterName,
                Issue::warning(format!(
                    "Parameter #{} of `{}::{}()` is named `{}` but parent `{}::{}()` names it `{}`",
                    parameter_index + 1,
                    child_name,
                    method_name,
                    child_param_name,
                    parent_name,
                    method_name,
                    parent_param_name
                ))
                .with_annotation(Annotation::primary(primary_span).with_message(format!(
                    "Parameter named `{child_param_name}` but parent uses `{parent_param_name}`",
                )))
                .with_annotation(Annotation::secondary(parent_class_span).with_message(format!(
                    "Parent method `{parent_name}::{method_name}()` parameter `{parent_param_name}` defined here",
                )))
                .with_annotation(
                    Annotation::secondary(child_class_span).with_message(format!("In class `{child_name}`")),
                )
                .with_note("Parameter name changes can break code using named arguments.")
                .with_help(format!(
                    "Consider renaming the parameter to `{parent_param_name}` to match the parent method."
                )),
            );
        }
    }
}

fn check_class_like_properties<'ctx>(context: &mut Context<'ctx, '_>, class_like_metadata: &'ctx ClassLikeMetadata) {
    if class_like_metadata.kind.is_enum() {
        return;
    }

    // Check properties declared directly in this class against parent classes
    for (property_name, property_metadata) in &class_like_metadata.properties {
        // Only check properties declared in this class, not inherited ones
        let Some(declaring_fqcn) = class_like_metadata.declaring_property_ids.get(property_name) else {
            continue;
        };

        if declaring_fqcn != &class_like_metadata.name {
            // Property is inherited, not declared in this class
            continue;
        }

        // Validate set hook parameter type is supertype of property type
        // Use type_declaration_metadata (native type) not get_type_metadata() (merged with docblock)
        if let Some(set_hook) = property_metadata.hooks.get(&atom("set"))
            && let Some(param) = &set_hook.parameter
            && let Some(param_type) = param.type_declaration_metadata.as_ref()
            && let Some(property_type) = property_metadata.type_declaration_metadata.as_ref()
        {
            // The set hook parameter type must contain the property type (contravariance)
            // i.e., any value assignable to the property type should be accepted by the hook
            if !is_type_compatible(context.codebase, &property_type.type_union, &param_type.type_union) {
                let property_type_id = property_type.type_union.get_id();
                let param_type_id = param_type.type_union.get_id();
                let class_name = class_like_metadata.original_name;

                context.collector.report_with_code(
                    IssueCode::IncompatiblePropertyHookParameterType,
                    Issue::error(format!(
                        "Set hook parameter type `{param_type_id}` for property `{class_name}::{property_name}` is incompatible with property type `{property_type_id}`."
                    ))
                    .with_annotation(
                        Annotation::primary(param_type.span)
                            .with_message(format!("This type `{param_type_id}` does not accept all values of type `{property_type_id}`")),
                    )
                    .with_annotation(
                        Annotation::secondary(property_type.span)
                            .with_message(format!("Property is declared with type `{property_type_id}`")),
                    )
                    .with_note("The set hook parameter type must be equal to or wider than the property type (contravariance).")
                    .with_help(format!("Change the set hook parameter type to `{property_type_id}` or a wider type that contains `{property_type_id}`.")),
                );
            }
        }

        // Validate docblock param type >= native param type
        if let Some(set_hook) = property_metadata.hooks.get(&atom("set"))
            && let Some(param) = &set_hook.parameter
            && let Some(native_type) = param.type_declaration_metadata.as_ref()
            && let Some(effective_type) = param.type_metadata.as_ref()
            && effective_type.from_docblock
            && !is_type_compatible(context.codebase, &native_type.type_union, &effective_type.type_union)
        {
            let native_type_str = native_type.type_union.get_id();
            let docblock_type_str = effective_type.type_union.get_id();

            context.collector.report_with_code(
                    IssueCode::DocblockTypeMismatch,
                    Issue::error(format!(
                        "Docblock type `{docblock_type_str}` is narrower than native parameter type `{native_type_str}`."
                    ))
                    .with_annotation(
                        Annotation::primary(effective_type.span)
                            .with_message(format!("Docblock type `{docblock_type_str}` cannot narrow native type `{native_type_str}`")),
                    )
                    .with_note(
                        "The @param docblock type must be a supertype of the native type. It can widen the type (e.g., int to int|string) but not narrow it.",
                    )
                    .with_help(format!(
                        "Change the docblock type to `{native_type_str}` or a wider type.",
                    )),
                );
        }

        // Check each parent class for this property
        for parent_fqcn in &class_like_metadata.all_parent_classes {
            let parent_fqcn_str = parent_fqcn.as_ref();
            let Some(parent_metadata) = context.codebase.get_class_like(parent_fqcn_str) else {
                continue;
            };

            let Some(parent_property) = parent_metadata.properties.get(property_name) else {
                continue;
            };

            if parent_property.read_visibility.is_private() && parent_property.write_visibility.is_private() {
                continue;
            }

            let property_span = property_metadata.name_span.unwrap_or(class_like_metadata.span);
            let parent_property_span = parent_property.name_span.unwrap_or(parent_metadata.span);
            let declaring_class_name = class_like_metadata.original_name;
            let parent_class_name = parent_metadata.original_name;

            if parent_property.flags.is_final() {
                context.collector.report_with_code(
                    IssueCode::OverrideFinalProperty,
                    Issue::error(format!(
                        "Cannot override final property `{parent_class_name}::{property_name}`."
                    ))
                    .with_annotation(
                        Annotation::primary(property_span)
                            .with_message("Attempting to override final property here"),
                    )
                    .with_annotation(
                        Annotation::secondary(parent_property_span)
                            .with_message(format!("Property `{parent_class_name}::{property_name}` is declared as final")),
                    )
                    .with_note("Final properties cannot be overridden in child classes.")
                    .with_help(format!(
                        "Remove the property `{property_name}` from `{declaring_class_name}`, or remove the final modifier from the parent property.",
                    )),
                );
            }

            for (hook_name, child_hook) in &property_metadata.hooks {
                if let Some(parent_hook) = parent_property.hooks.get(hook_name)
                    && parent_hook.flags.is_final()
                {
                    context.collector.report_with_code(
                            IssueCode::OverrideFinalPropertyHook,
                            Issue::error(format!(
                                "Cannot override final property hook `{parent_class_name}::{property_name}::{hook_name}()`."
                            ))
                            .with_annotation(
                                Annotation::primary(child_hook.span)
                                    .with_message("Attempting to override final hook here"),
                            )
                            .with_annotation(
                                Annotation::secondary(parent_hook.span)
                                    .with_message(format!("Hook `{parent_class_name}::{property_name}::{hook_name}()` is declared as final")),
                            )
                            .with_note("Final property hooks cannot be overridden in child classes.")
                            .with_help(format!(
                                "Remove the `{hook_name}` hook from `{declaring_class_name}::{property_name}`, or remove the final modifier from the parent hook.",
                            )),
                        );
                }
            }

            // Backed property with by-ref get + set hook is invalid
            if parent_property.hooks.is_empty()
                && let Some(get_hook) = property_metadata.hooks.get(&atom("get"))
                && get_hook.returns_by_ref
                && property_metadata.hooks.contains_key(&atom("set"))
            {
                context.collector.report_with_code(
                    IssueCode::BackedPropertyReferenceHook,
                    Issue::error(format!(
                        "Get hook of backed property `{declaring_class_name}::{property_name}` with set hook may not return by reference."
                    ))
                    .with_annotation(
                        Annotation::primary(get_hook.span)
                            .with_message("This get hook returns by reference"),
                    )
                    .with_annotation(
                        Annotation::secondary(parent_property_span)
                            .with_message(format!("Property `{parent_class_name}::{property_name}` creates a backing store")),
                    )
                    .with_note("A backed property (with backing store) that has a set hook cannot have a by-reference get hook.")
                    .with_help("Remove the `&` from the get hook declaration, or remove the set hook."),
                );
            }

            if property_metadata.read_visibility > parent_property.read_visibility {
                let property_span = property_metadata.name_span.unwrap_or(class_like_metadata.span);
                let parent_property_span = parent_property.name_span.unwrap_or(parent_metadata.span);

                let declaring_class_name = class_like_metadata.original_name;
                let parent_class_name = parent_metadata.original_name;

                context.collector.report_with_code(
                        IssueCode::IncompatiblePropertyAccess,
                        Issue::error(format!(
                            "Property `{declaring_class_name}::{property_name}` has a different read access level than `{parent_class_name}::{property_name}`."
                        ))
                        .with_annotation(
                            Annotation::primary(property_span)
                                .with_message(format!("This property is declared as `{}`", property_metadata.read_visibility.as_str())),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_property_span)
                                .with_message(format!("Parent property is declared as `{}`", parent_property.read_visibility.as_str())),
                        )
                        .with_note("The access level of an overridden property must not be more restrictive than the parent property.")
                        .with_help("Adjust the access level of the property in the child class to match or be less restrictive than the parent class."),
                    );
            }

            if (property_metadata.write_visibility != property_metadata.read_visibility
                || parent_property.write_visibility != parent_property.read_visibility)
                && property_metadata.write_visibility > parent_property.write_visibility
            {
                let property_span = property_metadata.name_span.unwrap_or(class_like_metadata.span);
                let parent_property_span = parent_property.name_span.unwrap_or(parent_metadata.span);

                let declaring_class_name = class_like_metadata.original_name;
                let parent_class_name = parent_metadata.original_name;

                context.collector.report_with_code(
                        IssueCode::IncompatiblePropertyAccess,
                        Issue::error(format!(
                            "Property `{declaring_class_name}::{property_name}` has a different write access level than `{parent_class_name}::{property_name}`."
                        ))
                        .with_annotation(
                            Annotation::primary(property_span)
                                .with_message(format!("This property is declared as `{}(set)`", property_metadata.write_visibility.as_str())),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_property_span)
                                .with_message(format!("Parent property is declared as `{}(set)`", parent_property.write_visibility.as_str())),
                        )
                        .with_note("The access level of an overridden property must not be more restrictive than the parent property.")
                        .with_help("Adjust the access level of the property in the child class to match or be less restrictive than the parent class."),
                    );
            }

            // Check static modifier consistency
            if property_metadata.flags.is_static() != parent_property.flags.is_static() {
                let property_span = property_metadata.name_span.unwrap_or(class_like_metadata.span);
                let parent_property_span = parent_property.name_span.unwrap_or(parent_metadata.span);

                let declaring_class_name = class_like_metadata.original_name;
                let parent_class_name = parent_metadata.original_name;
                let (child_modifier, parent_modifier) = if property_metadata.flags.is_static() {
                    ("static", "non-static")
                } else {
                    ("non-static", "static")
                };

                context.collector.report_with_code(
                        IssueCode::IncompatibleStaticModifier,
                        Issue::error(format!(
                            "Cannot redeclare {parent_modifier} property `{parent_class_name}::{property_name}` as {child_modifier} `{declaring_class_name}::{property_name}`."
                        ))
                        .with_annotation(
                            Annotation::primary(property_span)
                                .with_message(format!("This property is declared as `{child_modifier}`")),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_property_span)
                                .with_message(format!("Parent property is declared as `{parent_modifier}`")),
                        )
                        .with_note("Properties must maintain the same static modifier when overriding parent properties.")
                        .with_help(format!("Change this property to be `{parent_modifier}` to match the parent class.")),
                    );
            }

            // Check readonly modifier consistency
            if property_metadata.flags.is_readonly() != parent_property.flags.is_readonly() {
                let property_span = property_metadata.name_span.unwrap_or(class_like_metadata.span);
                let parent_property_span = parent_property.name_span.unwrap_or(parent_metadata.span);

                let declaring_class_name = class_like_metadata.original_name;
                let parent_class_name = parent_metadata.original_name;
                let (child_modifier, parent_modifier) = if property_metadata.flags.is_readonly() {
                    ("readonly", "non-readonly")
                } else {
                    ("non-readonly", "readonly")
                };

                context.collector.report_with_code(
                        IssueCode::IncompatibleReadonlyModifier,
                        Issue::error(format!(
                            "Cannot redeclare {parent_modifier} property `{parent_class_name}::{property_name}` as {child_modifier} `{declaring_class_name}::{property_name}`."
                        ))
                        .with_annotation(
                            Annotation::primary(property_span)
                                .with_message(format!("This property is declared as `{child_modifier}`")),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_property_span)
                                .with_message(format!("Parent property is declared as `{parent_modifier}`")),
                        )
                        .with_note("Properties must maintain the same readonly modifier when overriding parent properties.")
                        .with_help(format!("Change this property to be `{parent_modifier}` to match the parent class.")),
                    );
            }

            let mut has_type_incompatibility = false;
            match (
                property_metadata.type_declaration_metadata.as_ref(),
                parent_property.type_declaration_metadata.as_ref(),
            ) {
                (Some(declaring_type), Some(parent_type)) => {
                    let contains_parent =
                        is_type_compatible(context.codebase, &declaring_type.type_union, &parent_type.type_union);

                    let contains_declaring =
                        is_type_compatible(context.codebase, &parent_type.type_union, &declaring_type.type_union);

                    let is_wider = contains_parent && !contains_declaring;
                    let is_narrower = contains_declaring && !contains_parent;
                    if is_wider || is_narrower {
                        has_type_incompatibility = true;

                        let declaring_type_id = declaring_type.type_union.get_id();
                        let parent_type_id = parent_type.type_union.get_id();
                        let property_name = property_metadata.name.0;
                        let class_name = class_like_metadata.original_name;

                        context.collector.report_with_code(
                                IssueCode::IncompatiblePropertyType,
                                Issue::error(format!(
                                    "Property `{class_name}::{property_name}` has an incompatible type declaration."
                                ))
                                .with_annotation(
                                    Annotation::primary(declaring_type.span)
                                        .with_message(format!("This type `{declaring_type_id}` is incompatible with the parent's type.")),
                                )
                                .with_annotation(
                                    Annotation::secondary(parent_type.span)
                                        .with_message(format!("The parent property is defined with type `{parent_type_id}` here.")),
                                )
                                .with_note("PHP requires property types to be invariant, meaning the type declaration in a child class must be exactly the same as in the parent class.")
                                .with_help(format!("Change the type of `{property_name}` to `{parent_type_id}` to match the parent property."))
                            );
                    }
                }
                (Some(declaring_type), None) => {
                    has_type_incompatibility = true;

                    let property_name = property_metadata.name.0;
                    let class_name = class_like_metadata.original_name;

                    let mut issue = Issue::error(format!(
                        "Property `{class_name}::{property_name}` adds a type that is missing on the parent property."
                    ))
                    .with_annotation(
                        Annotation::primary(declaring_type.span)
                            .with_message("This type declaration is not present on the parent property"),
                    );

                    if let Some(parent_property_span) = parent_property.name_span {
                        issue = issue.with_annotation(
                            Annotation::secondary(parent_property_span)
                                .with_message("The parent property is defined here without a type"),
                        );
                    }

                    context.collector.report_with_code(IssueCode::IncompatiblePropertyType, issue
                            .with_note("Adding a type to a property that was untyped in a parent class is an incompatible change.")
                                   .with_help("You can either remove the type from this property or add an identical type to the property in the parent class."));
                }
                (None, Some(parent_type)) => {
                    has_type_incompatibility = true;

                    if let Some(property_span) = property_metadata.name_span {
                        let property_name = property_metadata.name.0;
                        let class_name = class_like_metadata.original_name;
                        let parent_type_id = parent_type.type_union.get_id();

                        context.collector.report_with_code(
                                IssueCode::IncompatiblePropertyType,
                                Issue::error(format!(
                                    "Property `{class_name}::{property_name}` is missing the type declaration from its parent."
                                ))
                                .with_annotation(
                                    Annotation::primary(property_span)
                                        .with_message("This property declaration is missing a type"),
                                )
                                .with_annotation(
                                    Annotation::secondary(parent_type.span)
                                        .with_message(format!("The parent property is defined with type `{parent_type_id}` here")),
                                )
                                .with_note("Removing a type from a property that was typed in a parent class is an incompatible change.")
                                .with_help(format!("Add the type declaration `{parent_type_id}` to this property to match the parent definition."))
                            );
                    }
                }
                (None, None) => {
                    // no type declaration, nothing to check
                }
            }

            if !has_type_incompatibility
                && let Some(declaring_type) = &property_metadata.type_metadata
                && declaring_type.from_docblock
                && let Some(parent_type) = &parent_property.type_metadata
                && (!is_type_compatible(context.codebase, &declaring_type.type_union, &parent_type.type_union)
                    || !is_type_compatible(context.codebase, &parent_type.type_union, &declaring_type.type_union))
            {
                let declaring_type_id = declaring_type.type_union.get_id();
                let parent_type_id = parent_type.type_union.get_id();
                let property_name = property_metadata.name.0;
                let class_name = class_like_metadata.original_name;

                context.collector.report_with_code(
                        IssueCode::IncompatiblePropertyType,
                        Issue::error(format!(
                            "Property `{class_name}::{property_name}` has an incompatible type declaration from docblock."
                        ))
                        .with_annotation(
                            Annotation::primary(declaring_type.span)
                                .with_message(format!("This type `{declaring_type_id}` is incompatible with the parent's type.")),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_type.span)
                                .with_message(format!("The parent property is defined with type `{parent_type_id}` here.")),
                        )
                        .with_note("PHP requires property types to be invariant, meaning the type declaration in a child class must be exactly the same as in the parent class.")
                        .with_help(format!("Change the type of `{property_name}` to `{parent_type_id}` to match the parent property.")),
                    );
            }
        }

        // Check interface hook by-ref signature compatibility
        for interface_fqcn in &class_like_metadata.all_parent_interfaces {
            let Some(interface_metadata) = context.codebase.get_class_like(interface_fqcn) else {
                continue;
            };

            let Some(interface_property) = interface_metadata.properties.get(property_name) else {
                continue;
            };

            for (hook_name, interface_hook) in &interface_property.hooks {
                if !interface_hook.returns_by_ref {
                    continue;
                }

                let Some(impl_hook) = property_metadata.hooks.get(hook_name) else {
                    continue;
                };

                if impl_hook.returns_by_ref {
                    continue;
                }

                let declaring_class_name = class_like_metadata.original_name;
                let interface_name = interface_metadata.original_name;

                context.collector.report_with_code(
                    IssueCode::IncompatiblePropertyHookSignature,
                    Issue::error(format!(
                        "Declaration of `{declaring_class_name}::{property_name}::{hook_name}()` must be compatible with `& {interface_name}::{property_name}::{hook_name}()`."
                    ))
                    .with_annotation(
                        Annotation::primary(impl_hook.span)
                            .with_message("This hook does not return by reference"),
                    )
                    .with_annotation(
                        Annotation::secondary(interface_hook.span)
                            .with_message(format!("Interface `{interface_name}` requires this hook to return by reference")),
                    )
                    .with_note("When an interface declares a by-reference hook (`&get`), the implementing class must also return by reference.")
                    .with_help(format!("Add `&` to the `{hook_name}` hook declaration: `&{hook_name} => ...`")),
                );
            }
        }
    }
}

fn check_class_like_constants<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    members: &[ClassLikeMember<'arena>],
) {
    for member in members {
        let ClassLikeMember::Constant(constant) = member else {
            continue;
        };

        for item in &constant.items {
            let constant_name = atom(item.name.value);

            let Some(trait_fqcn) = class_like_metadata.trait_constant_ids.get(&constant_name) else {
                continue;
            };

            let Some(trait_metadata) = context.codebase.get_class_like(trait_fqcn) else {
                continue;
            };

            let Some(trait_constant) = trait_metadata.constants.get(&constant_name) else {
                continue;
            };
            let Some(class_constant) = class_like_metadata.constants.get(&constant_name) else {
                continue;
            };

            let value_matches = trait_constant.inferred_type == class_constant.inferred_type;
            let visibility_matches = trait_constant.visibility == class_constant.visibility;
            let finality_matches = trait_constant.flags.is_final() == class_constant.flags.is_final();

            if value_matches && visibility_matches && finality_matches {
                continue;
            }

            let class_name = class_like_metadata.original_name;
            let trait_name = trait_metadata.original_name;

            if !value_matches && visibility_matches && finality_matches {
                context.collector.report_with_code(
                    IssueCode::TraitConstantOverride,
                    Issue::error(format!(
                        "Class `{class_name}` cannot override constant `{constant_name}` from trait `{trait_name}` with a different value."
                    ))
                    .with_annotation(
                        Annotation::primary(item.name.span())
                            .with_message(format!("This constant has a different value than in trait `{trait_name}`")),
                    )
                    .with_note("PHP does not allow a class to override constants from traits it directly uses with a different value.")
                    .with_note(format!("Trait `{trait_name}` declares constant `{constant_name}`, which is inherited by `{class_name}`."))
                    .with_help(format!("Either use the same value as in the trait, remove the constant declaration from `{class_name}`, or remove the `use {trait_name}` statement.")),
                );
            } else {
                let mut conflicts = Vec::new();
                if !value_matches {
                    conflicts.push("value");
                }
                if !visibility_matches {
                    conflicts.push("visibility");
                }
                if !finality_matches {
                    conflicts.push("finality");
                }
                let conflicts_str = conflicts.join(", ");

                context.collector.report_with_code(
                    IssueCode::IncompatibleConstantOverride,
                    Issue::error(format!(
                        "{class_name} and {trait_name} define the same constant ({constant_name}) in the composition of {class_name}. However, the definition differs and is considered incompatible."
                    ))
                    .with_annotation(
                        Annotation::primary(item.name.span())
                            .with_message(format!("This constant differs from trait definition ({conflicts_str} differ)")),
                    )
                    .with_note(format!("Trait `{trait_name}` declares constant `{constant_name}`, which is inherited by `{class_name}`."))
                    .with_note("PHP requires that constants from traits match exactly in value, visibility, and finality when redeclared.")
                    .with_help(format!("Either match the trait's definition exactly, remove the constant declaration from `{class_name}`, or remove the `use {trait_name}` statement.")),
                );
            }
        }
    }

    for member in members {
        let ClassLikeMember::Constant(constant) = member else {
            continue;
        };

        for item in &constant.items {
            let constant_name = atom(item.name.value);

            let Some(child_constant) = class_like_metadata.constants.get(&constant_name) else {
                continue;
            };

            for parent_fqcn in &class_like_metadata.all_parent_classes {
                let parent_fqcn_str = parent_fqcn.as_ref();
                let Some(parent_metadata) = context.codebase.get_class_like(parent_fqcn_str) else {
                    continue;
                };

                let Some(parent_constant) = parent_metadata.constants.get(&constant_name) else {
                    continue;
                };

                if parent_constant.flags.is_final() {
                    let child_span = item.name.span();
                    let parent_span = parent_constant.span;
                    let class_name = class_like_metadata.original_name;
                    let parent_class_name = parent_metadata.original_name;

                    context.collector.report_with_code(
                        IssueCode::OverrideFinalConstant,
                        Issue::error(format!(
                            "Class `{class_name}` cannot override final constant `{constant_name}` from parent class `{parent_class_name}`."
                        ))
                        .with_annotation(
                            Annotation::primary(child_span)
                                .with_message("This constant attempts to override a final constant"),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_span)
                                .with_message(format!("The constant is declared as final in parent class `{parent_class_name}` here")),
                        )
                        .with_note("PHP 8.1+ allows constants to be marked as final to prevent overriding in child classes.")
                        .with_help(format!("Remove this constant declaration from `{class_name}` or remove the final modifier from the parent constant.")),
                    );
                }

                if child_constant.visibility > parent_constant.visibility {
                    let child_span = item.name.span();
                    let parent_span = parent_constant.span;
                    let class_name = class_like_metadata.original_name;
                    let parent_class_name = parent_metadata.original_name;
                    let child_visibility = child_constant.visibility;
                    let parent_visibility = parent_constant.visibility;

                    context.collector.report_with_code(
                        IssueCode::IncompatibleConstantAccess,
                        Issue::error(format!(
                            "Constant `{class_name}::{constant_name}` has narrower visibility than parent constant."
                        ))
                        .with_annotation(
                            Annotation::primary(child_span)
                                .with_message(format!("This constant is declared as `{child_visibility}`, which is narrower than the parent's `{parent_visibility}`")),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_span)
                                .with_message(format!("Parent constant is declared as `{parent_visibility}` in `{parent_class_name}` here")),
                        )
                        .with_note("PHP requires that overriding constants maintain or widen visibility (public → protected → private).")
                        .with_help(format!("Change the visibility of `{constant_name}` to at least `{parent_visibility}` to match the parent constant.")),
                    );
                }

                let (Some(child_type), Some(parent_type)) =
                    (&child_constant.type_declaration, &parent_constant.type_declaration)
                else {
                    continue;
                };

                if is_type_compatible(context.codebase, &child_type.type_union, &parent_type.type_union) {
                    continue;
                }
                let child_type_id = child_type.type_union.get_id();
                let parent_type_id = parent_type.type_union.get_id();
                let class_name = class_like_metadata.original_name;
                let parent_class_name = parent_metadata.original_name;

                context.collector.report_with_code(
                    IssueCode::IncompatibleConstantType,
                    Issue::error(format!(
                        "Constant `{class_name}::{constant_name}` has an incompatible type declaration."
                    ))
                    .with_annotation(
                        Annotation::primary(child_type.span)
                            .with_message(format!("This type `{child_type_id}` is not compatible with the parent's type")),
                    )
                    .with_annotation(
                        Annotation::secondary(parent_type.span)
                            .with_message(format!("The parent constant is defined with type `{parent_type_id}` in `{parent_class_name}` here")),
                    )
                    .with_note("PHP 8.3+ allows typed constants with covariance, meaning the child type must be a subtype of the parent type.")
                    .with_help(format!("Change the type of `{constant_name}` to be compatible with `{parent_type_id}`.")),
                );
            }
        }
    }

    for member in members {
        let ClassLikeMember::Constant(constant) = member else {
            continue;
        };

        for item in &constant.items {
            let constant_name = atom(item.name.value);

            let Some(child_constant) = class_like_metadata.constants.get(&constant_name) else {
                continue;
            };

            for interface_fqcn in &class_like_metadata.all_parent_interfaces {
                let interface_fqcn_str = interface_fqcn.as_ref();
                let Some(interface_metadata) = context.codebase.get_class_like(interface_fqcn_str) else {
                    continue;
                };

                let Some(interface_constant) = interface_metadata.constants.get(&constant_name) else {
                    continue;
                };

                if child_constant.visibility != Visibility::Public {
                    let child_span = item.name.span();
                    let interface_span = interface_constant.span;
                    let class_name = class_like_metadata.original_name;
                    let interface_name = interface_metadata.original_name;
                    let child_visibility = child_constant.visibility;

                    context.collector.report_with_code(
                        IssueCode::IncompatibleConstantVisibility,
                        Issue::error(format!(
                            "Constant `{class_name}::{constant_name}` must be public to implement interface `{interface_name}`."
                        ))
                        .with_annotation(
                            Annotation::primary(child_span)
                                .with_message(format!("This constant is declared as `{child_visibility}`, but must be `public`")),
                        )
                        .with_annotation(
                            Annotation::secondary(interface_span)
                                .with_message(format!("Interface constant is declared in `{interface_name}` here")),
                        )
                        .with_note("All interface constants are implicitly public and implementing classes must maintain public visibility.")
                        .with_help(format!("Change the visibility of `{constant_name}` to `public`.")),
                    );
                }

                if let (Some(child_type), Some(interface_type)) =
                    (&child_constant.type_declaration, &interface_constant.type_declaration)
                    && !is_type_compatible(context.codebase, &child_type.type_union, &interface_type.type_union)
                {
                    let child_type_id = child_type.type_union.get_id();
                    let interface_type_id = interface_type.type_union.get_id();
                    let class_name = class_like_metadata.original_name;
                    let interface_name = interface_metadata.original_name;

                    context.collector.report_with_code(
                            IssueCode::IncompatibleConstantType,
                            Issue::error(format!(
                                "Constant `{class_name}::{constant_name}` has an incompatible type declaration."
                            ))
                            .with_annotation(
                                Annotation::primary(child_type.span)
                                    .with_message(format!("This type `{child_type_id}` is not compatible with the interface's type")),
                            )
                            .with_annotation(
                                Annotation::secondary(interface_type.span)
                                    .with_message(format!("The interface constant is defined with type `{interface_type_id}` in `{interface_name}` here")),
                            )
                            .with_note("Constants implementing interface constants must have compatible types (covariance allowed).")
                            .with_help(format!("Change the type of `{constant_name}` to be compatible with `{interface_type_id}`.")),
                        );
                }
            }
        }
    }
}

/// Check that a readonly class does not use traits with non-readonly properties.
///
/// In PHP, a readonly class can only use traits where all properties are declared readonly.
/// Using a trait with non-readonly properties in a readonly class causes a fatal error.
fn check_readonly_class_trait_properties<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    members: &[ClassLikeMember<'arena>],
) {
    if !class_like_metadata.flags.is_readonly() {
        return;
    }

    for member in members {
        if let ClassLikeMember::TraitUse(trait_use) = member {
            for trait_name_id in &trait_use.trait_names {
                let (trait_fqcn, _) = context.scope.resolve(NameKind::Default, trait_name_id.value());
                let trait_fqcn = Atom::from(trait_fqcn.as_str());

                let Some(trait_metadata) = context.codebase.get_class_like(trait_fqcn.as_ref()) else {
                    continue;
                };

                for (property_name, property) in &trait_metadata.properties {
                    if !property.flags.is_readonly() {
                        context.collector.report_with_code(
                            IssueCode::InvalidTraitUse,
                            Issue::error(format!(
                                "Readonly class `{}` cannot use trait `{}` which has non-readonly property `{}`",
                                class_like_metadata.name, trait_fqcn, property_name
                            ))
                            .with_annotation(Annotation::primary(trait_name_id.span()).with_message("Trait used here"))
                            .with_note("All properties in a trait used by a readonly class must be declared readonly.")
                            .with_help(format!(
                                "Add the `readonly` modifier to property `{}` in trait `{}`.",
                                property_name, trait_fqcn
                            )),
                        );
                    }
                }
            }
        }
    }
}
