use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
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
use mago_names::kind::NameKind;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::heuristic;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

pub mod constant;
pub mod enum_case;
pub mod method;
pub mod property;

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
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, name) else {
            tracing::warn!("Class {} not found in codebase", name);

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

        heuristic::check_class_like(class_like_metadata, self.members.as_slice(), context);

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
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, name) else {
            tracing::warn!("Interface {name} not found in codebase");

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

        heuristic::check_class_like(class_like_metadata, self.members.as_slice(), context);

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
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, name) else {
            tracing::warn!("Trait {} not found in codebase", name);

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

        heuristic::check_class_like(class_like_metadata, self.members.as_slice(), context);

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
        )?;

        let name = context.resolved_names.get(&self.name);
        let Some(class_like_metadata) = get_class_like(context.codebase, name) else {
            tracing::warn!("Enum {} not found in codebase", name);

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

        heuristic::check_class_like(class_like_metadata, self.members.as_slice(), context);

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

    let name = &class_like_metadata.name;

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
        for (method_name, fqcn) in &class_like_metadata.declaring_method_ids {
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

            let Some(declaring_class_like_metadata) = get_class_like(context.codebase, fqcn) else {
                continue;
            };

            let Some(function_like) = get_method(context.codebase, fqcn, method_name) else {
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
    }

    if !class_like_metadata.template_types.is_empty() {
        for (template_name, _) in &class_like_metadata.template_types {
            let (resolved_template_name, _) = context.scope.resolve(NameKind::Default, template_name);
            if let Some(conflicting_class) = get_class_like(context.codebase, &resolved_template_name) {
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

    for extended_type in extends.types.iter() {
        let extended_type_str = context.resolved_names.get(&extended_type);
        let extended_class_metadata = get_class_like(context.codebase, extended_type_str);

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

    for implemented_type in implements.types.iter() {
        let implemented_type_str = context.resolved_names.get(&implemented_type);
        let implemented_interface_metadata = get_class_like(context.codebase, implemented_type_str);

        match implemented_interface_metadata {
            Some(implemented_metadata) => {
                let implemented_name = implemented_metadata.original_name;
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
            }
            None => {
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

    for used_type in trait_use.trait_names.iter() {
        let used_type_str = context.resolved_names.get(&used_type);
        let used_trait_metadata = get_class_like(context.codebase, used_type_str);

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

fn check_template_parameters<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
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

        for (template_name, template_type_map) in &parent_metadata.template_types {
            let Some(extended_type) = extended_parameters.get(template_name) else {
                i += 1;
                continue;
            };

            let Some(template_type) = template_type_map.last().map(|(_, template_type)| template_type) else {
                i += 1;
                continue;
            };

            let extended_type_str = extended_type.get_id();

            if parent_metadata.template_variance.get(&i).is_some_and(|variance| variance.is_invariant()) {
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
                        .is_some_and(|variance| variance.is_covariant())
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

            if !template_type.is_mixed() {
                let mut template_result = TemplateResult::new(previous_extended_types.clone(), Default::default());
                let replaced_template_type = standin_type_replacer::replace(
                    template_type,
                    &mut template_result,
                    context.codebase,
                    &None,
                    None,
                    None,
                    StandinOptions::default(),
                );

                if !union_comparator::is_contained_by(
                    context.codebase,
                    extended_type,
                    &replaced_template_type,
                    false,
                    false,
                    false,
                    &mut ComparisonResult::default(),
                ) {
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

fn check_class_like_properties<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    class_like_metadata: &'ctx ClassLikeMetadata,
) {
    if class_like_metadata.kind.is_enum() {
        return;
    }

    for (property, fqcn) in &class_like_metadata.appearing_property_ids {
        let Some(declaring_property) = get_declaring_property(context.codebase, fqcn, property) else {
            continue;
        };

        if let Some(parents_fqcn) = class_like_metadata.overridden_property_ids.get(property) {
            for parent_fqcn in parents_fqcn {
                let Some(parent_metadata) = get_class_like(context.codebase, parent_fqcn) else {
                    continue;
                };

                let Some(parent_property) = parent_metadata.properties.get(property) else {
                    continue;
                };

                if declaring_property.read_visibility > parent_property.read_visibility
                    && let Some(property_span) = declaring_property.span
                    && let Some(parent_property_span) = parent_property.span
                {
                    let declaring_class_name = class_like_metadata.original_name;
                    let parent_class_name = parent_metadata.original_name;

                    context.collector.report_with_code(
                        IssueCode::OverriddenPropertyAccess,
                        Issue::error(format!(
                            "Property `{declaring_class_name}::{property}` has a different read access level than `{parent_class_name}::{property}`."
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
                    let declaring_class_name = class_like_metadata.original_name;
                    let parent_class_name = parent_metadata.original_name;

                    context.collector.report_with_code(
                        IssueCode::OverriddenPropertyAccess,
                        Issue::error(format!(
                            "Property `{declaring_class_name}::{property}` has a different write access level than `{parent_class_name}::{property}`."
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
                            &declaring_type.type_union,
                            &parent_type.type_union,
                            false,
                            false,
                            false,
                            &mut ComparisonResult::default(),
                        );

                        let contains_declaring = union_comparator::is_contained_by(
                            context.codebase,
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

                            let declaring_type_id = declaring_type.type_union.get_id();
                            let parent_type_id = parent_type.type_union.get_id();
                            let property_name = declaring_property.name.0;
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

                        let property_name = declaring_property.name.0;
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
                        };

                        context.collector.report_with_code(IssueCode::IncompatiblePropertyType, issue
                            .with_note("Adding a type to a property that was untyped in a parent class is an incompatible change.")
                                   .with_help("You can either remove the type from this property or add an identical type to the property in the parent class."));
                    }
                    (None, Some(parent_type)) => {
                        has_type_incompatibility = true;

                        if let Some(property_span) = declaring_property.name_span {
                            let property_name = declaring_property.name.0;
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
                    && let Some(declaring_type) = &declaring_property.type_metadata
                    && declaring_type.from_docblock
                    && let Some(parent_type) = &parent_property.type_metadata
                    && (!union_comparator::is_contained_by(
                        context.codebase,
                        &declaring_type.type_union,
                        &parent_type.type_union,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::default(),
                    ) || !union_comparator::is_contained_by(
                        context.codebase,
                        &parent_type.type_union,
                        &declaring_type.type_union,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::default(),
                    ))
                {
                    let declaring_type_id = declaring_type.type_union.get_id();
                    let parent_type_id = parent_type.type_union.get_id();
                    let property_name = declaring_property.name.0;
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
        }
    }
}
