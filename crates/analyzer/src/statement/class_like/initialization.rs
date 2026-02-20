use std::collections::HashSet;

use itertools::Itertools;
use mago_atom::AtomSet;

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;

/// Check property initialization for a class-like.
pub fn check_property_initialization<'ctx>(
    context: &mut Context<'ctx, '_>,
    artifacts: &AnalysisArtifacts,
    class_like_metadata: &'ctx ClassLikeMetadata,
    declaration_span: Span,
    name_span: Option<Span>,
) {
    if !context.settings.check_property_initialization {
        return;
    }

    if class_like_metadata.flags.is_abstract()
        || class_like_metadata.kind.is_interface()
        || class_like_metadata.kind.is_trait()
        || class_like_metadata.kind.is_enum()
    {
        return;
    }

    let uninitialized_properties: Vec<_> = class_like_metadata
        .declaring_property_ids
        .iter()
        .sorted_by_key(|(k, _)| *k)
        .filter_map(|(&name, declaring_class)| {
            let declaring_meta = context.codebase.get_class_like(declaring_class)?;
            let prop = declaring_meta.properties.get(&name)?;
            if property_requires_initialization(name, prop, class_like_metadata, declaring_meta, context) {
                Some((name, prop))
            } else {
                None
            }
        })
        .collect();

    if uninitialized_properties.is_empty() {
        return;
    }

    let constructor_method_id = class_like_metadata.declaring_method_ids.get(&atom("__construct"));
    let Some(constructor_method_id) = constructor_method_id else {
        let parent_initializes =
            check_parent_constructor_initializes(context, artifacts, class_like_metadata, &uninitialized_properties);

        if parent_initializes {
            return;
        }

        let initialized_by_initializers =
            compute_class_initializer_initializations(artifacts, context, class_like_metadata);

        let still_uninitialized: Vec<_> = uninitialized_properties
            .iter()
            .filter(|(prop_name, _)| !initialized_by_initializers.contains(prop_name))
            .copied()
            .collect();

        if !still_uninitialized.is_empty() {
            report_missing_constructor(context, class_like_metadata, declaration_span, name_span, &still_uninitialized);
        }

        return;
    };

    let constructor_declaring_class = constructor_method_id.get_class_name();
    if constructor_declaring_class != class_like_metadata.name {
        let own_uninitialized: Vec<_> = uninitialized_properties
            .iter()
            .filter(|(prop_name, _)| {
                class_like_metadata
                    .declaring_property_ids
                    .get(prop_name)
                    .is_some_and(|declaring_class| *declaring_class == class_like_metadata.name)
            })
            .copied()
            .collect();

        let inherited_uninitialized: Vec<_> = uninitialized_properties
            .iter()
            .filter(|(prop_name, _)| {
                class_like_metadata
                    .declaring_property_ids
                    .get(prop_name)
                    .is_some_and(|declaring_class| *declaring_class != class_like_metadata.name)
            })
            .copied()
            .collect();

        let initialized_by_initializers =
            compute_class_initializer_initializations(artifacts, context, class_like_metadata);

        let initialized_by_inherited_constructor = compute_transitive_initializations(
            artifacts,
            context,
            constructor_declaring_class,
            atom("__construct"),
            class_like_metadata.flags.is_final(),
            false,
        );

        // Check if the constructor comes from a trait (for cross-file trust logic)
        // Due to per-file artifacts, we can't verify trait constructor behavior across files,
        // so we trust that a trait's constructor initializes properties declared by the same trait
        let constructor_is_from_trait =
            context.codebase.get_class_like(&constructor_declaring_class).is_some_and(|m| m.kind.is_trait());

        // For inherited properties, check if the parent constructor, trait constructor, or class initializers initialize them
        if !inherited_uninitialized.is_empty() {
            let parent_initializes =
                check_parent_constructor_initializes(context, artifacts, class_like_metadata, &inherited_uninitialized);

            // Filter out properties that are initialized by class initializers or the inherited constructor
            let still_uninitialized_inherited: Vec<_> = inherited_uninitialized
                .iter()
                .filter(|(prop_name, _)| {
                    // Already known to be initialized by class initializers or inherited constructor?
                    if initialized_by_initializers.contains(prop_name)
                        || initialized_by_inherited_constructor.contains(prop_name)
                    {
                        return false;
                    }

                    // If constructor is from a trait, and this property is from ANY trait,
                    // trust that the trait constructor handles trait property initialization.
                    // This covers cases like: Trait A declares $foo, Trait B uses A and has __construct
                    // that initializes $foo, Class C uses B. We can't verify cross-file trait behavior.
                    if constructor_is_from_trait
                        && let Some(prop_declaring_class) =
                            class_like_metadata.declaring_property_ids.get(prop_name).copied()
                    {
                        let prop_is_from_trait =
                            context.codebase.get_class_like(&prop_declaring_class).is_some_and(|m| m.kind.is_trait());
                        if prop_is_from_trait {
                            return false; // Trait constructor + trait property - trust it
                        }
                    }

                    true // Still uninitialized
                })
                .copied()
                .collect();

            if !parent_initializes && !still_uninitialized_inherited.is_empty() {
                // Neither parent constructor, trait constructor, nor class initializers initialize inherited properties - report error
                report_missing_constructor(
                    context,
                    class_like_metadata,
                    declaration_span,
                    name_span,
                    &still_uninitialized_inherited,
                );
            }
        }

        // For own properties with inherited constructor, check if class initializers initialize them
        // Report each one that is NOT initialized by class initializers
        for (prop_name, prop_metadata) in &own_uninitialized {
            if !initialized_by_initializers.contains(prop_name) {
                // Get the declaring class for this property
                let declaring_class = class_like_metadata.declaring_property_ids.get(prop_name).copied();
                report_uninitialized_property(
                    context,
                    class_like_metadata,
                    *prop_name,
                    prop_metadata.name_span,
                    name_span.unwrap_or(declaration_span),
                    declaring_class,
                );
            }
        }

        return;
    }

    // Get properties initialized by the class's own constructor (including transitive through method calls)
    // Don't trust all methods for constructor - use standard trustworthiness rules
    let mut definitely_initialized = compute_transitive_initializations(
        artifacts,
        context,
        constructor_declaring_class,
        atom("__construct"),
        class_like_metadata.flags.is_final(),
        false,
    );

    // Also include properties initialized by class initializers
    let initialized_by_initializers =
        compute_class_initializer_initializations(artifacts, context, class_like_metadata);
    definitely_initialized.extend(initialized_by_initializers);

    // Report uninitialized properties
    for (prop_name, prop_metadata) in &uninitialized_properties {
        // prop_name from declaring_property_ids already includes $ prefix
        if !definitely_initialized.contains(prop_name) {
            // Get the declaring class for this property
            let declaring_class = class_like_metadata.declaring_property_ids.get(prop_name).copied();
            report_uninitialized_property(
                context,
                class_like_metadata,
                *prop_name,
                prop_metadata.name_span,
                name_span.unwrap_or(declaration_span),
                declaring_class,
            );
        }
    }
}

