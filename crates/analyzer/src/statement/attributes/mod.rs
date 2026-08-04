use foldhash::HashMap;
use foldhash::fast::RandomState;
use indexmap::IndexMap;
use mago_allocator::Arena;

use mago_codex::context::ScopeContext;
use mago_codex::flags::attribute::AttributeFlags;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::template::TemplateResult;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::Attribute;
use mago_syntax::cst::AttributeList;
use mago_word::WordMap;
use mago_word::word;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::MethodTargetContext;
use crate::invocation::analyzer::analyze_invocation;
use crate::visibility::check_method_visibility;
use mago_bytes::BytesDisplay;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AttributeTarget {
    ClassLike,
    Method,
    Property,
    Parameter,
    PromotedProperty,
    ClassLikeConstant,
    Function,
    Constant,
}

impl AttributeTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClassLike => "a class, interface, enum, or trait",
            Self::Method => "a method",
            Self::Property => "a property",
            Self::Parameter => "a parameter",
            Self::PromotedProperty => "a promoted property",
            Self::ClassLikeConstant => "a class constant",
            Self::Function => "a function",
            Self::Constant => "a constant",
        }
    }
}

/// Analyzes the attributes attached to a class-like declaration.
///
/// Attribute arguments on a class-like are evaluated in the scope of that
/// class-like, so `self::` and `parent::` must resolve against it rather than
/// against the enclosing scope.
pub fn analyze_class_like_attributes<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    artifacts: &mut AnalysisArtifacts,
    attribute_lists: &[AttributeList<'arena>],
    class_like_metadata: &'ctx ClassLikeMetadata,
) -> Result<(), AnalysisError>
where
    A: Arena,
{
    if attribute_lists.iter().all(|list| list.attributes.is_empty()) {
        return Ok(());
    }

    let mut scope = ScopeContext::new();
    scope.set_class_like(Some(class_like_metadata));
    scope.set_static(true);

    let mut block_context = BlockContext::new(scope, context.settings.register_super_globals);
    block_context.flags.set_inside_class_like_attribute(true);

    analyze_attributes(context, &mut block_context, artifacts, attribute_lists, AttributeTarget::ClassLike)
}

