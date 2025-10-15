use std::fmt;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::path::NamespacePath;
use crate::settings::PermittedDependencyKind;

/// Represents a single, specific architectural boundary breach found by the guard.
#[derive(Debug)]
pub struct BoundaryBreach {
    /// The namespace where the breach was detected.
    pub source_namespace: String,
    /// The fully qualified name of the symbol that was illegally used.
    pub dependency_fqn: String,
    /// The kind of the dependency being used (e.g., class-like, function).
    pub dependency_kind: PermittedDependencyKind,
    /// The specific code context where the breach occurred (e.g., an `extends` clause).
    pub vector: BreachVector,
    /// The source code location of the breach.
    pub span: Span,
    /// The logical reason why this dependency is considered a breach.
    pub reason: BreachReason,
}

/// Describes the specific "vector" or method by which a boundary breach occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreachVector {
    Extends,
    Implements,
    Use,
    TraitUse,
    PropertyType,
    ParameterType,
    ReturnType,
    Instantiation,
    StaticMethodCall,
    StaticPropertyAccess,
    ClassConstantAccess,
    FunctionCall,
    ConstantAccess,
    Attribute,
}

/// Explains the underlying architectural rule that was broken.
#[derive(Debug)]
pub enum BreachReason {
    /// A lower layer is trying to access a higher, more protected layer.
    Layering { source_layer: NamespacePath, target_layer: NamespacePath },
    /// No rule was found that explicitly allows this dependency.
    NoMatchingRule,
    /// One or more rules were evaluated, but none permitted this specific dependency.
    ForbiddenByRule { rule_namespaces: Vec<NamespacePath> },
}

impl From<BoundaryBreach> for Issue {
    fn from(breach: BoundaryBreach) -> Self {
        let mut issue = Issue::error(format!("Illegal dependency on `{}`", breach.dependency_fqn))
            .with_annotation(
                Annotation::primary(breach.span)
                    .with_message(format!("This {} is not allowed by the architectural rules", breach.vector)),
            )
            .with_note(format!("Breach occurred in namespace `{}`.", breach.source_namespace));

        match breach.reason {
            BreachReason::Layering { source_layer, target_layer } => {
                issue = issue.with_note("Layering Rule Conflict").with_note(format!(
                    "The `{}` layer is not allowed to depend on the `{}` layer.",
                    source_layer, target_layer
                ));
            }
            BreachReason::NoMatchingRule => {
                issue = issue
                    .with_note("No matching architectural rule found")
                    .with_note("No rule was found that explicitly allows this dependency.");
            }
            BreachReason::ForbiddenByRule { rule_namespaces } => {
                let namespaces = rule_namespaces.iter().map(|ns| format!("`{}`", ns)).collect::<Vec<_>>().join(", ");

                issue = issue.with_note("Dependency forbidden by architectural rules").with_note(format!(
                    "The following rule(s) were evaluated but none permitted this dependency: {}.",
                    namespaces
                ));
            }
        };

        issue.with_help("Update your guard configuration to allow this dependency or refactor the code to remove it.")
    }
}

impl fmt::Display for BreachVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Self::Extends => "extends clause",
            Self::Implements => "implements clause",
            Self::Use => "`use` statement",
            Self::TraitUse => "trait `use`",
            Self::PropertyType => "property type-hint",
            Self::ParameterType => "parameter type-hint",
            Self::ReturnType => "return type-hint",
            Self::Instantiation => "instantiation",
            Self::StaticMethodCall => "static method call",
            Self::StaticPropertyAccess => "static property access",
            Self::ClassConstantAccess => "class constant access",
            Self::FunctionCall => "function call",
            Self::ConstantAccess => "constant usage",
            Self::Attribute => "attribute usage",
        };

        write!(f, "{}", description)
    }
}

impl BreachVector {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Extends => "disallowed-extends",
            Self::Implements => "disallowed-implements",
            Self::Use => "disallowed-use",
            Self::TraitUse => "disallowed-trait-use",
            Self::PropertyType => "disallowed-property-type",
            Self::ParameterType => "disallowed-parameter-type",
            Self::ReturnType => "disallowed-return-type",
            Self::Instantiation => "disallowed-instantiation",
            Self::StaticMethodCall => "disallowed-static-call",
            Self::StaticPropertyAccess => "disallowed-static-property",
            Self::ClassConstantAccess => "disallowed-class-constant",
            Self::FunctionCall => "disallowed-function-call",
            Self::ConstantAccess => "disallowed-constant-usage",
            Self::Attribute => "disallowed-attribute",
        }
    }
}