/// Determines if a property requires initialization in the constructor.
fn property_requires_initialization(
    _name: Atom,
    property: &PropertyMetadata,
    class_like_metadata: &ClassLikeMetadata,
    declaring_class_metadata: &ClassLikeMetadata,
    context: &Context<'_, '_>,
) -> bool {
    // Has default value - doesn't need initialization
    if property.flags.has_default() {
        return false;
    }

    // Promoted property - initialized via constructor parameter
    if property.flags.is_promoted_property() {
        return false;
    }

    // Static property - not initialized in constructor
    if property.flags.is_static() {
        return false;
    }

    // Virtual/hooked property - may not need backing storage
    if property.flags.is_virtual_property() {
        return false;
    }

    // No type declaration - PHP doesn't require initialization
    if property.type_declaration_metadata.is_none() {
        return false;
    }

    // Check if any plugin considers this property initialized
    if context.plugin_registry.is_property_initialized(declaring_class_metadata, property) {
        return false;
    }

    // NOTE: Nullable types (`?string`, `null|string`) and mixed types STILL require
    // initialization. In PHP, accessing a typed property before initialization causes
    // an error, even if the type is nullable. They don't implicitly become `null`.

    // If declared in a different class (inherited)
    if declaring_class_metadata.name != class_like_metadata.name {
        // If declaring class is a concrete class (not abstract and not a trait), it handles initialization.
        if !declaring_class_metadata.flags.is_abstract() && !declaring_class_metadata.kind.is_trait() {
            return false;
        }

        // If abstract parent has a constructor and the child class does NOT define its own constructor,
        // trust the parent's constructor initializes its own properties.
        // The child will inherit the parent's constructor.
        //
        // However, if the child defines its own constructor, we cannot blindly trust the parent's
        // constructor because the child's constructor overrides it and may not call parent::__construct().
        if declaring_class_metadata.flags.is_abstract()
            && declaring_class_metadata.declaring_method_ids.contains_key(&atom("__construct"))
            && !class_like_metadata.declaring_method_ids.contains_key(&atom("__construct"))
        {
            return false;
        }
    }

    true
}

