use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::Span;

use crate::metadata::argument::ArgumentMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeMetadata {
    pub name: StringIdentifier,
    pub arguments: Vec<ArgumentMetadata>,
    pub name_span: Span,
    pub span: Span,
}

impl AttributeMetadata {
    pub fn new(name: StringIdentifier, name_span: Span, span: Span) -> Self {
        Self { name, arguments: Vec::new(), name_span, span }
    }

    #[inline]
    pub fn with_arguments(mut self, arguments: Vec<ArgumentMetadata>) -> Self {
        self.arguments = arguments;
        self
    }

    #[inline]
    pub fn add_argument(mut self, argument: ArgumentMetadata) -> Self {
        self.arguments.push(argument);
        self
    }

    #[inline]
    pub fn get_name(&self) -> StringIdentifier {
        self.name
    }

    #[inline]
    pub fn get_arguments(&self) -> &[ArgumentMetadata] {
        &self.arguments
    }

    #[inline]
    pub fn get_name_span(&self) -> Span {
        self.name_span
    }

    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }
}
