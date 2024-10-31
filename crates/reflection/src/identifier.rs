use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Name {
    pub value: StringIdentifier,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ClassLikeName {
    Class(Name),
    Interface(Name),
    Enum(Name),
    Trait(Name),
    AnonymousClass(Span),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ClassLikeMemberName {
    pub class_like: ClassLikeName,
    pub member: Name,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum FunctionLikeName {
    Function(Name),
    Method(ClassLikeName, Name),
    PropertyHook(ClassLikeName, Name, Name),
    Closure(Span),
    ArrowFunction(Span),
}

impl Name {
    pub fn new(value: StringIdentifier, span: Span) -> Self {
        Self { value, span }
    }
}

impl ClassLikeName {
    pub fn inner(&self) -> Option<&Name> {
        match self {
            ClassLikeName::Class(name) => Some(name),
            ClassLikeName::Interface(name) => Some(name),
            ClassLikeName::Enum(name) => Some(name),
            ClassLikeName::Trait(name) => Some(name),
            ClassLikeName::AnonymousClass(_) => None,
        }
    }
}

impl std::cmp::PartialEq<StringIdentifier> for Name {
    fn eq(&self, other: &StringIdentifier) -> bool {
        self.value == *other
    }
}

impl std::cmp::PartialEq<Name> for StringIdentifier {
    fn eq(&self, other: &Name) -> bool {
        *self == other.value
    }
}

impl std::cmp::PartialEq<StringIdentifier> for ClassLikeName {
    fn eq(&self, other: &StringIdentifier) -> bool {
        match self {
            ClassLikeName::Class(id) => id == other,
            ClassLikeName::Interface(id) => id == other,
            ClassLikeName::Enum(id) => id == other,
            ClassLikeName::Trait(id) => id == other,
            ClassLikeName::AnonymousClass(_) => false,
        }
    }
}

impl std::cmp::PartialEq<ClassLikeName> for StringIdentifier {
    fn eq(&self, other: &ClassLikeName) -> bool {
        other == self
    }
}