/// Compute properties initialized transitively through method calls.
///
/// The `trust_all_methods` parameter allows skipping the trustworthiness check for called methods.
/// This is used for class initializers where we want to trust the entire call chain, since
/// the framework guarantees the initializer method is called and will execute all its calls.
///
/// This function uses an iterative approach instead of recursion to avoid stack overflow
/// in release builds when analyzing deep class hierarchies (common in frameworks like Symfony).
fn compute_transitive_initializations(
    artifacts: &AnalysisArtifacts,
    context: &Context<'_, '_>,
    class_name: Atom,
    method_name: Atom,
    class_is_final: bool,
    trust_all_methods: bool,
) -> AtomSet {
    let mut all_initialized = AtomSet::default();

    let mut work_queue: Vec<(Atom, Atom, bool, bool, Atom)> =
        vec![(class_name, method_name, class_is_final, trust_all_methods, class_name)];

    let mut visited: HashSet<(Atom, Atom)> = HashSet::default();

    while let Some((current_class, current_method, current_is_final, current_trust_all, origin_class)) =
        work_queue.pop()
    {
        if !visited.insert((current_class, current_method)) {
            continue;
        }

        let mut methods_to_process = vec![current_method];
        let mut visited_methods: HashSet<Atom> = HashSet::default();

        while let Some(method) = methods_to_process.pop() {
            if !visited_methods.insert(method) {
                continue;
            }

            let method_key = (current_class, method);

            if let Some(props) = artifacts.method_initialized_properties.get(&method_key) {
                if current_class == origin_class {
                    all_initialized.extend(props.iter().copied());
                } else if let Some(origin_meta) = context.codebase.get_class_like(&origin_class) {
                    for prop_name in props {
                        let is_inherited = origin_meta
                            .declaring_property_ids
                            .get(prop_name)
                            .is_none_or(|declaring_class| *declaring_class != origin_class);
                        if is_inherited {
                            all_initialized.insert(*prop_name);
                        }
                    }
                }
            }

            if let Some(called_methods) = artifacts.method_calls_this_methods.get(&method_key) {
                for called_method in called_methods {
                    if visited_methods.contains(called_method) {
                        continue;
                    }

                    if current_trust_all
                        || is_method_trustworthy(context, current_class, *called_method, current_is_final)
                    {
                        methods_to_process.push(*called_method);
                    }
                }
            }
        }

        let method_key = (current_class, current_method);
        if artifacts.method_calls_parent_constructor.get(&method_key) == Some(&true)
            && let Some(class_meta) = context.codebase.get_class_like(&current_class)
            && let Some(parent_name) = &class_meta.direct_parent_class
            && let Some(parent_meta) = context.codebase.get_class_like(parent_name)
            && parent_meta.declaring_method_ids.contains_key(&atom("__construct"))
        {
            work_queue.push((*parent_name, atom("__construct"), parent_meta.flags.is_final(), false, origin_class));
        }

        if let Some(parent_initializer_name) = artifacts.method_calls_parent_initializer.get(&method_key)
            && let Some(class_meta) = context.codebase.get_class_like(&current_class)
            && let Some(parent_name) = &class_meta.direct_parent_class
            && let Some(parent_meta) = context.codebase.get_class_like(parent_name)
            && let Some(method_id) = parent_meta.declaring_method_ids.get(parent_initializer_name)
        {
            let declaring_class_name = method_id.get_class_name();
            work_queue.push((
                declaring_class_name,
                *parent_initializer_name,
                parent_meta.flags.is_final(),
                true,
                origin_class,
            ));
        }
    }

    all_initialized
}

