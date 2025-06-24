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
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
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

    check_class_like_extends(context, class_like_metadata, extends_ast)?;
    check_class_like_implements(context, class_like_metadata, implements_ast)?;

    if !class_like_metadata.invalid_dependencies.is_empty() {
        return Ok(());
    }

    if !class_like_metadata.kind.is_trait() && !class_like_metadata.is_abstract() {
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

            if method_metadata.is_abstract() {
                let method_name_str = context.interner.lookup(method_name);
                let fqcn_str = context.interner.lookup(&declaring_class_like_metadata.original_name);
                let method_span = function_like.get_name_span().unwrap_or_else(|| function_like.get_span());

                context.buffer.report(
                    TypingIssueKind::UnimplementedAbstractMethod,
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
                let conflicting_class_span =
                    conflicting_class.get_name_span().unwrap_or_else(|| conflicting_class.get_span());

                context.buffer.report(
                    TypingIssueKind::NameAlreadyInUse,
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

    check_class_like_properties(context, class_like_metadata)?;

    let mut block_context = BlockContext::new({
        let mut scope = ScopeContext::new();

        scope.set_class_like(Some(class_like_metadata));
        scope.set_static(true);
        scope
    });

    for member in members {
        match member {
            ClassLikeMember::TraitUse(_) => {
                // todo!
            }
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
        }
    }

    Ok(())
}

fn check_class_like_extends(
    context: &mut Context<'_>,
    class_like_metadata: &ClassLikeMetadata,
    extends_ast: Option<&Extends>,
) -> Result<(), AnalysisError> {
    let extending_class = class_like_metadata.is_class();
    let extending_interface = class_like_metadata.is_interface();

    if !extending_class && !extending_interface {
        return Ok(());
    }

    if let Some(extends) = extends_ast {
        for extended_type in extends.types.iter() {
            let extended_type_id = context.resolved_names.get(&extended_type);
            let extended_class_metadata = get_class_like(context.codebase, context.interner, extended_type_id);

            let Some(extended_class_metadata) = extended_class_metadata else {
                context.buffer.report(
                    TypingIssueKind::NonExistentClassLike,
                    Issue::error(format!(
                        "Cannot extend unknown {}: `{}` not found.",
                        if extending_class { "class" } else { "interface" },
                        context.interner.lookup(extended_type.value()),
                    ))
                    .with_annotation(Annotation::primary(extended_type.span()).with_message("This class or interface could not be found"))
                    .with_help("Ensure the name is correct, including its namespace, and that it's properly defined and autoloadable."),
                );

                continue;
            };

            let extending_name_str = context.interner.lookup(&class_like_metadata.original_name);
            let extending_class_span =
                class_like_metadata.get_name_span().unwrap_or_else(|| class_like_metadata.get_span());
            let extended_name_str = context.interner.lookup(&extended_class_metadata.original_name);
            let extended_kind_str = extended_class_metadata.get_kind().as_str();
            let extended_kind_prefix = if extended_class_metadata.is_class_or_trait() { "a" } else { "an" };
            let extended_class_span =
                extended_class_metadata.get_name_span().unwrap_or_else(|| extended_class_metadata.get_span());

            if extending_interface {
                if !extended_class_metadata.is_interface() {
                    context.buffer.report(
                        TypingIssueKind::InvalidExtend,
                        Issue::error(format!("Interface `{extending_name_str}` cannot extend {extended_kind_prefix} {extended_kind_str}."))
                            .with_annotation(
                                Annotation::primary(extended_type.span())
                                    .with_message(format!("`{extended_name_str}` is {extended_kind_prefix} {extended_kind_str}, not an interface"))
                            )
                            .with_note("In PHP, interfaces can only extend other interfaces.")
                            .with_help("Change the extended type to be an interface, or change this declaration to a class if you intended to extend a class."),
                    );

                    continue;
                }

                if extended_class_metadata.is_enum_interface && !class_like_metadata.is_enum_interface {
                    context.buffer.report(
                        TypingIssueKind::InvalidExtend,
                        Issue::error(format!("Interface `{extending_name_str}` cannot extend enum interface `{extended_name_str}`."))
                            .with_annotation(Annotation::primary(extended_type.span()).with_message("This interface is an enum interface"))
                            .with_note("An enum interface cannot be extended by a non-enum interface.")
                            .with_help("Change the extended type to be a regular interface, or add `@enum-interface` to the extending interface."),
                    );
                }

                check_template_parameters(
                    context,
                    class_like_metadata,
                    extended_class_metadata,
                    class_like_metadata
                        .template_type_extends_count
                        .get(&extended_class_metadata.name)
                        .copied()
                        .unwrap_or(0),
                    InheritanceKind::Extends(extended_type.span()),
                )?;
            }

            if extending_class {
                if !extended_class_metadata.is_class() {
                    context.buffer.report(
                        TypingIssueKind::InvalidExtend,
                        Issue::error(format!("Class `{extending_name_str}` cannot extend {extended_kind_prefix} {extended_kind_str}."))
                            .with_annotation(
                                Annotation::primary(extended_type.span())
                                    .with_message(format!("`{extended_name_str}` is {extended_kind_prefix} {extended_kind_str}, not a class"))
                            )
                            .with_note("In PHP, classes can only extend other classes.")
                            .with_help("Change the extended type to be a class, or change this declaration to an interface if you intended to extend an interface."),
                    );

                    continue;
                }

                if extended_class_metadata.is_final() {
                    context.buffer.report(
                        TypingIssueKind::InvalidExtend,
                        Issue::error(format!("Class `{extending_name_str}` cannot extend final class `{extended_name_str}`."))
                            .with_annotation(Annotation::primary(extended_type.span()).with_message("This class is declared as final"))
                            .with_annotation(Annotation::secondary(extended_class_span).with_message("This class is marked `final` here"))
                            .with_note("A class declared as `final` cannot be extended.")
                            .with_help("Remove the `final` keyword from the class declaration, or change the class you are extending to a non-final class."),
                    );
                }

                if extended_class_metadata.is_readonly && !class_like_metadata.is_readonly {
                    context.buffer.report(
                        TypingIssueKind::InvalidExtend,
                        Issue::error(format!("Class `{extending_name_str}` cannot extend readonly class `{extended_name_str}`."))
                            .with_annotation(Annotation::primary(extended_type.span()).with_message("This class is declared as readonly"))
                            .with_annotation(Annotation::secondary(extended_class_span).with_message("This class is marked `readonly` here"))
                            .with_note("A class declared as `readonly` cannot be extended by a non-readonly class.")
                            .with_help("Remove the `readonly` keyword from the class declaration, or change the class you are extending to a non-readonly class."),
                    );
                }

                if extended_class_metadata.is_deprecated {
                    context.buffer.report(
                        TypingIssueKind::DeprecatedClass,
                        Issue::error(format!("Class `{extending_name_str}` cannot extend deprecated class `{extended_name_str}`."))
                            .with_annotation(Annotation::primary(extended_type.span()).with_message("This class is marked as deprecated"))
                            .with_note("The parent class is deprecated and may be removed in a future version, which would break this child class.")
                            .with_help("Consider refactoring to avoid extending this class, or consult its documentation for alternatives."),
                    );
                }

                if extended_class_metadata.is_external_mutation_free && !class_like_metadata.is_external_mutation_free {
                    context.buffer.report(
                        TypingIssueKind::InvalidExtend,
                        Issue::error("A mutable class cannot extend a `@external-mutation-free` class.")
                            .with_annotation(
                                Annotation::primary(extending_class_span).with_message("This class is mutable..."),
                            )
                            .with_annotation(Annotation::secondary(extended_type.span()).with_message(format!(
                                "...but extends `{extended_name_str}` which is `@external-mutation-free`",
                            )))
                            .with_help(format!(
                                "Mark `{extending_name_str}` as `@external-mutation-free` or remove the annotation from the parent.",
                            )),
                    );
                }

                if extended_class_metadata.is_mutation_free && !class_like_metadata.is_mutation_free {
                    context.buffer.report(
                        TypingIssueKind::InvalidExtend,
                        Issue::error("A mutable class cannot extend a `@mutation-free` class.")
                            .with_annotation(
                                Annotation::primary(extending_class_span).with_message("This class is mutable..."),
                            )
                            .with_annotation(Annotation::secondary(extended_type.span()).with_message(format!(
                                "...but extends `{extended_name_str}` which is `@mutation-free`",
                            )))
                            .with_help(format!(
                                "Mark `{extending_name_str}` as `@mutation-free` or remove the annotation from the parent.",
                            )),
                    );
                }

                if !class_like_metadata.is_abstract {
                    for required_interface in &extended_class_metadata.require_implements {
                        if !class_like_metadata.has_parent_interface(required_interface) {
                            let required_iface_str = context.interner.lookup(required_interface);

                            context.buffer.report(
                                TypingIssueKind::MissingRequiredInterface,
                                Issue::error(format!(
                                    "Class `{extending_name_str}` must implement interface `{required_iface_str}` as required by its parent.",
                                ))
                                .with_annotation(
                                    Annotation::primary(extending_class_span).with_message(format!(
                                        "`{extending_name_str}` is missing a required interface",
                                    )),
                                )
                                .with_annotation(
                                    Annotation::secondary(
                                        extended_class_span
                                    )
                                    .with_message(format!(
                                        "Parent class `{extended_name_str}` requires descendants to implement `{required_iface_str}`",
                                    )),
                                )
                                .with_help(format!(
                                    "Add `implements {required_iface_str}` to the `{extending_name_str}` class definition.",
                                )),
                            );
                        }
                    }
                }

                if !class_like_metadata.is_abstract()
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
                    let class_like_name_str = context.interner.lookup(&class_like_metadata.original_name);
                    let extended_name_str = context.interner.lookup(&extended_class_metadata.original_name);

                    context.buffer.report(
                        TypingIssueKind::InvalidImplement,
                        Issue::error(format!(
                            "Class `{class_like_name_str}` cannot extend class `{extended_name_str}` because it is not listed in the `@inheritors` annotation."
                        ))
                        .with_annotation(
                            Annotation::primary(extended_type.span())
                                .with_message(format!("The class `{extended_name_str}` does not permit `{class_like_name_str}` to extend it")),
                        )
                        .with_note("The `@inheritors` annotation restricts which classes can extend this class.")
                        .with_help(format!(
                            "Add `{class_like_name_str}` to the `@inheritors` annotation of `{extended_name_str}`."
                        )),
                    );
                }

                check_template_parameters(
                    context,
                    class_like_metadata,
                    extended_class_metadata,
                    class_like_metadata
                        .template_type_extends_count
                        .get(&extended_class_metadata.name)
                        .copied()
                        .unwrap_or(0),
                    InheritanceKind::Extends(extended_type.span()),
                )?;
            }
        }
    }

    Ok(())
}

fn check_class_like_implements(
    context: &mut Context<'_>,
    class_like_metadata: &ClassLikeMetadata,
    implements_ast: Option<&Implements>,
) -> Result<(), AnalysisError> {
    if !class_like_metadata.is_class() && !class_like_metadata.is_enum() {
        // Interfaces and traits cannot implement interfaces
        // This will be caught by the semantic analyzer
        return Ok(());
    }

    let Some(implements) = implements_ast else {
        // No interfaces to implement, nothing to validate
        return Ok(());
    };

    for implemented_type in implements.types.iter() {
        let implemented_type_id = context.resolved_names.get(&implemented_type);
        let implemented_interface_metadata = get_class_like(context.codebase, context.interner, implemented_type_id);

        match implemented_interface_metadata {
            Some(implemented_metadata) => {
                if !implemented_metadata.is_interface() {
                    let class_name_str = context.interner.lookup(&class_like_metadata.original_name);
                    let implemented_name_str = context.interner.lookup(&implemented_metadata.original_name);
                    let implemented_kind_str = implemented_metadata.get_kind().as_str();
                    let implemented_kind_prefix = if implemented_metadata.is_class_or_trait() { "a" } else { "an" };

                    context.buffer.report(
                        TypingIssueKind::InvalidImplement,
                        Issue::error(format!("Cannot implement `{implemented_name_str}` because it is not an interface."))
                            .with_annotation(Annotation::primary(implemented_type.span()).with_message(format!("`{implemented_name_str}` is {implemented_kind_prefix} {implemented_kind_str}, not an interface")))
                            .with_note("The `implements` keyword can only be used with interfaces. To inherit from a class, use `extends`.".to_string())
                            .with_help(format!("Change `{implemented_name_str}` to be an interface or remove it from the `implements` list of `{class_name_str}`.")),
                    );

                    continue;
                }

                if implemented_metadata.is_enum_interface && !class_like_metadata.kind.is_enum() {
                    let class_name_str = context.interner.lookup(&class_like_metadata.original_name);
                    let implemented_name_str = context.interner.lookup(&implemented_metadata.original_name);

                    context.buffer.report(
                        TypingIssueKind::InvalidImplement,
                        Issue::error(format!(
                            "Cannot implement enum interface `{implemented_name_str}` in a non-enum `{class_name_str}`."
                        ))
                        .with_annotation(
                            Annotation::primary(implemented_type.span())
                                .with_message("This interface is an enum interface"),
                        )
                        .with_annotation(
                            Annotation::secondary(
                                implemented_metadata.get_name_span().unwrap_or(implemented_metadata.get_span()),
                            )
                            .with_message("This interface is marked with `@enum-interface`"),
                        )
                        .with_note("An enum interface can only be implemented by an enums.")
                        .with_help("Change this class to be an enum, or remove the `implements` clause."),
                    );
                }

                if !class_like_metadata.is_abstract()
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
                    let class_like_name_str = context.interner.lookup(&class_like_metadata.original_name);
                    let implemented_name_str = context.interner.lookup(&implemented_metadata.original_name);

                    context.buffer.report(
                        TypingIssueKind::InvalidImplement,
                        Issue::error(format!(
                            "Class `{class_like_name_str}` cannot implement interface `{implemented_name_str}` because it is not listed in the `@inheritors` annotation."
                        ))
                        .with_annotation(
                            Annotation::primary(implemented_type.span())
                                .with_message(format!("The interface `{implemented_name_str}` does not permit `{class_like_name_str}` to implement it")),
                        )
                        .with_note("The `@inheritors` annotation restricts which classes can implement this interface.")
                        .with_help(format!(
                            "Add `{class_like_name_str}` to the `@inheritors` annotation of `{implemented_name_str}`."
                        )),
                    );
                }

                check_template_parameters(
                    context,
                    class_like_metadata,
                    implemented_metadata,
                    class_like_metadata
                        .template_type_implements_count
                        .get(&implemented_metadata.name)
                        .copied()
                        .unwrap_or(0),
                    InheritanceKind::Implements(implemented_type.span()),
                )?;
            }
            None => {
                let class_name_str = context.interner.lookup(&class_like_metadata.original_name);
                let implemented_name_str = context.interner.lookup(implemented_type.value());

                context.buffer.report(
                    TypingIssueKind::NonExistentClassLike,
                    Issue::error(format!("Cannot implement unknown interface `{implemented_name_str}`."))
                        .with_annotation(Annotation::primary(implemented_type.span()).with_message("This interface could not be found"))
                        .with_note("Ensure the name is correct, including its namespace, and that it's properly defined and autoloadable.")
                        .with_help(format!("Check if `{implemented_name_str}` is defined and accessible in the current context of `{class_name_str}`.")),
                );
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InheritanceKind {
    Extends(Span),
    Implements(Span),
    #[allow(dead_code)]
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
) -> Result<(), AnalysisError> {
    let expected_parameters_count = parent_metadata.get_template_types().len();

    let class_name_str = context.interner.lookup(&class_like_metadata.original_name);
    let class_kind_str = class_like_metadata.get_kind().as_str();
    let parent_name_str = context.interner.lookup(&parent_metadata.original_name);
    let class_name_span = class_like_metadata.get_name_span().unwrap_or_else(|| class_like_metadata.get_span());
    let parent_definition_span = parent_metadata.get_name_span().unwrap_or_else(|| parent_metadata.get_span());
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

        context.buffer.report(TypingIssueKind::MissingTemplateParameter, issue);
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

        context.buffer.report(TypingIssueKind::ExcessTemplateParameter, issue);
    }

    let own_template_parameters_len = class_like_metadata.get_template_types().len();
    if parent_metadata.has_consistent_templates() && own_template_parameters_len != expected_parameters_count {
        context.buffer.report(
            TypingIssueKind::InconsistentTemplate,
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

        for (template_name, template_type_map) in parent_metadata.get_template_types() {
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

            if parent_metadata.get_template_variance_for_index(i).is_some_and(|variance| variance.is_invariant()) {
                for extended_type_atomic in &extended_type.types {
                    let TAtomic::GenericParameter(generic_parameter) = extended_type_atomic else {
                        continue;
                    };

                    let Some(local_offset) = class_like_metadata
                        .get_template_types()
                        .iter()
                        .position(|(name, _)| *name == generic_parameter.parameter_name)
                    else {
                        continue;
                    };

                    if class_like_metadata
                        .get_template_variance_for_index(local_offset)
                        .is_some_and(|variance| variance.is_covariant())
                    {
                        let child_template_name_str = context.interner.lookup(&generic_parameter.parameter_name);

                        context.buffer.report(
                            TypingIssueKind::InvalidTemplateParameter,
                            Issue::error("Invalid template variance: cannot use a covariant template to satisfy an invariant one.")
                                .with_annotation(Annotation::primary(class_name_span).with_message(format!("In the definition of `{class_name_str}`")))
                                .with_note(format!("The parent `{parent_name_str}` defines template `{template_name_str}` as invariant (`@template`)."))
                                .with_note(format!("But it is being satisfied by the covariant template `{child_template_name_str}` (`@template-covariant`) from `{class_name_str}`."))
                                .with_help("Make the child template parameter invariant as well (`@template`), or change the parent's variance if appropriate."),
                        );
                    }
                }
            }

            if parent_metadata.has_consistent_templates() {
                for extended_type_atomic in &extended_type.types {
                    let extended_as_template = extended_type_atomic.get_generic_parameter_name();
                    if extended_as_template.is_none() {
                        context.buffer.report(
                            TypingIssueKind::InvalidTemplateParameter,
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
                        context.buffer.report(
                            TypingIssueKind::InvalidTemplateParameter,
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

                    context.buffer.report(
                        TypingIssueKind::InvalidTemplateParameter,
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

    Ok(())
}

fn check_class_like_properties(
    context: &mut Context<'_>,
    class_like_metadata: &ClassLikeMetadata,
) -> Result<(), AnalysisError> {
    if class_like_metadata.kind.is_enum() {
        return Ok(());
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

                    context.buffer.report(
                        TypingIssueKind::OverriddenPropertyAccess,
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

                    context.buffer.report(
                        TypingIssueKind::OverriddenPropertyAccess,
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

                            context.buffer.report(
                                TypingIssueKind::IncompatiblePropertyType,
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

                        context.buffer.report(TypingIssueKind::IncompatiblePropertyType, issue
                            .with_note("Adding a type to a property that was untyped in a parent class is an incompatible change.")
                                   .with_help("You can either remove the type from this property or add an identical type to the property in the parent class."));
                    }
                    (None, Some(parent_type)) => {
                        has_type_incompatibility = true;

                        if let Some(property_span) = declaring_property.name_span {
                            let property_name_str = context.interner.lookup(&declaring_property.name.0);
                            let class_name_str = context.interner.lookup(&class_like_metadata.original_name);
                            let parent_type_id = parent_type.type_union.get_id(Some(context.interner));

                            context.buffer.report(
                                TypingIssueKind::IncompatiblePropertyType,
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

                    context.buffer.report(
                        TypingIssueKind::IncompatiblePropertyType,
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = template_parameter_sanity_check,
        code = indoc! {r#"
            <?php

            /**
             * @template Tk of array-key
             * @template Tv
             */
            interface CollectionInterface
            {
            }

            /**
             * @template Tk of array-key
             * @template Tv
             */
            interface IndexAccessInterface
            {
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends CollectionInterface<Tk, Tv>
             * @extends IndexAccessInterface<Tk, Tv>
             */
            interface AccessibleCollectionInterface extends CollectionInterface, IndexAccessInterface
            {
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends IndexAccessInterface<Tk, Tv>
             */
            interface MutableIndexAccessInterface extends IndexAccessInterface
            {
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends CollectionInterface<Tk, Tv>
             */
            interface MutableCollectionInterface extends CollectionInterface
            {
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends AccessibleCollectionInterface<Tk, Tv>
             * @extends MutableCollectionInterface<Tk, Tv>
             * @extends MutableIndexAccessInterface<Tk, Tv>
             */
            interface MutableAccessibleCollectionInterface extends
                AccessibleCollectionInterface,
                MutableCollectionInterface,
                MutableIndexAccessInterface
            {
            }

            /**
             * @template T of array-key
             *
             * @extends AccessibleCollectionInterface<T, T>
             */
            interface SetInterface extends AccessibleCollectionInterface
            {
            }

            /**
             * @template T of array-key
             *
             * @extends SetInterface<T>
             * @extends MutableAccessibleCollectionInterface<T, T>
             */
            interface MutableSetInterface extends MutableAccessibleCollectionInterface, SetInterface
            {
            }
        "#}
    }
}
