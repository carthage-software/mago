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
use crate::settings::DependencyRestriction;
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
    dependency_fqn: &[u8],
    dependency_kind: PermittedDependencyKind,
    vector: BreachVector,
    span: Span,
) {
    if let Some(reason) = check_allowed(ctx, dependency_fqn, dependency_kind) {
        ctx.boundary_breaches.push(BoundaryBreach {
            source_namespace: ctx.get_current_namespace().to_vec(),
            dependency_fqn: dependency_fqn.to_vec(),
            dependency_kind,
            vector,
            span,
            reason,
        });
    }
}

/// Checks if a usage is allowed based on the configured rules.
fn check_allowed(
    ctx: &GuardContext<'_, '_>,
    target_fqn: &[u8],
    dependency_kind: PermittedDependencyKind,
) -> Option<BreachReason> {
    if let Some(restriction) =
        ctx.settings.perimeter.restrictions.iter().find(|restriction| {
            violates_restriction(restriction, ctx.get_current_namespace(), target_fqn, dependency_kind)
        })
    {
        return Some(BreachReason::ForbiddenByRestriction { dependency: restriction.dependency.clone() });
    }

    let rule = ctx
        .settings
        .perimeter
        .rules
        .iter()
        .filter_map(|rule| {
            let specificity = match &rule.namespace {
                NamespacePath::Global if ctx.get_current_namespace().is_empty() => 0,
                NamespacePath::Specific(rule_namespace)
                    if matcher::matches(ctx.get_current_namespace(), rule_namespace.as_bytes(), false, true) =>
                {
                    rule_namespace.len()
                }
                _ => return None,
            };

            Some((rule, specificity))
        })
        .max_by_key(|(_, specificity)| *specificity)
        .map(|(rule, _)| rule);

    if let Some(rule) = rule {
        for allowed in &rule.permit {
            match allowed {
                PermittedDependency::Dependency(path) => {
                    if is_path_allowed(ctx.codebase, ctx.settings, path, &rule.namespace, target_fqn) {
                        return None;
                    }
                }
                PermittedDependency::DependencyOfKind { path, kinds } => {
                    if kinds.contains(&dependency_kind)
                        && is_path_allowed(ctx.codebase, ctx.settings, path, &rule.namespace, target_fqn)
                    {
                        return None;
                    }
                }
            }
        }
    }

    let rules: Vec<_> = rule.into_iter().collect();
    if !ctx.settings.perimeter.layering.is_empty() {
        let source_layer_index = get_layer_index(ctx.get_current_namespace(), ctx.settings);
        let target_layer_index = get_layer_index(target_fqn, ctx.settings);

        if let (Some(src_idx), Some(tgt_idx)) = (source_layer_index, target_layer_index) {
            if src_idx >= tgt_idx {
                return None;
            }
            return Some(BreachReason::Layering {
                source_layer: ctx.settings.perimeter.layering[src_idx].clone(),
                target_layer: ctx.settings.perimeter.layering[tgt_idx].clone(),
            });
        }
    }

    if rules.is_empty() && ctx.settings.perimeter.rules.is_empty() && ctx.settings.perimeter.layering.is_empty() {
        None
    } else if rules.is_empty() {
        Some(BreachReason::NoMatchingRule)
    } else {
        Some(BreachReason::ForbiddenByRule { rule_namespaces: rules.iter().map(|r| r.namespace.clone()).collect() })
    }
}

fn matches_source_namespace(namespace: &[u8], pattern: &str) -> bool {
    if pattern.eq_ignore_ascii_case("@global") {
        namespace.is_empty()
    } else {
        matcher::matches(namespace, pattern.as_bytes(), false, true)
    }
}

fn violates_restriction(
    restriction: &DependencyRestriction,
    source_namespace: &[u8],
    target_fqn: &[u8],
    dependency_kind: PermittedDependencyKind,
) -> bool {
    if (!restriction.kinds.is_empty() && !restriction.kinds.contains(&dependency_kind))
        || !matches_selector(target_fqn, &restriction.dependency)
    {
        return false;
    }

    let explicitly_denied =
        restriction.deny_from.iter().any(|pattern| matches_source_namespace(source_namespace, pattern));
    let allowed = restriction.allow_from.is_empty()
        || restriction.allow_from.iter().any(|pattern| matches_source_namespace(source_namespace, pattern));

    explicitly_denied || !allowed
}

fn matches_selector(target_fqn: &[u8], selector: &SymbolSelector) -> bool {
    match selector {
        SymbolSelector::Namespace(namespace) => match namespace {
            NamespacePath::Global => !target_fqn.contains(&b'\\'),
            NamespacePath::Specific(pattern) => matcher::matches(target_fqn, pattern.as_bytes(), false, true),
        },
        SymbolSelector::Symbol(symbol) => target_fqn.eq_ignore_ascii_case(symbol.as_bytes()),
        SymbolSelector::Pattern(pattern) => matcher::matches(target_fqn, pattern.as_bytes(), false, false),
    }
}

/// Checks if a fully qualified name is considered native/builtin.
fn is_native(codebase: &CodebaseMetadata, fqn: &[u8]) -> bool {
    codebase
        .get_class_like(fqn)
        .map(|c| &c.flags)
        .or_else(|| codebase.get_function(fqn).map(|f| &f.flags))
        .or_else(|| codebase.get_constant(fqn).map(|c| &c.flags))
        .is_some_and(|flags| flags.is_built_in())
}

fn get_layer_index(namespace: &[u8], settings: &Settings) -> Option<usize> {
    for (i, layer_namespace) in settings.perimeter.layering.iter().enumerate() {
        match layer_namespace {
            NamespacePath::Global if namespace.is_empty() => {
                return Some(i);
            }
            NamespacePath::Specific(ns) if matcher::matches(namespace, ns.as_bytes(), false, true) => {
                return Some(i);
            }
            _ => {}
        }
    }

    None
}

/// Checks if a target FQN is allowed based on a specific path configuration.
fn is_path_allowed(
    codebase: &CodebaseMetadata,
    settings: &Settings,
    path: &Path,
    rule_namespace: &NamespacePath,
    target_fqn: &[u8],
) -> bool {
    match path {
        Path::All => true,
        Path::Native => is_native(codebase, target_fqn),
        Path::Self_ => match rule_namespace {
            NamespacePath::Global => !target_fqn.contains(&b'\\'),
            NamespacePath::Specific(namespace) => matcher::matches(target_fqn, namespace.as_bytes(), false, true),
        },
        Path::Layer(layer_name) => settings.perimeter.layers.get(layer_name).is_some_and(|layer_patterns| {
            layer_patterns
                .iter()
                .any(|pattern| is_path_allowed(codebase, settings, pattern, rule_namespace, target_fqn))
        }),
        Path::Selector(selector) => matches_selector(target_fqn, selector),
    }
}
