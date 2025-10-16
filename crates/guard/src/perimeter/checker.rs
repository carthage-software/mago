use mago_codex::metadata::CodebaseMetadata;
use mago_span::Span;

use crate::context::GuardContext;
use crate::matcher;
use crate::path::NamespacePath;
use crate::path::Path;
use crate::path::SymbolSelector;
use crate::report::breach::BoundaryBreach;
use crate::report::breach::BreachReason;
use crate::report::breach::BreachVector;
use crate::settings::PermittedDependency;
use crate::settings::PermittedDependencyKind;
use crate::settings::Settings;

/// Checks a symbol usage and reports violations.
///
/// # Arguments
///
/// * `ctx` - The guard context
/// * `target_fqn` - The fully qualified name being used
/// * `target_type` - The type of symbol being used
/// * `usage_kind` - The kind of usage
/// * `span` - The span of the usage in the source code
pub fn check_usage(
    ctx: &mut GuardContext<'_, '_>,
    dependency_fqn: &str,
    dependency_kind: PermittedDependencyKind,
    vector: BreachVector,
    span: Span,
) {
    if let Some(reason) = check_allowed(ctx, dependency_fqn, dependency_kind) {
        ctx.boundary_breaches.push(BoundaryBreach {
            source_namespace: ctx.get_current_namespace().to_string(),
            dependency_fqn: dependency_fqn.to_string(),
            dependency_kind,
            vector,
            span,
            reason,
        });
    }
}

/// Checks if a usage is allowed based on the configured rules.
///
/// # Arguments
///
/// * `codebase` - The codebase metadata for symbol lookups
/// * `settings` - The guard settings
/// * `source_namespace` - The namespace where the usage occurs
/// * `target_fqn` - The fully qualified name being used
/// * `target_type` - The type of the symbol being used (class, interface, etc.)
///
/// # Returns
///
/// `Some(ArchitecturalViolationReason)` if the usage is not allowed, `None` if it is allowed
/// or if layering rules apply and permit the usage.
fn check_allowed(
    ctx: &GuardContext<'_, '_>,
    target_fqn: &str,
    dependency_kind: PermittedDependencyKind,
) -> Option<BreachReason> {
    let rules: Vec<_> = ctx
        .settings
        .perimeter
        .rules
        .iter()
        .filter(|rule| match &rule.namespace {
            NamespacePath::Global => ctx.get_current_namespace().is_empty(),
            NamespacePath::Specific(rule_namespace) => {
                matcher::matches(ctx.get_current_namespace(), rule_namespace, false, true)
            }
        })
        .collect();

    if !rules.is_empty() {
        for rule in &rules {
            for allowed in &rule.permit {
                match allowed {
                    PermittedDependency::Dependency(path) => {
                        if is_path_allowed(ctx.codebase, ctx.settings, path, ctx.get_current_namespace(), target_fqn) {
                            return None;
                        }
                    }
                    PermittedDependency::DependencyOfKind { path, kinds } => {
                        if kinds.contains(&dependency_kind)
                            && is_path_allowed(
                                ctx.codebase,
                                ctx.settings,
                                path,
                                ctx.get_current_namespace(),
                                target_fqn,
                            )
                        {
                            return None;
                        }
                    }
                }
            }
        }
    }

    if !ctx.settings.perimeter.layering.is_empty() {
        let source_layer_index = get_layer_index(ctx.get_current_namespace(), ctx.settings);
        let target_layer_index = get_layer_index(target_fqn, ctx.settings);

        if let (Some(src_idx), Some(tgt_idx)) = (source_layer_index, target_layer_index) {
            if src_idx >= tgt_idx {
                return None;
            } else {
                return Some(BreachReason::Layering {
                    source_layer: ctx.settings.perimeter.layering[src_idx].clone(),
                    target_layer: ctx.settings.perimeter.layering[tgt_idx].clone(),
                });
            }
        }
    }

    if rules.is_empty() {
        Some(BreachReason::NoMatchingRule)
    } else {
        Some(BreachReason::ForbiddenByRule { rule_namespaces: rules.iter().map(|r| r.namespace.clone()).collect() })
    }
}

