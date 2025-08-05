use ahash::RandomState;
use indexmap::IndexMap;

use mago_codex::context::ScopeContext;
use mago_codex::get_class_like;
use mago_codex::get_declaring_property;
use mago_codex::get_method;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::standin_type_replacer;
use mago_codex::ttype::template::standin_type_replacer::StandinOptions;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_names::kind::NameKind;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::Code;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

pub mod constant;
pub mod enum_case;
pub mod method;
pub mod property;

impl Analyzable for Class {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        block_context: &mut BlockContext,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLike,
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, context.interner, name) else {
            let name_str = context.interner.lookup(name);
            tracing::warn!("Class {} not found in codebase", name_str);

            return Ok(());
        };

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

        Ok(())
    }
}

impl Analyzable for Interface {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        block_context: &mut BlockContext,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLike,
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, context.interner, name) else {
            let name_str = context.interner.lookup(name);
            tracing::warn!("Interface {} not found in codebase", name_str);

            return Ok(());
        };

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

        Ok(())
    }
}

impl Analyzable for Trait {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        block_context: &mut BlockContext,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLike,
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, context.interner, name) else {
            let name_str = context.interner.lookup(name);
            tracing::warn!("Trait {} not found in codebase", name_str);

            return Ok(());
        };

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

        Ok(())
    }
}

impl Analyzable for Enum {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        block_context: &mut BlockContext<'_>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLike,
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, context.interner, name) else {
            let name_str = context.interner.lookup(name);
            tracing::warn!("Enum {} not found in codebase", name_str);

            return Ok(());
        };

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

        Ok(())
    }
}

pub(crate) fn analyze_class_like<'a>(
    context: &mut Context<'a>,
    artifacts: &mut AnalysisArtifacts,
    name_span: Option<Span>,
    declaration_span: Span,
    extends_ast: Option<&Extends>,
    implements_ast: Option<&Implements>,
    class_like_metadata: &'a ClassLikeMetadata,
    members: &[ClassLikeMember],
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

    if class_like_metadata.unchecked {
        return Ok(());
    }

    let name = &class_like_metadata.name;
    let name_str = context.interner.lookup(&class_like_metadata.original_name);

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

    if !class_like_metadata.kind.is_trait() && !class_like_metadata.is_abstract {
        for (method_name, fqcn) in &class_like_metadata.declaring_method_ids {
            if class_like_metadata.kind.is_enum() {
                let method_name_str = context.interner.lookup(method_name);
                if method_name_str.eq_ignore_ascii_case("cases") {
                    continue;
                }

                if class_like_metadata.enum_type.is_some()
                    && (method_name_str.eq_ignore_ascii_case("from") || method_name_str.eq_ignore_ascii_case("tryFrom"))
                {
                    continue;
                }
            }

            let Some(declaring_class_like_metadata) = get_class_like(context.codebase, context.interner, fqcn) else {
                continue;
            };

            let Some(function_like) = get_method(context.codebase, context.interner, fqcn, method_name) else {
                continue;
            };

            let Some(method_metadata) = function_like.method_metadata.as_ref() else {
                continue;
            };

            if method_metadata.is_abstract {
                let method_name_str = context.interner.lookup(method_name);
                let fqcn_str = context.interner.lookup(&declaring_class_like_metadata.original_name);
                let method_span = function_like.name_span.unwrap_or(function_like.span);

                context.collector.report_with_code(
                    Code::UNIMPLEMENTED_ABSTRACT_METHOD,
                    Issue::error(format!(
                        "Class `{name_str}` does not implement the abstract method `{method_name_str}`.",
                    ))
                    .with_annotation(
                        Annotation::primary(name_span.unwrap_or(declaration_span))
                            .with_message(format!("`{name_str}` is not abstract and must implement this method")),
                    )
                    .with_annotation(
                        Annotation::secondary(method_span).with_message(
                            format!("`{fqcn_str}::{method_name_str}` is defined as abstract here")
                        ),
                    )
                    .with_note("When a concrete class extends an abstract class or implements an interface, it must provide an implementation for all inherited abstract methods.".to_string())
                    .with_help(format!(
                        "You can either implement the `{method_name_str}` method in `{name_str}`, or declare `{name_str}` as an abstract class.",
                    )),
                );
            }
        }
    }

    if !class_like_metadata.template_types.is_empty() {
        let class_name_str = context.interner.lookup(name);

        for (template_name, _) in &class_like_metadata.template_types {
            let template_name_str = context.interner.lookup(template_name);
            let (resolved_template_name, _) = context.scope.resolve(NameKind::Default, template_name_str);
            let resolved_template_name_id = context.interner.intern(resolved_template_name);
            if let Some(conflicting_class) =
                get_class_like(context.codebase, context.interner, &resolved_template_name_id)
            {
                let conflicting_class_name = context.interner.lookup(&conflicting_class.name);
                let conflicting_class_span = conflicting_class.name_span.unwrap_or(conflicting_class.span);

                context.collector.report_with_code(
                    Code::NAME_ALREADY_IN_USE,
                    Issue::error(format!(
                        "In class `{class_name_str}`, the template parameter `{template_name_str}` conflicts with an existing class.",
                    ))
                    .with_annotation(
                        Annotation::primary(name_span.unwrap_or(declaration_span))
                            .with_message("The docblock for this class defines the conflicting template parameter"),
                    )
                    .with_annotation(
                        Annotation::secondary(conflicting_class_span)
                            .with_message(format!("The conflicting type `{conflicting_class_name}` is defined here")),
                    )
                    .with_note("Template parameter names (from `@template`) must not conflict with existing classes, interfaces, enums, or traits in the same scope.")
                    .with_help(format!(
                        "In the docblock for the `{class_name_str}` type, rename the `@template {template_name_str}` parameter to avoid this naming collision.",
                    )),
                );
            }
        }
    }

    check_class_like_properties(context, class_like_metadata);

    let mut block_context = BlockContext::new({
        let mut scope = ScopeContext::new();

        scope.set_class_like(Some(class_like_metadata));
        scope.set_static(true);
        scope
    });

    for member in members {
        match member {
            ClassLikeMember::Constant(class_like_constant) => {
                class_like_constant.analyze(context, &mut block_context, artifacts)?;
            }
            ClassLikeMember::Property(property) => {
                property.analyze(context, &mut block_context, artifacts)?;
            }
            ClassLikeMember::EnumCase(enum_case) => {
                enum_case.analyze(context, &mut block_context, artifacts)?;
            }
            ClassLikeMember::Method(method) => {
                method.analyze(context, &mut block_context, artifacts)?;
            }
            _ => {
                continue;
            }
        }
    }

    Ok(())
}

