use mago_phpdoc_syntax::cst::TemplateTagValueVariance;
use mago_phpdoc_syntax::cst::r#type::GenericParameterVariance;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Variance {
    Invariant,
    Covariant,
    Contravariant,
    Bivariant,
}

impl From<GenericParameterVariance<'_>> for Variance {
    fn from(variance: GenericParameterVariance<'_>) -> Self {
        match variance {
            GenericParameterVariance::Covariant(_) => Variance::Covariant,
            GenericParameterVariance::Contravariant(_) => Variance::Contravariant,
        }
    }
}

impl From<TemplateTagValueVariance> for Variance {
    fn from(variance: TemplateTagValueVariance) -> Self {
        match variance {
            TemplateTagValueVariance::Invariant => Variance::Invariant,
            TemplateTagValueVariance::Covariant => Variance::Covariant,
            TemplateTagValueVariance::Contravariant => Variance::Contravariant,
        }
    }
}

impl Variance {
    #[inline]
    #[must_use]
    pub const fn is_invariant(&self) -> bool {
        matches!(self, Variance::Invariant)
    }

    #[inline]
    #[must_use]
    pub const fn is_covariant(&self) -> bool {
        matches!(self, Variance::Covariant)
    }

    #[inline]
    #[must_use]
    pub const fn is_bivariant(&self) -> bool {
        matches!(self, Variance::Bivariant)
    }

    #[inline]
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Variance::Covariant => Variance::Contravariant,
            Variance::Contravariant => Variance::Covariant,
            other => other,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        matches!(self, Variance::Covariant | Variance::Invariant)
    }

    #[inline]
    #[must_use]
    pub const fn project(self, polarity: Variance) -> Option<bool> {
        let reads = matches!(self, Variance::Covariant | Variance::Invariant);
        let writes = matches!(self, Variance::Contravariant | Variance::Invariant);

        match polarity {
            Variance::Contravariant => {
                if writes {
                    Some(true)
                } else {
                    None
                }
            }
            _ => {
                if reads {
                    Some(true)
                } else {
                    Some(false)
                }
            }
        }
    }
}

impl std::fmt::Display for Variance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variance::Invariant => write!(f, "invariant"),
            Variance::Covariant => write!(f, "covariant"),
            Variance::Contravariant => write!(f, "contravariant"),
            Variance::Bivariant => write!(f, "*"),
        }
    }
}