/// Extracts the root namespace from a fully qualified name.
///
/// # Arguments
///
/// * `fqn` - The fully qualified name
///
/// # Returns
///
/// The root namespace, or the full name if there's no namespace separator
fn get_root_namespace(fqn: &str) -> &str {
    if let Some(pos) = fqn.find('\\') { &fqn[..pos] } else { fqn }
}

/// Checks if a fully qualified name is considered native/builtin.
///
/// A symbol is native if:
/// - It's a class-like with the BUILTIN metadata flag set
/// - It's a function with the BUILTIN metadata flag set
/// - It's a constant with the BUILTIN metadata flag set
///
/// # Arguments
///
/// * `codebase` - The codebase metadata to look up the symbol
/// * `fqn` - The fully qualified name to check
///
/// # Returns
///
/// `true` if the name is considered native/builtin, `false` otherwise
fn is_native(codebase: &CodebaseMetadata, fqn: &str) -> bool {
    codebase
        .get_class_like(fqn)
        .map(|c| &c.flags)
        .or_else(|| codebase.get_function(fqn).map(|f| &f.flags))
        .or_else(|| codebase.get_constant(fqn).map(|c| &c.flags))
        .is_some_and(|flags| flags.is_built_in())
}

fn get_layer_index(namespace: &str, settings: &Settings) -> Option<usize> {
    for (i, layer_namespace) in settings.perimeter.layering.iter().enumerate() {
        match layer_namespace {
            NamespacePath::Global if namespace.is_empty() => {
                return Some(i);
            }
            NamespacePath::Specific(ns) if matcher::matches(namespace, ns, false, true) => {
                return Some(i);
            }
            _ => {}
        }
    }

    None
}

/// Checks if a target FQN is allowed based on a specific path configuration.
///
/// # Arguments
///
/// * `codebase` - The codebase metadata for symbol lookups
/// * `settings` - The guard settings
/// * `path` - The path configuration
/// * `source_namespace` - The namespace where the usage occurs
/// * `target_fqn` - The fully qualified name being used
/// * `_target_type` - The type of the symbol being used
///
/// # Returns
///
/// `true` if the path allows the target FQN, `false` otherwise
fn is_path_allowed(
    codebase: &CodebaseMetadata,
    settings: &Settings,
    path: &Path,
    source_namespace: &str,
    target_fqn: &str,
) -> bool {
    match path {
        Path::All => true,
        Path::Native => is_native(codebase, target_fqn),
        Path::Self_ => {
            matcher::matches(target_fqn, source_namespace, false, false)
                || get_root_namespace(source_namespace).eq_ignore_ascii_case(get_root_namespace(target_fqn))
        }
        Path::Layer(layer_name) => settings.perimeter.layers.get(layer_name).is_some_and(|layer_patterns| {
            layer_patterns
                .iter()
                .any(|pattern| is_path_allowed(codebase, settings, pattern, source_namespace, target_fqn))
        }),
        Path::Selector(selector) => match selector {
            SymbolSelector::Namespace(ns) => match ns {
                NamespacePath::Global => !target_fqn.contains('\\'),
                NamespacePath::Specific(pattern) => matcher::matches(target_fqn, pattern, false, true),
            },
            SymbolSelector::Symbol(sn) => target_fqn.eq_ignore_ascii_case(sn),
            SymbolSelector::Pattern(p) => matcher::matches(target_fqn, p, false, false),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_root_namespace() {
        assert_eq!(get_root_namespace("Foo\\Bar\\Baz"), "Foo");
        assert_eq!(get_root_namespace("Foo\\Bar"), "Foo");
        assert_eq!(get_root_namespace("Foo"), "Foo");
    }
}