/// Compute properties initialized by any class initializer method.
///
/// This function checks all methods listed in `class_initializers` setting
/// and returns the union of properties they initialize. It also walks up
/// the inheritance chain to check parent classes for class initializers.
fn compute_class_initializer_initializations(
    artifacts: &AnalysisArtifacts,
    context: &Context<'_, '_>,
    class_like_metadata: &ClassLikeMetadata,
) -> AtomSet {
    let mut all_initialized = AtomSet::default();

    // No class initializers configured
    if context.settings.class_initializers.is_empty() {
        return all_initialized;
    }

    let class_name = class_like_metadata.name;
    let class_is_final = class_like_metadata.flags.is_final();

    for initializer_name in &context.settings.class_initializers {
        if let Some(method_id) = class_like_metadata.declaring_method_ids.get(initializer_name) {
            let declaring_class_name = method_id.get_class_name();

            let initialized = compute_transitive_initializations(
                artifacts,
                context,
                declaring_class_name,
                *initializer_name,
                class_is_final,
                true,
            );

            all_initialized.extend(initialized);
        }
    }

    let mut current_class = class_like_metadata.direct_parent_class.as_ref();
    while let Some(parent_name) = current_class {
        let Some(parent_meta) = context.codebase.get_class_like(parent_name) else {
            break;
        };

        for initializer_name in &context.settings.class_initializers {
            if let Some(method_id) = parent_meta.declaring_method_ids.get(initializer_name) {
                let declaring_class_name = method_id.get_class_name();

                let initialized = compute_transitive_initializations(
                    artifacts,
                    context,
                    declaring_class_name,
                    *initializer_name,
                    parent_meta.flags.is_final(),
                    true,
                );

                for prop_name in initialized {
                    let is_inherited = class_like_metadata
                        .declaring_property_ids
                        .get(&prop_name)
                        .is_none_or(|declaring_class| *declaring_class != class_name);

                    if is_inherited {
                        all_initialized.insert(prop_name);
                    }
                }
            }
        }

        current_class = parent_meta.direct_parent_class.as_ref();
    }

    let has_any_initializer = {
        let current_has = context
            .settings
            .class_initializers
            .iter()
            .any(|init_name| class_like_metadata.declaring_method_ids.contains_key(init_name));

        if current_has {
            true
        } else {
            let mut found = false;
            let mut check_class = class_like_metadata.direct_parent_class.as_ref();
            while let Some(parent_name) = check_class {
                let Some(parent_meta) = context.codebase.get_class_like(parent_name) else {
                    break;
                };
                if context
                    .settings
                    .class_initializers
                    .iter()
                    .any(|init_name| parent_meta.declaring_method_ids.contains_key(init_name))
                {
                    found = true;
                    break;
                }
                check_class = parent_meta.direct_parent_class.as_ref();
            }
            found
        }
    };

    if has_any_initializer {
        let mut current_class = class_like_metadata.direct_parent_class.as_ref();
        while let Some(parent_name) = current_class {
            let Some(parent_meta) = context.codebase.get_class_like(parent_name) else {
                break;
            };

            let parent_has_initializer = context
                .settings
                .class_initializers
                .iter()
                .any(|init_name| parent_meta.declaring_method_ids.contains_key(init_name));

            if parent_has_initializer {
                for (prop_name, declaring_class) in &parent_meta.declaring_property_ids {
                    if *declaring_class == *parent_name {
                        all_initialized.insert(*prop_name);
                    }
                }
            }

            current_class = parent_meta.direct_parent_class.as_ref();
        }
    }

    all_initialized
}

/// A method is trustworthy if it cannot be overridden by subclasses,
/// or if it's explicitly listed as a class initializer.
fn is_method_trustworthy(context: &Context<'_, '_>, class_name: Atom, method_name: Atom, class_is_final: bool) -> bool {
    if class_is_final {
        return true;
    }

    if context.settings.class_initializers.contains(&method_name) {
        return true;
    }

    if let Some(visibility) = context.codebase.get_method_visibility(&class_name, &method_name)
        && visibility.is_private()
    {
        return true;
    }

    if context.codebase.method_is_final(&class_name, &method_name) {
        return true;
    }

    false
}