fn check_class_like_extends(
    context: &mut Context<'_>,
    class_like_metadata: &ClassLikeMetadata,
    extends_ast: Option<&Extends>,
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
    let using_name_str = context.interner.lookup(&class_like_metadata.original_name);
    let using_class_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);

    for extended_type in extends.types.iter() {
        let extended_type_id = context.resolved_names.get(&extended_type);
        let extended_class_metadata = get_class_like(context.codebase, context.interner, extended_type_id);

        // Case: The extended type does not exist.
        let Some(extended_class_metadata) = extended_class_metadata else {
            let extended_name = context.interner.lookup(extended_type.value());
            context.collector.report_with_code(
                Code::NON_EXISTENT_CLASS_LIKE,
                Issue::error(format!("{using_kind_capitalized} `{using_name_str}` cannot extend unknown type `{extended_name}`"))
                    .with_annotation(Annotation::primary(extended_type.span()).with_message("This type could not be found"))
                    .with_note("Mago could not find a definition for this class, interface, or trait.")
                    .with_help("Ensure the name is correct, including its namespace, and that it is properly defined and autoloadable."),
            );
            continue;
        };

        let extended_name_str = context.interner.lookup(&extended_class_metadata.original_name);
        let extended_kind_str = extended_class_metadata.kind.as_str();
        let extended_kind_prefix =
            if extended_class_metadata.kind.is_class() || extended_class_metadata.kind.is_trait() { "a" } else { "an" };
        let extended_class_span = extended_class_metadata.name_span.unwrap_or(extended_class_metadata.span);

        if extended_class_metadata.is_deprecated {
            context.collector.report_with_code(
                Code::DEPRECATED_CLASS,
                Issue::warning(format!("Use of deprecated class `{extended_name_str}` in `extends` clause"))
                    .with_annotation(Annotation::primary(extended_type.span()).with_message("This class is marked as deprecated"))
                    .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("`{extended_name_str}` was marked deprecated here")))
                    .with_note("The parent type is deprecated and may be removed in a future version, which would break this child type.")
                    .with_help("Consider refactoring to avoid extending this type, or consult its documentation for alternatives."),
            );
        }

        if class_like_metadata.kind.is_interface() {
            if !extended_class_metadata.kind.is_interface() {
                context.collector.report_with_code(
                    Code::INVALID_EXTEND,
                    Issue::error(format!("Interface `{using_name_str}` cannot extend non-interface type `{extended_name_str}`"))
                        .with_annotation(Annotation::primary(extended_type.span())
                            .with_message(format!("...because it is {extended_kind_prefix} {extended_kind_str}, not an interface")))
                        .with_annotation(Annotation::secondary(extended_class_span)
                            .with_message(format!("`{extended_name_str}` is defined as {extended_kind_prefix} {extended_kind_str} here")))
                        .with_note("In PHP, an interface can only extend other interfaces.")
                        .with_help(format!("To resolve this, change `{extended_name_str}` to be an interface, or change `{using_name_str}` to a class if you intended to extend a class.")),
                );

                continue;
            }

            if extended_class_metadata.is_enum_interface && !class_like_metadata.is_enum_interface {
                context.collector.report_with_code(
                    Code::INVALID_EXTEND,
                    Issue::error(format!("Interface `{using_name_str}` cannot extend enum-interface `{extended_name_str}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message("This interface is not an `@enum-interface`..."))
                        .with_annotation(Annotation::secondary(extended_type.span()).with_message("...but it extends an `@enum-interface`"))
                        .with_note("An interface marked with `@enum-interface` can only be extended by other interfaces that are also marked with `@enum-interface`.")
                        .with_help(format!("To resolve this, add the `@enum-interface` PHPDoc tag to `{using_name_str}`, or extend a regular, non-enum interface.")),
                );
            }
        }

        if class_like_metadata.kind.is_class() {
            if !extended_class_metadata.kind.is_class() {
                context.collector.report_with_code(
                    Code::INVALID_EXTEND,
                    Issue::error(format!(
                        "Class `{using_name_str}` cannot extend non-class type `{extended_name_str}`"
                    ))
                    .with_annotation(Annotation::primary(extended_type.span()).with_message(format!(
                        "...because it is {extended_kind_prefix} {extended_kind_str}, not a class"
                    )))
                    .with_annotation(Annotation::secondary(extended_class_span).with_message(format!(
                        "`{extended_name_str}` is defined as {extended_kind_prefix} {extended_kind_str} here"
                    )))
                    .with_note("In PHP, a class can only extend another class.")
                    .with_help("To inherit from an interface, use `implements`. To use a trait, use `use`."),
                );

                continue;
            }

            if extended_class_metadata.is_final {
                context.collector.report_with_code(
                    Code::EXTEND_FINAL_CLASS,
                    Issue::error(format!("Class `{using_name_str}` cannot extend final class `{extended_name_str}`"))
                        .with_annotation(Annotation::primary(extended_type.span()).with_message("This inheritance is not allowed"))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("`{extended_name_str}` is declared 'final' here")))
                        .with_note("A class marked as `final` cannot be extended by any other class.")
                        .with_help(format!("To resolve this, either remove the `final` keyword from `{extended_name_str}`, or choose a different class to extend.")),
                );
            }

            if extended_class_metadata.is_readonly && !class_like_metadata.is_readonly {
                context.collector.report_with_code(
                    Code::INVALID_EXTEND,
                    Issue::error(format!("Non-readonly class `{using_name_str}` cannot extend readonly class `{extended_name_str}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message("This class is not `readonly`..."))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("...but it extends `{extended_name_str}`, which is `readonly`")))
                        .with_note("A `readonly` class can only be extended by another `readonly` class.")
                        .with_help(format!("To resolve this, either make the `{using_name_str}` class `readonly`, or extend a different, non-readonly class.")),
                );
            }

            if extended_class_metadata.is_external_mutation_free && !class_like_metadata.is_external_mutation_free {
                context.collector.report_with_code(
                    Code::INVALID_EXTEND,
                    Issue::error(format!("Mutable class `{using_name_str}` cannot extend `@external-mutation-free` class `{extended_name_str}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message("This class is mutable..."))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("...but it extends `{extended_name_str}`, which is `@external-mutation-free`")))
                        .with_note("A class that allows mutation cannot inherit from a class that guarantees immutability via `@external-mutation-free`.")
                        .with_help(format!("To resolve this, either mark `{using_name_str}` with the `@external-mutation-free` annotation, or choose a different parent class.")),
                );
            }

            if extended_class_metadata.is_mutation_free && !class_like_metadata.is_mutation_free {
                context.collector.report_with_code(
                    Code::INVALID_EXTEND,
                    Issue::error(format!("Mutable class `{using_name_str}` cannot extend `@mutation-free` class `{extended_name_str}`"))
                        .with_annotation(Annotation::primary(using_class_span).with_message("This class is mutable..."))
                        .with_annotation(Annotation::secondary(extended_class_span).with_message(format!("...but it extends `{extended_name_str}`, which is `@mutation-free`")))
                        .with_note("A class that allows mutation cannot inherit from a class that guarantees immutability via `@mutation-free`.")
                        .with_help(format!("To resolve this, either mark `{using_name_str}` with the `@mutation-free` annotation, or choose a different parent class.")),
                );
            }

            if !class_like_metadata.is_abstract {
                for required_interface in &extended_class_metadata.require_implements {
                    if !class_like_metadata.all_parent_interfaces.contains(required_interface) {
                        let required_iface_str = context.interner.lookup(required_interface);
                        context.collector.report_with_code(
                              Code::MISSING_REQUIRED_INTERFACE,
                              Issue::error(format!("Class `{using_name_str}` must implement required interface `{required_iface_str}`"))
                                  .with_annotation(Annotation::primary(using_class_span).with_message(format!("...because its parent `{extended_name_str}` requires it")))
                                  .with_annotation(Annotation::secondary(extended_class_span).with_message("Requirement declared here (likely via `@require-implements`)"))
                                  .with_note("When a class uses `@require-implements`, all of its concrete child classes must implement the specified interface.")
                                  .with_help(format!("Add `implements {required_iface_str}` to the `{using_name_str}` definition, or declare `{using_name_str}` as `abstract`.")),
                          );
                    }
                }
            }

            if !class_like_metadata.is_abstract
                && let Some(permitted_inheritors) = &extended_class_metadata.permitted_inheritors
                && !permitted_inheritors.contains(&class_like_metadata.name)
                && !class_like_metadata
                    .all_parent_interfaces
                    .iter()
                    .any(|parent_interface| permitted_inheritors.contains(parent_interface))
                && !class_like_metadata
                    .all_parent_classes
                    .iter()
                    .any(|parent_class| permitted_inheritors.contains(parent_class))
            {
                context.collector.report_with_code(
                    Code::INVALID_EXTEND,
                    Issue::error(format!("Class `{using_name_str}` is not permitted to extend `{extended_name_str}`"))
                        .with_annotation(Annotation::primary(extended_type.span()).with_message("This inheritance is restricted"))
                        .with_annotation(Annotation::secondary(extended_class_span)
                            .with_message(format!("The `@inheritors` annotation on this class does not include `{using_name_str}`")))
                        .with_note("The `@inheritors` annotation on a class or interface restricts which types are allowed to extend it.")
                        .with_help(format!("To allow this, add `{using_name_str}` to the list in the `@inheritors` PHPDoc tag for `{extended_name_str}`.")),
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

fn check_class_like_implements(
    context: &mut Context<'_>,
    class_like_metadata: &ClassLikeMetadata,
    implements_ast: Option<&Implements>,
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
    let using_name_str = context.interner.lookup(&class_like_metadata.original_name);
    let using_class_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);

    for implemented_type in implements.types.iter() {
        let implemented_type_id = context.resolved_names.get(&implemented_type);
        let implemented_interface_metadata = get_class_like(context.codebase, context.interner, implemented_type_id);

        match implemented_interface_metadata {
            Some(implemented_metadata) => {
                let implemented_name_str = context.interner.lookup(&implemented_metadata.original_name);
                let implemented_kind_str = implemented_metadata.kind.as_str();
                let implemented_class_span = implemented_metadata.name_span.unwrap_or(implemented_metadata.span);
                let implemented_kind_prefix =
                    if implemented_metadata.kind.is_class() || implemented_metadata.kind.is_trait() {
                        "a"
                    } else {
                        "an"
                    };

                if !implemented_metadata.kind.is_interface() {
                    context.collector.report_with_code(
                        Code::INVALID_IMPLEMENT,
                        Issue::error(format!("{using_kind_capitalized} `{using_name_str}` cannot implement non-interface type `{implemented_name_str}`"))
                            .with_annotation(Annotation::primary(implemented_type.span())
                                .with_message(format!("...because it is {implemented_kind_prefix} {implemented_kind_str}, not an interface")))
                            .with_annotation(Annotation::secondary(implemented_class_span)
                                .with_message(format!("`{implemented_name_str}` is defined as {implemented_kind_prefix} {implemented_kind_str} here")))
                            .with_note("The `implements` keyword is exclusively for implementing interfaces.")
                            .with_help("To inherit from a class, use `extends`. To use a trait, use `use`."),
                    );

                    continue;
                }

                if implemented_metadata.is_enum_interface && !class_like_metadata.kind.is_enum() {
                    context.collector.report_with_code(
                        Code::INVALID_IMPLEMENT,
                        Issue::error(format!("{using_kind_capitalized} `{using_name_str}` cannot implement enum-only interface `{implemented_name_str}`"))
                            .with_annotation(Annotation::primary(using_class_span).with_message(format!("This {using_kind_str} is not an enum...")))
                            .with_annotation(Annotation::secondary(implemented_type.span()).with_message("...but it implements an interface restricted to enums"))
                            .with_annotation(Annotation::secondary(implemented_class_span).with_message("This interface is marked with `@enum-interface` here"))
                            .with_note("An interface marked with `@enum-interface` can only be implemented by enums.")
                            .with_help(format!("To resolve this, either change `{using_name_str}` to be an enum, or implement a different, non-enum interface.")),
                    );
                }

                if !class_like_metadata.is_abstract
                    && let Some(permitted_inheritors) = &implemented_metadata.permitted_inheritors
                    && !permitted_inheritors.contains(&class_like_metadata.name)
                    && !class_like_metadata
                        .all_parent_interfaces
                        .iter()
                        .any(|parent_interface| permitted_inheritors.contains(parent_interface))
                    && !class_like_metadata
                        .all_parent_classes
                        .iter()
                        .any(|parent_class| permitted_inheritors.contains(parent_class))
                {
                    context.collector.report_with_code(
                        Code::INVALID_IMPLEMENT,
                        Issue::error(format!("{using_kind_capitalized} `{using_name_str}` is not permitted to implement `{implemented_name_str}`"))
                             .with_annotation(Annotation::primary(implemented_type.span()).with_message("This implementation is restricted"))
                            .with_annotation(Annotation::secondary(implemented_class_span)
                                .with_message(format!("The `@inheritors` annotation on this interface does not include `{using_name_str}`")))
                            .with_note("The `@inheritors` annotation on an interface restricts which types are allowed to implement it.")
                            .with_help(format!("To allow this, add `{using_name_str}` to the list in the `@inheritors` PHPDoc tag for `{implemented_name_str}`.")),
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
            }
            None => {
                let implemented_name = context.interner.lookup(implemented_type.value());

                context.collector.report_with_code(
                    Code::NON_EXISTENT_CLASS_LIKE,
                    Issue::error(format!("{using_kind_capitalized} `{using_name_str}` cannot implement unknown type `{implemented_name}`"))
                        .with_annotation(Annotation::primary(implemented_type.span()).with_message("This type could not be found"))
                        .with_note("Mago could not find a definition for this interface. The `implements` keyword is for interfaces only.")
                        .with_help("Ensure the name is correct, including its namespace, and that it is properly defined and autoloadable."),
                );
            }
        }
    }
}

fn check_class_like_use(context: &mut Context<'_>, class_like_metadata: &ClassLikeMetadata, trait_use: &TraitUse) {
    let using_kind_str = class_like_metadata.kind.as_str();
    let using_kind_capitalized =
        format!("{}{}", using_kind_str.chars().next().unwrap().to_uppercase(), &using_kind_str[1..]);
    let using_name_str = context.interner.lookup(&class_like_metadata.original_name);
    let using_class_span = class_like_metadata.name_span.unwrap_or(class_like_metadata.span);

    for used_type in trait_use.trait_names.iter() {
        let used_type_id = context.resolved_names.get(&used_type);
        let used_trait_metadata = get_class_like(context.codebase, context.interner, used_type_id);

        let Some(used_trait_metadata) = used_trait_metadata else {
            let used_name = context.interner.lookup(used_type.value());
            context.collector.report_with_code(
                Code::NON_EXISTENT_CLASS_LIKE,
                Issue::error(format!("{using_kind_capitalized} `{using_name_str}` cannot use unknown type `{used_name}`"))
                    .with_annotation(Annotation::primary(used_type.span()).with_message("This type could not be found"))
                    .with_note("Mago could not find a definition for this trait. The `use` keyword is for traits only.")
                    .with_help("Ensure the name is correct, including its namespace, and that it is properly defined and autoloadable."),
            );

            continue;
        };

        let used_name_str = context.interner.lookup(&used_trait_metadata.original_name);
        let used_kind_str = used_trait_metadata.kind.as_str();
        let used_kind_prefix =
            if used_trait_metadata.kind.is_class() || used_trait_metadata.kind.is_trait() { "a" } else { "an" };
        let used_class_span = used_trait_metadata.name_span.unwrap_or(used_trait_metadata.span);

        // Case: Using something that is not a trait.
        if !used_trait_metadata.kind.is_trait() {
            context.collector.report_with_code(
                Code::INVALID_TRAIT_USE,
                Issue::error(format!(
                    "{using_kind_capitalized} `{using_name_str}` cannot use non-trait type `{used_name_str}`"
                ))
                .with_annotation(
                    Annotation::primary(used_type.span())
                        .with_message(format!("...because it is {used_kind_prefix} {used_kind_str}, not a trait")),
                )
                .with_annotation(
                    Annotation::secondary(used_class_span).with_message(format!(
                        "`{used_name_str}` is defined as {used_kind_prefix} {used_kind_str} here"
                    )),
                )
                .with_note("The `use` keyword is exclusively for including traits in classes, enums, or other traits.")
                .with_help("To inherit from a class, use `extends`. To implement an interface, use `implements`."),
            );

            continue;
        }

        if used_trait_metadata.is_deprecated {
            context.collector.report_with_code(
                Code::DEPRECATED_TRAIT,
                Issue::error(format!("Use of deprecated trait `{used_name_str}` in `{using_name_str}`"))
                    .with_annotation(Annotation::primary(used_type.span()).with_message("This trait is marked as deprecated"))
                    .with_annotation(Annotation::secondary(used_class_span).with_message(format!("`{used_name_str}` was marked as deprecated here")))
                    .with_note("This trait is deprecated and may be removed in a future version, which would break the consuming type.")
                    .with_help("Consider refactoring to avoid using this trait, or consult its documentation for alternatives."),
            );
        }

        if used_trait_metadata.is_external_mutation_free && !class_like_metadata.is_external_mutation_free {
            context.collector.report_with_code(
                Code::INVALID_TRAIT_USE,
                Issue::error(format!("Mutable {using_kind_str} `{using_name_str}` cannot use `@external-mutation-free` trait `{used_name_str}`"))
                    .with_annotation(Annotation::primary(using_class_span).with_message(format!("This {using_kind_str} is mutable...")))
                    .with_annotation(Annotation::secondary(used_class_span).with_message(format!("...but it uses `{used_name_str}`, which is `@external-mutation-free`")))
                    .with_note("A type that allows mutation cannot use a trait that guarantees immutability via `@external-mutation-free`.")
                    .with_help(format!("To resolve this, either mark `{using_name_str}` with the `@external-mutation-free` annotation, or use a different trait.")),
            );
        }

        if used_trait_metadata.is_mutation_free && !class_like_metadata.is_mutation_free {
            context.collector.report_with_code(
                Code::INVALID_TRAIT_USE,
                Issue::error(format!("Mutable {using_kind_str} `{using_name_str}` cannot use `@mutation-free` trait `{used_name_str}`"))
                    .with_annotation(Annotation::primary(using_class_span).with_message(format!("This {using_kind_str} is mutable...")))
                    .with_annotation(Annotation::secondary(used_class_span).with_message(format!("...but it uses `{used_name_str}`, which is `@mutation-free`")))
                    .with_note("A type that allows mutation cannot use a trait that guarantees immutability via `@mutation-free`.")
                    .with_help(format!("To resolve this, either mark `{using_name_str}` with the `@mutation-free` annotation, or use a different trait.")),
            );
        }

        if !class_like_metadata.is_abstract {
            for required_interface in &used_trait_metadata.require_implements {
                if !class_like_metadata.all_parent_interfaces.contains(required_interface) {
                    let required_iface_str = context.interner.lookup(required_interface);
                    context.collector.report_with_code(
                        Code::MISSING_REQUIRED_INTERFACE,
                        Issue::error(format!("{using_kind_capitalized} `{using_name_str}` must implement required interface `{required_iface_str}`"))
                            .with_annotation(Annotation::primary(using_class_span).with_message(format!("...because the trait `{used_name_str}` requires it")))
                            .with_annotation(Annotation::secondary(used_type.span()).with_message(format!("The requirement is introduced by using `{used_name_str}` here")))
                            .with_note("When a trait uses `@require-implements`, any concrete class using that trait must implement the specified interface.")
                            .with_help(format!("Add `implements {required_iface_str}` to the `{using_name_str}` definition, or declare it as `abstract`.")),
                    );
                }
            }

            for required_class in &used_trait_metadata.require_extends {
                if !class_like_metadata.all_parent_classes.contains(required_class) {
                    let required_class_str = context.interner.lookup(required_class);
                    context.collector.report_with_code(
                        Code::MISSING_REQUIRED_PARENT,
                        Issue::error(format!("{using_kind_capitalized} `{using_name_str}` must extend required class `{required_class_str}`"))
                            .with_annotation(Annotation::primary(using_class_span).with_message(format!("...because the trait `{used_name_str}` requires it")))
                            .with_annotation(Annotation::secondary(used_type.span()).with_message(format!("The requirement is introduced by using `{used_name_str}` here")))
                            .with_note("When a trait uses `@require-extends`, any class using that trait must extend the specified class.")
                            .with_help(format!("Add `extends {required_class_str}` to the `{using_name_str}` definition, or ensure it is a parent class.")),
                    );
                }
            }
        }

        if !class_like_metadata.is_abstract
            && let Some(permitted_inheritors) = &used_trait_metadata.permitted_inheritors
            && !permitted_inheritors.contains(&class_like_metadata.name)
            && !class_like_metadata
                .all_parent_interfaces
                .iter()
                .any(|parent_interface| permitted_inheritors.contains(parent_interface))
            && !class_like_metadata
                .all_parent_classes
                .iter()
                .any(|parent_class| permitted_inheritors.contains(parent_class))
        {
            context.collector.report_with_code(
                Code::INVALID_TRAIT_USE,
                Issue::error(format!("{using_kind_capitalized} `{using_name_str}` is not permitted to use trait `{used_name_str}`"))
                    .with_annotation(Annotation::primary(used_type.span()).with_message("This usage is restricted"))
                    .with_annotation(Annotation::secondary(used_class_span).with_message(format!("The `@inheritors` annotation on this trait does not include `{using_name_str}`")))
                    .with_note("The `@inheritors` annotation on a trait restricts which types are allowed to use it.")
                    .with_help(format!("To allow this, add `{using_name_str}` to the list in the `@inheritors` PHPDoc tag for `{used_name_str}`.")),
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

fn check_template_parameters(
    context: &mut Context<'_>,
    class_like_metadata: &ClassLikeMetadata,
    parent_metadata: &ClassLikeMetadata,
    actual_parameters_count: usize,
    inheritance: InheritanceKind,
) {
    let expected_parameters_count = parent_metadata.template_types.len();

    let class_name_str = context.interner.lookup(&class_like_metadata.original_name);
    let class_kind_str = class_like_metadata.kind.as_str();
    let parent_name_str = context.interner.lookup(&parent_metadata.original_name);
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
            "Too few template arguments for `{parent_name_str}`: expected {expected_parameters_count}, but found {actual_parameters_count}."
        ))
        .with_annotation(
            Annotation::primary(primary_annotation_span)
                .with_message(format!("Too few template arguments provided here when `{class_name_str}` {inheritance_keyword} `{parent_name_str}`")),
        )
        .with_annotation(
            Annotation::secondary(class_name_span)
                .with_message(format!("Declaration of `{class_name_str}` is here")),
        )
        .with_annotation(
            Annotation::secondary(parent_definition_span)
                .with_message(format!("`{parent_name_str}` is defined with {expected_parameters_count} template parameters")),
        )
        .with_help(format!("Provide all {expected_parameters_count} required template arguments in the `{inheritance_tag}` docblock tag for `{class_name_str}`."));

        context.collector.report_with_code(Code::MISSING_TEMPLATE_PARAMETER, issue);
    } else if expected_parameters_count < actual_parameters_count {
        let issue = Issue::error(format!(
            "Too many template arguments for `{parent_name_str}`: expected {expected_parameters_count}, but found {actual_parameters_count}."
        ))
        .with_annotation(
            Annotation::primary(primary_annotation_span)
                .with_message(format!("Too many template arguments provided here when `{class_name_str}` {inheritance_keyword} `{parent_name_str}`")),
        )
        .with_annotation(
            Annotation::secondary(class_name_span)
                .with_message(format!("Declaration of `{class_name_str}` is here")),
        )
        .with_annotation(
            Annotation::secondary(parent_definition_span)
                .with_message(format!("`{parent_name_str}` is defined with {expected_parameters_count} template parameters")),
        )
        .with_help(format!("Remove the extra arguments from the `{inheritance_tag}` tag for `{class_name_str}`."));

        context.collector.report_with_code(Code::EXCESS_TEMPLATE_PARAMETER, issue);
    }

    let own_template_parameters_len = class_like_metadata.template_types.len();
    if parent_metadata.has_consistent_templates && own_template_parameters_len != expected_parameters_count {
        context.collector.report_with_code(
            Code::INCONSISTENT_TEMPLATE,
            Issue::error(format!(
                "Template parameter count mismatch: `{class_name_str}` must have {expected_parameters_count} template parameters to match `{parent_name_str}`."
            ))
            .with_annotation(Annotation::primary(class_name_span).with_message(format!("This {class_kind_str} defines {own_template_parameters_len} template parameters...")))
            .with_annotation(Annotation::secondary(parent_definition_span).with_message(format!("...but parent `{parent_name_str}` is marked `@consistent-templates` and expects {expected_parameters_count}.")))
            .with_help("Ensure the number of template parameters on this {class_kind_str} matches its parent."),
        );
    }

    if expected_parameters_count > 0
        && let Some(extended_parameters) = class_like_metadata.template_extended_parameters.get(&parent_metadata.name)
    {
        let mut i = 0;
        let mut previous_extended_types: IndexMap<StringIdentifier, Vec<(GenericParent, TUnion)>, RandomState> =
            IndexMap::default();

        for (template_name, template_type_map) in &parent_metadata.template_types {
            let Some(extended_type) = extended_parameters.get(template_name) else {
                i += 1;
                continue;
            };

            let Some(template_type) = template_type_map.last().map(|(_, template_type)| template_type) else {
                i += 1;
                continue;
            };

            let template_name_str = context.interner.lookup(template_name);
            let extended_type_str = extended_type.get_id(Some(context.interner));

            if parent_metadata.template_variance.get(&i).is_some_and(|variance| variance.is_invariant()) {
                for extended_type_atomic in &extended_type.types {
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
                        .is_some_and(|variance| variance.is_covariant())
                    {
                        let child_template_name_str = context.interner.lookup(&generic_parameter.parameter_name);

                        context.collector.report_with_code(
                            Code::INVALID_TEMPLATE_PARAMETER,
                            Issue::error("Invalid template variance: cannot use a covariant template to satisfy an invariant one.")
                                .with_annotation(Annotation::primary(class_name_span).with_message(format!("In the definition of `{class_name_str}`")))
                                .with_note(format!("The parent `{parent_name_str}` defines template `{template_name_str}` as invariant (`@template`)."))
                                .with_note(format!("But it is being satisfied by the covariant template `{child_template_name_str}` (`@template-covariant`) from `{class_name_str}`."))
                                .with_help("Make the child template parameter invariant as well (`@template`), or change the parent's variance if appropriate."),
                        );
                    }
                }
            }

            if parent_metadata.has_consistent_templates {
                for extended_type_atomic in &extended_type.types {
                    let extended_as_template = extended_type_atomic.get_generic_parameter_name();
                    if extended_as_template.is_none() {
                        context.collector.report_with_code(
                            Code::INVALID_TEMPLATE_PARAMETER,
                            Issue::error("Inconsistent template: expected a template parameter, but found a concrete type.")
                                .with_annotation(Annotation::primary(parent_definition_span).with_message(format!(
                                    "Expected a template parameter, but got `{}`",
                                    extended_type.get_id(Some(context.interner)),
                                )))
                                .with_note(format!("Because `{parent_name_str}` is marked `@consistent-templates`, its template parameters must be extended with other template parameters, not concrete types."))
                                .with_help(format!("Change this to a template parameter defined on `{class_name_str}`.")),
                        );
                    } else if let Some(child_template_name) = extended_as_template
                        && let Some(child_template_map) = class_like_metadata.get_template_type(&child_template_name)
                        && let Some((_, child_template_type)) = child_template_map.last()
                        && child_template_type.get_id(None) != template_type.get_id(None)
                    {
                        context.collector.report_with_code(
                            Code::INVALID_TEMPLATE_PARAMETER,
                            Issue::error("Inconsistent template: template parameter constraints do not match.")
                                .with_annotation(Annotation::primary(class_name_span).with_message(format!("This template parameter has constraint `{}`...", child_template_type.get_id(Some(context.interner)))))
                                .with_annotation(Annotation::secondary(parent_definition_span).with_message(format!("...but parent `{parent_name_str}` requires a constraint of `{}` for this template.", template_type.get_id(Some(context.interner)))))
                                .with_note(format!("Because `{parent_name_str}` is marked `@consistent-templates`, the constraints of its template parameters must be identical in child classes."))
                                .with_help("Adjust the constraint on the child template parameter to match the parent's."),
                        );
                    }
                }
            }

            if !template_type.is_mixed() {
                let mut template_result = TemplateResult::new(previous_extended_types.clone(), Default::default());
                let replaced_template_type = standin_type_replacer::replace(
                    template_type,
                    &mut template_result,
                    context.codebase,
                    context.interner,
                    &None,
                    None,
                    None,
                    StandinOptions::default(),
                );

                if !union_comparator::is_contained_by(
                    context.codebase,
                    context.interner,
                    extended_type,
                    &replaced_template_type,
                    false,
                    false,
                    false,
                    &mut ComparisonResult::default(),
                ) {
                    let replaced_type_str = replaced_template_type.get_id(Some(context.interner));

                    context.collector.report_with_code(
                        Code::INVALID_TEMPLATE_PARAMETER,
                        Issue::error(format!("Template argument for `{parent_name_str}` is not compatible with its constraint."))
                            .with_annotation(Annotation::primary(class_name_span).with_message(format!("In the definition of `{class_name_str}`")))
                            .with_note(format!("The type `{extended_type_str}` provided for template `{template_name_str}`..."))
                            .with_note(format!("...does not satisfy the required constraint of `{replaced_type_str}` from `{parent_name_str}`."))
                            .with_help("Change the provided type to be compatible with the template constraint."),
                    );
                } else {
                    previous_extended_types
                        .entry(*template_name)
                        .or_default()
                        .push((GenericParent::ClassLike(class_like_metadata.name), extended_type.clone()));
                }
            } else {
                previous_extended_types
                    .entry(*template_name)
                    .or_default()
                    .push((GenericParent::ClassLike(class_like_metadata.name), extended_type.clone()));
            }

            i += 1;
        }
    }
}

fn check_class_like_properties(context: &mut Context<'_>, class_like_metadata: &ClassLikeMetadata) {
    if class_like_metadata.kind.is_enum() {
        return;
    }

    for (property, fqcn) in &class_like_metadata.appearing_property_ids {
        let Some(declaring_property) = get_declaring_property(context.codebase, context.interner, fqcn, property)
        else {
            continue;
        };

        if let Some(parents_fqcn) = class_like_metadata.overridden_property_ids.get(property) {
            for parent_fqcn in parents_fqcn {
                let Some(parent_metadata) = get_class_like(context.codebase, context.interner, parent_fqcn) else {
                    continue;
                };

                let Some(parent_property) = parent_metadata.properties.get(property) else {
                    continue;
                };

                if declaring_property.read_visibility > parent_property.read_visibility
                    && let Some(property_span) = declaring_property.span
                    && let Some(parent_property_span) = parent_property.span
                {
                    let property_name_str = context.interner.lookup(property);
                    let declaring_class_name_str = context.interner.lookup(&class_like_metadata.original_name);
                    let parent_class_name_str = context.interner.lookup(&parent_metadata.original_name);

                    context.collector.report_with_code(
                        Code::OVERRIDDEN_PROPERTY_ACCESS,
                        Issue::error(format!(
                            "Property `{declaring_class_name_str}::{property_name_str}` has a different read access level than `{parent_class_name_str}::{property_name_str}`."
                        ))
                        .with_annotation(
                            Annotation::primary(property_span)
                                .with_message(format!("This property is declared as `{}`", declaring_property.read_visibility.as_str())),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_property_span)
                                .with_message(format!("Parent property is declared as `{}`", parent_property.read_visibility.as_str())),
                        )
                        .with_note("The access level of an overridden property must not be more restrictive than the parent property.")
                        .with_help("Adjust the access level of the property in the child class to match or be less restrictive than the parent class."),
                    );
                }

                if (declaring_property.write_visibility != declaring_property.read_visibility
                    || parent_property.write_visibility != parent_property.read_visibility)
                    && declaring_property.write_visibility > parent_property.write_visibility
                    && let Some(property_span) = declaring_property.span
                    && let Some(parent_property_span) = parent_property.span
                {
                    let property_name_str = context.interner.lookup(property);
                    let declaring_class_name_str = context.interner.lookup(&class_like_metadata.original_name);
                    let parent_class_name_str = context.interner.lookup(&parent_metadata.original_name);

                    context.collector.report_with_code(
                        Code::OVERRIDDEN_PROPERTY_ACCESS,
                        Issue::error(format!(
                            "Property `{declaring_class_name_str}::{property_name_str}` has a different write access level than `{parent_class_name_str}::{property_name_str}`."
                        ))
                        .with_annotation(
                            Annotation::primary(property_span)
                                .with_message(format!("This property is declared as `{}(set)`", declaring_property.write_visibility.as_str())),
                        )
                        .with_annotation(
                            Annotation::secondary(parent_property_span)
                                .with_message(format!("Parent property is declared as `{}(set)`", parent_property.write_visibility.as_str())),
                        )
                        .with_note("The access level of an overridden property must not be more restrictive than the parent property.")
                        .with_help("Adjust the access level of the property in the child class to match or be less restrictive than the parent class."),
                    );
                }

                let mut has_type_incompatibility = false;
                match (
                    declaring_property.type_declaration_metadata.as_ref(),
                    parent_property.type_declaration_metadata.as_ref(),
                ) {
                    (Some(declaring_type), Some(parent_type)) => {
                        let contains_parent = union_comparator::is_contained_by(
                            context.codebase,
                            context.interner,
                            &declaring_type.type_union,
                            &parent_type.type_union,
                            false,
                            false,
                            false,
                            &mut ComparisonResult::default(),
                        );

                        let contains_declaring = union_comparator::is_contained_by(
                            context.codebase,
                            context.interner,
                            &parent_type.type_union,
                            &declaring_type.type_union,
                            false,
                            false,
                            false,
                            &mut ComparisonResult::default(),
                        );

                        let is_wider = contains_parent && !contains_declaring;
                        let is_narrower = contains_declaring && !contains_parent;
                        if is_wider || is_narrower {
                            has_type_incompatibility = true;

                            let declaring_type_id = declaring_type.type_union.get_id(Some(context.interner));
                            let parent_type_id = parent_type.type_union.get_id(Some(context.interner));
                            let property_name_str = context.interner.lookup(&declaring_property.name.0);
                            let class_name_str = context.interner.lookup(&class_like_metadata.original_name);

                            context.collector.report_with_code(
                                Code::INCOMPATIBLE_PROPERTY_TYPE,
                                Issue::error(format!(
                                    "Property `{class_name_str}::{property_name_str}` has an incompatible type declaration."
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
                                .with_help(format!("Change the type of `{property_name_str}` to `{parent_type_id}` to match the parent property."))
                            );
                        }
                    }
                    (Some(declaring_type), None) => {
                        has_type_incompatibility = true;

                        let property_name_str = context.interner.lookup(&declaring_property.name.0);
                        let class_name_str = context.interner.lookup(&class_like_metadata.original_name);

                        let mut issue = Issue::error(format!(
                            "Property `{class_name_str}::{property_name_str}` adds a type that is missing on the parent property."
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
                        };

                        context.collector.report_with_code(Code::INCOMPATIBLE_PROPERTY_TYPE, issue
                            .with_note("Adding a type to a property that was untyped in a parent class is an incompatible change.")
                                   .with_help("You can either remove the type from this property or add an identical type to the property in the parent class."));
                    }
                    (None, Some(parent_type)) => {
                        has_type_incompatibility = true;

                        if let Some(property_span) = declaring_property.name_span {
                            let property_name_str = context.interner.lookup(&declaring_property.name.0);
                            let class_name_str = context.interner.lookup(&class_like_metadata.original_name);
                            let parent_type_id = parent_type.type_union.get_id(Some(context.interner));

                            context.collector.report_with_code(
                                Code::INCOMPATIBLE_PROPERTY_TYPE,
                                Issue::error(format!(
                                    "Property `{class_name_str}::{property_name_str}` is missing the type declaration from its parent."
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
                    && let Some(declaring_type) = &declaring_property.type_metadata
                    && declaring_type.from_docblock
                    && let Some(parent_type) = &parent_property.type_metadata
                    && (!union_comparator::is_contained_by(
                        context.codebase,
                        context.interner,
                        &declaring_type.type_union,
                        &parent_type.type_union,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::default(),
                    ) || !union_comparator::is_contained_by(
                        context.codebase,
                        context.interner,
                        &parent_type.type_union,
                        &declaring_type.type_union,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::default(),
                    ))
                {
                    let declaring_type_id = declaring_type.type_union.get_id(Some(context.interner));
                    let parent_type_id = parent_type.type_union.get_id(Some(context.interner));
                    let property_name_str = context.interner.lookup(&declaring_property.name.0);
                    let class_name_str = context.interner.lookup(&class_like_metadata.original_name);

                    context.collector.report_with_code(
                        Code::INCOMPATIBLE_PROPERTY_TYPE,
                        Issue::error(format!(
                            "Property `{class_name_str}::{property_name_str}` has an incompatible type declaration from docblock."
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
                        .with_help(format!("Change the type of `{property_name_str}` to `{parent_type_id}` to match the parent property.")),
                    );
                }
            }
        }
    }
}
