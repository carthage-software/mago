use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::metadata::ttype::TypeMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArgumentMetadata {
    Named { name: StringIdentifier, inferred_type: Option<TypeMetadata>, span: Span },
    Positional { inferred_type: Option<TypeMetadata>, is_variadic: bool, span: Span },
}

impl ArgumentMetadata {
    #[inline]
    pub fn new_named(name: StringIdentifier, span: Span) -> Self {
        ArgumentMetadata::Named { name, inferred_type: None, span }
    }

    #[inline]
    pub fn new_positional(span: Span) -> Self {
        ArgumentMetadata::Positional { inferred_type: None, is_variadic: false, span }
    }

    #[inline]
    pub fn set_inferred_type(&mut self, inferred_type: Option<TypeMetadata>) {
        match self {
            ArgumentMetadata::Named { inferred_type: dest, .. } => *dest = inferred_type,
            ArgumentMetadata::Positional { inferred_type: dest, .. } => *dest = inferred_type,
        }
    }

    #[inline]
    pub fn with_inferred_type(mut self, inferred_type: Option<TypeMetadata>) -> Self {
        self.set_inferred_type(inferred_type);
        self
    }

    #[inline]
    pub fn unset_inferred_type(&mut self) {
        match self {
            ArgumentMetadata::Named { inferred_type: dest, .. } => *dest = None,
            ArgumentMetadata::Positional { inferred_type: dest, .. } => *dest = None,
        }
    }

    #[inline]
    pub fn without_inferred_type(mut self) -> Self {
        self.unset_inferred_type();
        self
    }

    #[inline]
    pub fn set_variadic(&mut self, is_variadic: bool) {
        if let ArgumentMetadata::Positional { is_variadic: dest, .. } = self {
            *dest = is_variadic
        }
    }

    #[inline]
    pub fn with_variadic(mut self, is_variadic: bool) -> Self {
        self.set_variadic(is_variadic);
        self
    }

    #[inline]
    pub const fn get_name(&self) -> Option<StringIdentifier> {
        match self {
            ArgumentMetadata::Named { name, .. } => Some(*name),
            ArgumentMetadata::Positional { .. } => None,
        }
    }

    #[inline]
    pub const fn get_inferred_type(&self) -> Option<&TypeMetadata> {
        match self {
            ArgumentMetadata::Named { inferred_type, .. } => inferred_type.as_ref(),
            ArgumentMetadata::Positional { inferred_type, .. } => inferred_type.as_ref(),
        }
    }

    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            ArgumentMetadata::Named { span, .. } => *span,
            ArgumentMetadata::Positional { span, .. } => *span,
        }
    }

    #[inline]
    pub const fn is_named(&self) -> bool {
        matches!(self, ArgumentMetadata::Named { .. })
    }

    #[inline]
    pub const fn is_positional(&self) -> bool {
        matches!(self, ArgumentMetadata::Positional { .. })
    }

    #[inline]
    pub const fn is_variadic(&self) -> bool {
        match self {
            ArgumentMetadata::Named { .. } => false,
            ArgumentMetadata::Positional { is_variadic, .. } => *is_variadic,
        }
    }
}

impl HasSpan for ArgumentMetadata {
    fn span(&self) -> Span {
        match self {
            ArgumentMetadata::Named { span, .. } => *span,
            ArgumentMetadata::Positional { span, .. } => *span,
        }
    }
}