/// Check if parent class constructor initializes the required properties.
/// This walks up the inheritance chain to find a constructor.
///
/// For multi-file analysis where artifacts may not be available for parent classes,
/// we trust that a parent class's constructor initializes properties declared in that
/// same parent class. This is a reasonable assumption - if a class has both typed
/// properties and a constructor, the constructor should initialize those properties.
fn check_parent_constructor_initializes(
    context: &Context<'_, '_>,
    artifacts: &AnalysisArtifacts,
    class_like_metadata: &ClassLikeMetadata,
    uninitialized_properties: &[(Atom, &PropertyMetadata)],
) -> bool {
    let mut current_class = class_like_metadata.direct_parent_class.as_ref();

    while let Some(parent_name) = current_class {
        let Some(parent_meta) = context.codebase.get_class_like(parent_name) else {
            return false;
        };

        if parent_meta.declaring_method_ids.contains_key(&atom("__construct")) {
            let method_key = (*parent_name, atom("__construct"));
            let constructor_initialized = artifacts.method_initialized_properties.get(&method_key);

            let all_initialized = uninitialized_properties.iter().all(|(prop_name, _)| {
                if parent_meta.initialized_properties.contains(prop_name) {
                    return true;
                }

                if let Some(init_props) = constructor_initialized
                    && init_props.contains(prop_name)
                {
                    return true;
                }

                if constructor_initialized.is_none() {
                    let prop_declaring_class = class_like_metadata.declaring_property_ids.get(prop_name).copied();
                    if prop_declaring_class == Some(*parent_name) {
                        return true;
                    }
                }

                false
            });

            if all_initialized {
                return true;
            }

            return false;
        }

        current_class = parent_meta.direct_parent_class.as_ref();
    }

    false
}

fn report_missing_constructor(
    context: &mut Context<'_, '_>,
    class_like_metadata: &ClassLikeMetadata,
    declaration_span: Span,
    name_span: Option<Span>,
    uninitialized_properties: &[(Atom, &PropertyMetadata)],
) {
    let class_name = &class_like_metadata.original_name;
    let prop_names: Vec<_> = uninitialized_properties.iter().map(|(name, _)| name.to_string()).collect();
    let prop_list = prop_names.join(", ");

    let mut issue = Issue::error(format!(
        "Class `{class_name}` has typed properties without default values but no constructor to initialize them."
    ))
    .with_annotation(
        Annotation::primary(name_span.unwrap_or(declaration_span)).with_message("This class needs a constructor"),
    );

    for (_, prop_meta) in uninitialized_properties {
        if let Some(span) = prop_meta.name_span {
            issue =
                issue.with_annotation(Annotation::secondary(span).with_message("This property needs initialization"));
        }
    }

    issue = issue
        .with_note(format!("Properties requiring initialization: {prop_list}"))
        .with_help("Add a constructor that initializes all typed properties, or provide default values.");

    context.collector.report_with_code(IssueCode::MissingConstructor, issue);
}

fn report_uninitialized_property(
    context: &mut Context<'_, '_>,
    class_like_metadata: &ClassLikeMetadata,
    prop_name: Atom,
    prop_span: Option<Span>,
    class_span: Span,
    declaring_class: Option<Atom>,
) {
    let class_name = &class_like_metadata.original_name;

    let mut issue =
        Issue::error(format!("Property `{prop_name}` is not initialized in the constructor of class `{class_name}`."));

    let is_inherited = declaring_class.is_some_and(|decl| decl != class_like_metadata.name);

    if let Some(span) = prop_span {
        let message = if is_inherited
            && let Some(decl_class) = declaring_class
            && let Some(decl_meta) = context.codebase.get_class_like(&decl_class)
        {
            format!("Property declared in `{}`", decl_meta.original_name)
        } else {
            "This property is not initialized".to_string()
        };
        issue = issue.with_annotation(Annotation::primary(span).with_message(message));
    }

    issue = issue.with_annotation(Annotation::secondary(class_span).with_message(format!("In class `{class_name}`")));

    issue = issue.with_note("Typed properties without default values must be initialized in the constructor.");

    let help = if is_inherited {
        format!(
            "Initialize `{prop_name}` in the constructor, call `parent::__construct()` if parent handles initialization, or provide a default value."
        )
    } else {
        format!("Initialize `{prop_name}` in the constructor, provide a default value, or make the type nullable.")
    };
    issue = issue.with_help(help);

    context.collector.report_with_code(IssueCode::UninitializedProperty, issue);
}
