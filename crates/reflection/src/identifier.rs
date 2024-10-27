use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ClassLikeIdentifier {
    Class(StringIdentifier, Span),
    Interface(StringIdentifier, Span),
    Enum(StringIdentifier, Span),
    Trait(StringIdentifier, Span),
    AnonymousClass(Span),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ConstantIdentifier {
    name: StringIdentifier,
    span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ClassLikeMemberIdentifier {
    class_like: ClassLikeIdentifier,
    name: StringIdentifier,
    span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum FunctionLikeIdentifier {
    Function(StringIdentifier, Span),
    Method(StringIdentifier, StringIdentifier, Span),
    PropertyHook(StringIdentifier, StringIdentifier, StringIdentifier, Span),
    Closure(Span),
    ArrowFunction(Span),
}
