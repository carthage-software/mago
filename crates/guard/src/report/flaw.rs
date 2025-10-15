use std::fmt;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::settings::StructuralInheritanceConstraint;
use crate::settings::StructuralSymbolKind;

/// Represents a single structural flaw where code does not adhere to defined architectural rules.
#[derive(Debug)]
pub struct StructuralFlaw {
    /// The fully qualified name of the symbol that has a flaw.
    pub symbol_fqn: String,
    /// The kind of the flawed symbol (e.g., class-like, function).
    pub symbol_kind: StructuralSymbolKind,
    /// The source code location of the flawed symbol's definition.
    pub span: Span,
    /// The specific kind why the symbol's structure is considered flawed.
    pub kind: FlawKind,
    /// An optional human-readable explanation for the flaw.
    pub reason: Option<String>,
}

/// Describes the specific kind of structural flaw.
#[derive(Debug)]
pub enum FlawKind {
    MustBeNamed { pattern: String },
    MustBeFinal,
    MustNotBeFinal,
    MustBeAbstract,
    MustNotBeAbstract,
    MustBeReadonly,
    MustNotBeReadonly,
    MustImplement { expected: StructuralInheritanceConstraint },
    MustExtend { expected: StructuralInheritanceConstraint },
    MustUseTrait { expected: StructuralInheritanceConstraint },
    MustUseAttribute { expected: StructuralInheritanceConstraint },
    MustBe { allowed: Vec<StructuralSymbolKind> },
}

impl From<StructuralFlaw> for Issue {
    /// Converts a `StructuralFlaw` into a rich, user-friendly `Issue`.
    fn from(flaw: StructuralFlaw) -> Self {
        let mut issue = Issue::error(format!("Structural flaw in `{}`", flaw.symbol_fqn))
            .with_annotation(Annotation::primary(flaw.span).with_message(flaw.kind.to_string()));

        if let Some(reason) = flaw.reason {
            issue = issue.with_note(reason);
        }

        let help = match &flaw.kind {
            FlawKind::MustBeNamed { pattern } => {
                format!("Rename this {} to match the pattern: `{}`.", flaw.symbol_kind, pattern)
            }
            FlawKind::MustBeFinal => format!("Declare this {} as `final`.", flaw.symbol_kind),
            FlawKind::MustNotBeFinal => format!("Remove the `final` modifier from this {}.", flaw.symbol_kind),
            FlawKind::MustBeAbstract => format!("Declare this {} as `abstract`.", flaw.symbol_kind),
            FlawKind::MustNotBeAbstract => format!("Remove the `abstract` modifier from this {}.", flaw.symbol_kind),
            FlawKind::MustBeReadonly => format!("Declare this {} as `readonly`.", flaw.symbol_kind),
            FlawKind::MustNotBeReadonly => format!("Remove the `readonly` modifier from this {}.", flaw.symbol_kind),
            FlawKind::MustImplement { .. } => {
                format!("Ensure this {} implements the required interface(s).", flaw.symbol_kind)
            }
            FlawKind::MustExtend { .. } => {
                format!("Ensure this {} extends the required base class.", flaw.symbol_kind)
            }
            FlawKind::MustUseTrait { .. } => format!("Ensure this {} uses the required trait(s).", flaw.symbol_kind),
            FlawKind::MustUseAttribute { .. } => {
                format!("Ensure this {} uses the required attribute(s).", flaw.symbol_kind)
            }
            FlawKind::MustBe { .. } => {
                "Move this symbol to a different namespace or update your guard configuration.".to_string()
            }
        };

        issue.with_help(help)
    }
}

impl fmt::Display for FlawKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MustBeNamed { pattern } => write!(f, "Name does not match pattern `{}`", pattern),
            Self::MustBeFinal => write!(f, "This must be declared as `final`"),
            Self::MustNotBeFinal => write!(f, "This must not be declared as `final`"),
            Self::MustBeAbstract => write!(f, "This must be declared as `abstract`"),
            Self::MustNotBeAbstract => write!(f, "This must not be declared as `abstract`"),
            Self::MustBeReadonly => write!(f, "This must be declared as `readonly`"),
            Self::MustNotBeReadonly => write!(f, "This must not be declared as `readonly`"),
            Self::MustImplement { expected } => {
                write!(f, "Does not implement required interface(s): {}", expected)
            }
            Self::MustExtend { expected } => {
                write!(f, "Does not extend required base class: {}", expected)
            }
            Self::MustUseTrait { expected } => {
                write!(f, "Does not use required trait(s): {}", expected)
            }
            Self::MustUseAttribute { expected } => {
                write!(f, "Does not use required attribute(s): {}", expected)
            }
            Self::MustBe { allowed } => {
                let allowed_str = allowed.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                write!(f, "This namespace should only contain: {}", allowed_str)
            }
        }
    }
}

impl FlawKind {
    /// Returns the error code for this type of flaw.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::MustBeNamed { .. } => "must-be-named",
            Self::MustBeFinal => "must-be-final",
            Self::MustNotBeFinal => "must-be-non-final",
            Self::MustBeAbstract => "must-be-abstract",
            Self::MustNotBeAbstract => "must-be-non-abstract",
            Self::MustBeReadonly => "must-be-readonly",
            Self::MustNotBeReadonly => "must-be-non-readonly",
            Self::MustImplement { .. } => "must-implement",
            Self::MustExtend { .. } => "must-extend",
            Self::MustUseTrait { .. } => "must-use-trait",
            Self::MustUseAttribute { .. } => "must-use-attribute",
            Self::MustBe { .. } => "must-be",
        }
    }
}
