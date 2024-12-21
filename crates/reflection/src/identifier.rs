use mago_interner::ThreadedInterner;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Name {
    /// The name of the symbol.
    pub value: StringIdentifier,
    /// The lowercased name of the symbol.
    pub lower: StringIdentifier,
    /// The span of the symbol name.
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
    pub fn new(value: StringIdentifier, lower: StringIdentifier, span: Span) -> Self {
        Name { value, lower, span }
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

    pub fn get_key(&self, interner: &ThreadedInterner) -> String {
        match self {
            ClassLikeName::Class(name)
            | ClassLikeName::Interface(name)
            | ClassLikeName::Enum(name)
            | ClassLikeName::Trait(name) => interner.lookup(&name.value).to_string(),
            ClassLikeName::AnonymousClass(span) => {
                format!(
                    "anonymous-class@{}:{}-{}",
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset
                )
            }
        }
    }
}

impl ClassLikeMemberName {
    pub fn get_key(&self, interner: &ThreadedInterner) -> String {
        let class_name = self.class_like.get_key(interner);
        let member_name = interner.lookup(&self.member.value);

        format!("{}::{}", class_name, member_name)
    }
}

impl FunctionLikeName {
    pub fn is_function(&self) -> bool {
        matches!(self, FunctionLikeName::Function(_))
    }

    pub fn get_key(&self, interner: &ThreadedInterner) -> String {
        match self {
            FunctionLikeName::Function(name) => interner.lookup(&name.value).to_string(),
            FunctionLikeName::Method(class_like_name, name) => {
                let class_name = class_like_name.get_key(interner);

                format!("{}::{}", class_name, interner.lookup(&name.value))
            }
            FunctionLikeName::PropertyHook(class_like_name, property_name, name) => {
                let class_name = class_like_name.get_key(interner);

                format!("{}::{}::{}", class_name, interner.lookup(&property_name.value), interner.lookup(&name.value))
            }
            FunctionLikeName::Closure(span) => {
                format!("closure@{}:{}-{}", interner.lookup(&span.start.source.0), span.start.offset, span.end.offset)
            }
            FunctionLikeName::ArrowFunction(span) => {
                format!(
                    "arrow-function@{}:{}-{}",
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset
                )
            }
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
