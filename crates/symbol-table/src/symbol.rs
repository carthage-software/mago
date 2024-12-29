use mago_source::SourceCategory;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_source::HasSource;
use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

/// Represents the different kinds of symbols that can be encountered in the code.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
pub enum SymbolKind {
    /// A class definition.
    Class,
    /// A trait definition.
    Trait,
    /// An enum definition.
    Enum,
    /// An interface definition.
    Interface,
    /// An anonymous class definition.
    AnonymousClass,
    /// A function definition.
    Function,
    /// A constant definition.
    Constant,
    /// A class-like constant definition (e.g., a static constant within a class).
    ClassLikeConstant,
    /// A method definition.
    Method,
    /// An enum case definition.
    EnumCase,
    /// A property definition.
    Property,
    /// An arrow function definition.
    ArrowFunction,
    /// A closure definition (anonymous function).
    Closure,
}

/// Encapsulates the name and fully qualified name of a symbol, along with its span in the source code.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SymbolIdentifier {
    /// The name of the symbol.
    pub name: StringIdentifier,
    /// The fully qualified name of the symbol, including its namespace.
    ///
    /// If the symbol is a class-like member (e.g., a method, property, or constant),
    ///  this will be the name of the class-like symbol, followed by a `::`, followed by the name of the member,
    ///  unless the class-like symbol is an anonymous class, in which case this will be the name of the member.
    pub fully_qualified_name: StringIdentifier,
    /// The span of the symbol's name in the source code.
    pub span: Span,
}

/// A lightweight reference to a symbol, containing its kind, identifier (if named), and location.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SymbolReference {
    /// The kind of the referenced symbol.
    pub kind: SymbolKind,
    /// The identifier of the symbol, if it has a name. This will be `None` for anonymous symbols like closures or anonymous classes.
    pub identifier: Option<SymbolIdentifier>,
    /// The span of the full definition of the symbol in the source code.
    pub span: Span,
}

/// Represents a symbol in the code, such as a class, function, variable, etc.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Symbol {
    /// The kind of the symbol.
    pub kind: SymbolKind,
    /// The namespace of the symbol, if any.
    pub namespace: Option<StringIdentifier>,
    /// The identifier of the symbol, if it has a name. This will be `None` for anonymous symbols like closures or anonymous classes.
    pub identifier: Option<SymbolIdentifier>,
    /// The span of the full definition of the symbol in the source code.
    pub span: Span,
    /// A reference to the lexically enclosing scope of this symbol, if any.
    /// This can be the containing class for a method, the containing function for a nested function, etc.
    pub scope: Option<SymbolReference>,
}

impl SymbolKind {
    /// Returns `true` if the symbol kind represents a class-like type, including classes, traits, enums, interfaces, and anonymous classes.
    pub fn is_class_like(&self) -> bool {
        matches!(
            self,
            SymbolKind::Class
                | SymbolKind::Trait
                | SymbolKind::Enum
                | SymbolKind::Interface
                | SymbolKind::AnonymousClass
        )
    }

    /// Returns `true` if the symbol kind represents a function-like type, including functions, methods, arrow functions, and closures.
    pub fn is_function_like(&self) -> bool {
        matches!(self, SymbolKind::Function | SymbolKind::Method | SymbolKind::ArrowFunction | SymbolKind::Closure)
    }

    /// Returns `true` if the symbol kind represents a constant-like type, including constants and class-like constants.
    pub fn is_constant_like(&self) -> bool {
        matches!(self, SymbolKind::Constant | SymbolKind::ClassLikeConstant)
    }

    /// Returns `true` if the symbol kind represents an anonymous type, such as an anonymous class, arrow function, or closure.
    pub fn is_anonymous(&self) -> bool {
        matches!(self, SymbolKind::AnonymousClass | SymbolKind::ArrowFunction | SymbolKind::Closure)
    }
}

impl Symbol {
    /// Converts the symbol into a `SymbolReference`.
    pub fn to_reference(&self) -> SymbolReference {
        SymbolReference { kind: self.kind, identifier: self.identifier, span: self.span }
    }

    /// Returns `true` if the symbol is user-defined within the project or comes from an external library.
    pub const fn category(&self) -> SourceCategory {
        self.span.start.source.category()
    }
}

impl HasSpan for Symbol {
    /// Returns the span of the full definition of the symbol in the source code.
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSource for Symbol {
    /// Returns the source identifier of the file containing the symbol.
    fn source(&self) -> SourceIdentifier {
        self.span.source()
    }
}

impl HasSpan for SymbolReference {
    /// Returns the span of the full definition of the referenced symbol in the source code.
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSource for SymbolReference {
    /// Returns the source identifier of the file containing the referenced symbol.
    fn source(&self) -> SourceIdentifier {
        self.span.source()
    }
}