pub fn analyze_attributes<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    attribute_lists: &[AttributeList<'arena>],
    target: AttributeTarget,
) -> Result<(), AnalysisError>
where
    A: Arena,
{
    let attributes = attribute_lists.iter().flat_map(|list| list.attributes.iter()).collect::<Vec<_>>();

    let mut used_attributes = HashMap::default();
    for attribute in attributes {
        let attribute_name_bytes = context.resolved_names.get(&attribute.name);
        let attribute_name = BytesDisplay(attribute_name_bytes);

        let Some(metadata) = context.codebase.get_class_like(attribute_name_bytes) else {
            context.collector.report_with_code(
                IssueCode::NonExistentAttributeClass,
                Issue::error(format!("Attribute class `{attribute_name}` not found or could not be autoloaded."))
                .with_annotation(
                    Annotation::primary(attribute.name.span()).with_message(format!("Unknown attribute class `{attribute_name}`")),
                )
                .with_note("Attributes must be classes that are defined, correctly namespaced, and autoloadable. Ensure the class exists and is accessible.")
                .with_help("Verify the attribute class name, its namespace, and your autoloader configuration. Make sure the class is defined."),
            );

            analyze_attribute_argument_expressions(attribute, context, block_context, artifacts)?;
            continue;
        };

        let class_like_kind_str = metadata.kind.as_str();

        if !metadata.kind.is_class() {
            context.collector.report_with_code(
                IssueCode::NonClassUsedAsAttribute,
                Issue::error(format!(
                    "The {class_like_kind_str} `{attribute_name}` cannot be used as an attribute.",
                ))
                .with_annotation(
                    Annotation::primary(attribute.name.span())
                        .with_message(format!(
                            "`{attribute_name}` is a{} {class_like_kind_str} and not a class",
                            if metadata.kind.is_interface() || metadata.kind.is_enum() { "n" } else { "" }
                        )),
                )
                .with_annotation(
                    Annotation::secondary(metadata.name_span.unwrap_or(metadata.span))
                        .with_message(format!(
                            "`{attribute_name}` defined as a{} {class_like_kind_str} here",
                            if metadata.kind.is_interface() || metadata.kind.is_enum() { "n" } else { "" }
                        )),
                )
                .with_note("Only classes can be declared as attributes.")
                .with_note("Interfaces, enums, and traits are not valid attribute types.")
                .with_help(format!("Ensure you are using a class intended to be an attribute. Replace `{attribute_name}` with a valid attribute class.")),
            );

            analyze_attribute_argument_expressions(attribute, context, block_context, artifacts)?;
            continue;
        }

        if metadata.flags.is_abstract() {
            context.collector.report_with_code(
                IssueCode::AbstractClassUsedAsAttribute,
                Issue::error(format!("The abstract class `{attribute_name}` cannot be used as an attribute.",))
                    .with_annotation(Annotation::primary(attribute.name.span()).with_message(format!(
                        "`{attribute_name}` is an abstract class and cannot be instantiated as an attribute"
                    )))
                    .with_annotation(
                        Annotation::secondary(metadata.name_span.unwrap_or(metadata.span))
                            .with_message(format!("`{attribute_name}` defined here as an abstract class")),
                    )
                    .with_note("Attributes must be concrete classes that can be instantiated.")
                    .with_help(format!("Use a concrete class instead of `{attribute_name}` for attributes.")),
            );

            analyze_attribute_argument_expressions(attribute, context, block_context, artifacts)?;
            continue;
        }

        let Some(attribute_flags) = &metadata.attribute_flags else {
            context.collector.report_with_code(
                IssueCode::ClassNotMarkedAsAttribute,
                Issue::error(format!(
                    "Class `{attribute_name}` is used as an attribute but is not declared with `#[Attribute]`.",
                ))
                .with_annotation(
                    Annotation::primary(attribute.name.span()).with_message(format!("`{attribute_name}` used as an attribute here")),
                )
                .with_annotation(
                    Annotation::secondary(metadata.name_span.unwrap_or(metadata.span))
                        .with_message(format!("Class `{attribute_name}` defined here needs an `#[Attribute]` declaration")),
                )
                .with_note("To be used as a PHP attribute, a class must itself be decorated with the `#[\\Attribute]` system attribute.")
                .with_help(format!("Add `#[\\Attribute]` to the definition of class `{attribute_name}` to declare it as an attribute, or use a different class that is a valid attribute.")),
            );

            analyze_attribute_argument_expressions(attribute, context, block_context, artifacts)?;
            continue;
        };

        if let Some(first_usage_span) = used_attributes.get(&attribute_name_bytes)
            && !attribute_flags.is_repeatable()
        {
            context.collector.report_with_code(
                IssueCode::AttributeNotRepeatable,
                Issue::error(format!("Attribute `{attribute_name}` is not declared as repeatable and has already been used."))
                .with_annotation(
                    Annotation::primary(attribute.name.span())
                        .with_message(format!("Duplicate use of non-repeatable attribute `{attribute_name}`")),
                )
                .with_annotation(
                    Annotation::secondary(*first_usage_span)
                        .with_message(format!("Attribute `{attribute_name}` was first used here")),
                )
                .with_note(format!(
                    "The attribute `{attribute_name}` is not declared with `Attribute::IS_REPEATABLE` in its `#[Attribute]` flags. Non-repeatable attributes can only be applied once to a given target (e.g., a class, method, property).",
                ))
                .with_help(format!(
                    "Remove this duplicate `{attribute_name}` attribute, or if multiple instances are intended and valid, modify the attribute class `{attribute_name}` to include `Attribute::IS_REPEATABLE` in its `#[Attribute]` declaration (e.g., `#[Attribute(Attribute::TARGET_ALL | Attribute::IS_REPEATABLE)]`).",
                )),
            );
        }

        used_attributes.insert(attribute_name_bytes, attribute.name.span());

        if let Some(flags) = metadata.attribute_flags {
            let is_valid_target = match target {
                AttributeTarget::ClassLike => flags.targets_class(),
                AttributeTarget::Method => flags.targets_method(),
                AttributeTarget::Property => flags.targets_property(),
                AttributeTarget::Parameter => flags.targets_parameter(),
                AttributeTarget::PromotedProperty => flags.targets_property() || flags.targets_parameter(),
                AttributeTarget::ClassLikeConstant => flags.targets_class_constant(),
                AttributeTarget::Function => flags.targets_function(),
                AttributeTarget::Constant => flags.targets_constant(),
            };

            if !is_valid_target {
                report_invalid_target(context, metadata, attribute, target, flags);
            }
        }

        analyze_attribute_constructor(context, block_context, artifacts, attribute, metadata)?;
    }

    Ok(())
}

