use mago_atom::Atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_codex::visibility::Visibility;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureCompatibilityIssue {
    FinalMethodOverride,
    StaticModifierMismatch { child_is_static: bool, parent_is_static: bool },
    VisibilityNarrowed { child_visibility: Visibility, parent_visibility: Visibility },
    ParameterCountMismatch { child_required_count: usize, parent_required_count: usize },
    IncompatibleParameterType { parameter_index: usize, child_type: Atom, parent_type: Atom },
    IncompatibleReturnType { child_type: Atom, parent_type: Atom },
    ParameterNameMismatch { parameter_index: usize, child_name: Atom, parent_name: Atom },
}

/// Validates that a child method signature is compatible with a parent method signature.
///
/// This function checks the Liskov Substitution Principle (LSP) rules:
/// - Static modifier must match exactly (invariant)
/// - Visibility can only widen (public >= protected >= private)
/// - Parameters are contravariant (child must accept >= parent accepts)
/// - Return type is covariant (child must return <= parent returns)
/// - Parameter count (child must accept at least parent's required parameters)
/// - Parameter names should match (warning only - breaks named arguments)
///
/// # Arguments
///
/// * `codebase` - The codebase metadata for type lookups
/// * `child_class_name` - The fully qualified name of the class containing the overriding method
/// * `child_method` - The overriding method
/// * `parent_method` - The parent/interface method being overridden/implemented
///
/// # Returns
///
/// A vector of issues found. Empty vector if signatures are fully compatible.
/// Errors are returned first, then warnings.
pub fn validate_method_signature_compatibility(
    codebase: &CodebaseMetadata,
    child_class_name: Atom,
    child_method: &FunctionLikeMetadata,
    parent_method: &FunctionLikeMetadata,
) -> Vec<SignatureCompatibilityIssue> {
    if !child_method.flags.is_user_defined() {
        // The child method is not user-defined; skip validation.
        return Vec::new();
    }

    let mut issues = Vec::new();

    let Some(child_method_meta) = child_method.method_metadata.as_ref() else {
        return issues;
    };

    let Some(parent_method_meta) = parent_method.method_metadata.as_ref() else {
        return issues;
    };

    if parent_method_meta.is_final {
        issues.push(SignatureCompatibilityIssue::FinalMethodOverride);

        return issues;
    }

    if child_method_meta.is_static != parent_method_meta.is_static {
        issues.push(SignatureCompatibilityIssue::StaticModifierMismatch {
            child_is_static: child_method_meta.is_static,
            parent_is_static: parent_method_meta.is_static,
        });

        return issues;
    }

    if is_visibility_narrowed(child_method_meta.visibility, parent_method_meta.visibility) {
        issues.push(SignatureCompatibilityIssue::VisibilityNarrowed {
            child_visibility: child_method_meta.visibility,
            parent_visibility: parent_method_meta.visibility,
        });

        return issues;
    }

    let parent_param_count = parent_method.parameters.len();
    let child_param_count = child_method.parameters.len();

    if child_param_count < parent_param_count {
        let child_required_count = child_method.parameters.iter().filter(|p| !p.flags.has_default()).count();
        let parent_required_count = parent_method.parameters.iter().filter(|p| !p.flags.has_default()).count();

        issues
            .push(SignatureCompatibilityIssue::ParameterCountMismatch { child_required_count, parent_required_count });

        return issues;
    }

    for (index, parent_param) in parent_method.parameters.iter().enumerate() {
        let child_param = &child_method.parameters[index];

        let parent_is_optional = parent_param.flags.has_default();
        let child_is_optional = child_param.flags.has_default();

        if parent_is_optional && !child_is_optional {
            let child_required_count = child_method.parameters.iter().filter(|p| !p.flags.has_default()).count();
            let parent_required_count = parent_method.parameters.iter().filter(|p| !p.flags.has_default()).count();

            issues.push(SignatureCompatibilityIssue::ParameterCountMismatch {
                child_required_count,
                parent_required_count,
            });

            return issues;
        }
    }

    for index in parent_param_count..child_param_count {
        let child_param = &child_method.parameters[index];
        if !child_param.flags.has_default() {
            let child_required_count = child_method.parameters.iter().filter(|p| !p.flags.has_default()).count();
            let parent_required_count = parent_method.parameters.iter().filter(|p| !p.flags.has_default()).count();

            issues.push(SignatureCompatibilityIssue::ParameterCountMismatch {
                child_required_count,
                parent_required_count,
            });

            return issues;
        }
    }

    for (index, parent_param) in parent_method.parameters.iter().enumerate() {
        let Some(child_param) = child_method.parameters.get(index) else {
            continue;
        };

        let parent_param_type = match &parent_param.type_metadata {
            Some(t) => &t.type_union,
            None => continue,
        };

        let child_param_type = match &child_param.type_metadata {
            Some(t) => &t.type_union,
            None => continue,
        };

        let mut expanded_parent_param_type = parent_param_type.clone();
        let mut expanded_child_param_type = child_param_type.clone();

        let expansion_options = TypeExpansionOptions {
            self_class: Some(child_class_name),
            static_class_type: StaticClassType::Name(child_class_name),
            function_is_final: child_method_meta.is_final,
            ..Default::default()
        };

        if expanded_parent_param_type.is_expandable() {
            expand_union(codebase, &mut expanded_parent_param_type, &expansion_options);
        }
        if expanded_child_param_type.is_expandable() {
            expand_union(codebase, &mut expanded_child_param_type, &expansion_options);
        }

        let is_compatible = union_comparator::is_contained_by(
            codebase,
            &expanded_parent_param_type,
            &expanded_child_param_type,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        );

        if !is_compatible {
            issues.push(SignatureCompatibilityIssue::IncompatibleParameterType {
                parameter_index: index,
                child_type: expanded_child_param_type.get_id(),
                parent_type: expanded_parent_param_type.get_id(),
            });

            return issues;
        }
    }

    for (index, parent_param) in parent_method.parameters.iter().enumerate() {
        let Some(child_param) = child_method.parameters.get(index) else {
            continue;
        };

        if parent_param.name != child_param.name {
            issues.push(SignatureCompatibilityIssue::ParameterNameMismatch {
                parameter_index: index,
                child_name: child_param.name.0,
                parent_name: parent_param.name.0,
            });
        }
    }

    if let (Some(parent_return), Some(child_return)) =
        (&parent_method.return_type_declaration_metadata, &child_method.return_type_declaration_metadata)
    {
        let parent_return_type = &parent_return.type_union;
        let child_return_type = &child_return.type_union;

        let mut expanded_parent_return_type = parent_return_type.clone();
        let mut expanded_child_return_type = child_return_type.clone();

        let expansion_options = TypeExpansionOptions {
            self_class: Some(child_class_name),
            static_class_type: StaticClassType::Name(child_class_name),
            function_is_final: child_method_meta.is_final,
            ..Default::default()
        };

        if expanded_parent_return_type.is_expandable() {
            expand_union(codebase, &mut expanded_parent_return_type, &expansion_options);
        }
        if expanded_child_return_type.is_expandable() {
            expand_union(codebase, &mut expanded_child_return_type, &expansion_options);
        }

        let mut comparison_result = ComparisonResult::new();
        let is_compatible = union_comparator::is_contained_by(
            codebase,
            &expanded_child_return_type,
            &expanded_parent_return_type,
            false,
            false,
            false,
            &mut comparison_result,
        );

        if !is_compatible {
            issues.push(SignatureCompatibilityIssue::IncompatibleReturnType {
                child_type: expanded_child_return_type.get_id(),
                parent_type: expanded_parent_return_type.get_id(),
            });
            return issues;
        }
    }

    issues
}

const fn is_visibility_narrowed(child_visibility: Visibility, parent_visibility: Visibility) -> bool {
    matches!(
        (parent_visibility, child_visibility),
        (Visibility::Public, Visibility::Protected | Visibility::Private)
            | (Visibility::Protected, Visibility::Private)
    )
}