fn analyze_attribute_argument_expressions<'ctx, 'arena, A>(
    attribute: &Attribute<'arena>,
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError>
where
    A: Arena,
{
    if let Some(argument_list) = &attribute.argument_list {
        argument_list.analyze(context, block_context, artifacts)?;
    }

    Ok(())
}

fn analyze_attribute_constructor<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    attribute: &Attribute<'arena>,
    metadata: &'ctx ClassLikeMetadata,
) -> Result<(), AnalysisError>
where
    A: Arena,
{
    let constructor_id = MethodIdentifier::new(metadata.original_name, word("__construct"));
    let declaring_constructor_id = context.codebase.get_declaring_method_identifier(&constructor_id);

    artifacts.symbol_references.add_reference_for_method_call(&block_context.scope, &constructor_id);

    let Some(constructor) = context.codebase.get_method_by_id(&declaring_constructor_id) else {
        if let Some(argument_list) = &attribute.argument_list {
            if !argument_list.arguments.is_empty() {
                let attribute_name = metadata.original_name;
                context.collector.report_with_code(
                    IssueCode::TooManyArguments,
                    Issue::error(format!(
                        "Attribute class `{attribute_name}` has no `__construct` method, but arguments were provided."
                    ))
                    .with_annotation(Annotation::primary(argument_list.span()).with_message("Arguments provided here"))
                    .with_annotation(
                        Annotation::secondary(attribute.name.span())
                            .with_message(format!("Attribute class `{attribute_name}` has no constructor")),
                    )
                    .with_help("Remove the arguments, or define a public `__construct` method on the attribute class."),
                );
            }

            argument_list.analyze(context, block_context, artifacts)?;
        }

        return Ok(());
    };

    artifacts.symbol_references.add_reference_for_method_call(&block_context.scope, &declaring_constructor_id);

    let invocation = Invocation {
        target: InvocationTarget::FunctionLike {
            identifier: FunctionLikeIdentifier::Method(
                declaring_constructor_id.get_class_name(),
                declaring_constructor_id.get_method_name(),
            ),
            metadata: constructor,
            inferred_return_type: None,
            method_context: Some(MethodTargetContext {
                declaring_method_id: Some(declaring_constructor_id),
                class_like_metadata: metadata,
                class_type: StaticClassType::None,
                declaring_object_type: None,
            }),
            span: attribute.name.span(),
        },
        arguments_source: match &attribute.argument_list {
            Some(argument_list) => InvocationArgumentsSource::AttributeArgumentList(argument_list),
            None => InvocationArgumentsSource::None(attribute.span()),
        },
        span: attribute.span(),
    };

    let mut template_result = TemplateResult::new(IndexMap::with_hasher(RandomState::default()), HashMap::default());
    let mut argument_types = WordMap::default();
    analyze_invocation(
        context,
        block_context,
        artifacts,
        &invocation,
        Some((metadata.name, None)),
        &mut template_result,
        &mut argument_types,
    )?;

    // Attributes are instantiated by reflection, never from the annotated declaration, so
    // the constructor must be reachable from the global scope.
    check_method_visibility(
        context,
        None,
        declaring_constructor_id.get_class_name().as_bytes(),
        declaring_constructor_id.get_method_name().as_bytes(),
        attribute.span(),
        Some(attribute.name.span()),
    );

    Ok(())
}

fn report_invalid_target<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    metadata: &'ctx ClassLikeMetadata,
    attribute: &Attribute<'arena>,
    target: AttributeTarget,
    flags: AttributeFlags,
) where
    A: Arena,
{
    let attribute_name = metadata.original_name;
    let attribute_name_bytes = attribute_name.as_bytes();
    let short_attribute_name = match memchr::memrchr(b'\\', attribute_name_bytes) {
        Some(i) => &attribute_name_bytes[i + 1..],
        None => attribute_name_bytes,
    };
    let short_attribute_name = BytesDisplay(short_attribute_name);
    let allowed_targets = flags.get_target_names().join(", ");

    context.collector.report_with_code(
        IssueCode::InvalidAttributeTarget,
        Issue::error(format!("Attribute `{attribute_name}` cannot be used on {}.", target.as_str()))
            .with_annotation(Annotation::primary(attribute.name.span()).with_message("This attribute is not allowed here"))
            .with_annotation(
                Annotation::secondary(metadata.name_span.unwrap_or(metadata.span))
                    .with_message(format!("`{attribute_name}` defined here")),
            )
            .with_note(format!(
                "The definition of `{attribute_name}` restricts its use to the following targets: {allowed_targets}."
            ))
            .with_help(format!(
                "Remove the `#[{short_attribute_name}]` attribute from this location, or update the `#[Attribute]` declaration on the `{attribute_name}` class to include `{}` as a valid target.",
                target.as_str()
            ))
    );
}
