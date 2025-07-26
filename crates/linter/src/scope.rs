use mago_codex::get_anonymous_class;
use mago_codex::get_class;
use mago_codex::get_closure;
use mago_codex::get_enum;
use mago_codex::get_function;
use mago_codex::get_interface;
use mago_codex::get_method;
use mago_codex::get_trait;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_interner::StringIdentifier;
use mago_span::Position;
use mago_span::Span;

use crate::context::LintContext;

/// Represents a scope related to class-like constructs.
/// This includes classes, interfaces, traits, enums, and anonymous classes.
/// The identifier or span stored in each variant is used to retrieve the corresponding metadata.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClassLikeScope {
    /// A regular class identified by its name.
    Class(StringIdentifier),
    /// An interface identified by its name.
    Interface(StringIdentifier),
    /// A trait identified by its name.
    Trait(StringIdentifier),
    /// An enum identified by its name.
    Enum(StringIdentifier),
    /// An anonymous class identified by its source code span.
    AnonymousClass(Span),
}

/// Represents a scope related to function-like constructs.
/// This includes functions, methods, closures, and arrow functions.
/// The identifier or span stored in each variant is used to retrieve the corresponding metadata.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionLikeScope {
    /// A regular function identified by its name.
    Function(StringIdentifier),
    /// A method identified by its name.
    Method(StringIdentifier),
    /// A closure or an arrow function identified by its source code span.
    Closure(Position),
}

/// Represents an entry in the scope stack. It can be either class-like or function-like.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Scope {
    /// A class-like scope entry.
    ClassLike(ClassLikeScope),
    /// A function-like scope entry.
    FunctionLike(FunctionLikeScope),
}

/// A stack that tracks the scopes entered during linting.
///
/// The `ScopeStack` supports pushing and popping scope entries, and provides
/// methods to retrieve the metadata for the most recently entered scope of a
/// particular kind. For example, calling `get_class_metadata` returns the metadata
/// for the last entered class scope (if any), and similar for function-like scopes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScopeStack {
    stack: Vec<Scope>,
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeStack {
    /// Creates a new, empty `ScopeStack`.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Pushes a new `Scope` onto the stack.
    ///
    /// # Parameters
    ///
    /// - `scope`: The scope to push onto the stack.
    pub fn push(&mut self, scope: Scope) {
        self.stack.push(scope);
    }

    /// Pops the most recently pushed `Scope` from the stack.
    ///
    /// Returns `Some(Scope)` if the stack was not empty, or `None` otherwise.
    pub fn pop(&mut self) -> Option<Scope> {
        self.stack.pop()
    }

    /// Retrieves a reference to the most recently pushed class-like scope, regardless of its variant.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeScope)` if a class-like scope exists in the stack.
    /// - `None` if no class-like scope is found.
    pub fn get_class_like_scope(&self) -> Option<&ClassLikeScope> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(class_like) => Some(class_like),
            _ => None,
        })
    }

    /// Retrieves a reference to the most recently pushed function-like scope, regardless of its variant.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeScope)` if a function-like scope exists in the stack.
    /// - `None` if no function-like scope is found.
    pub fn get_function_like_scope(&self) -> Option<&FunctionLikeScope> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::FunctionLike(function_like) => Some(function_like),
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent class-like scope, regardless of its specific variant.
    ///
    /// This method uses the provided `context` to look up the corresponding metadata in the codebase.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeMetadata)` if the metadata is found.
    /// - `None` if no matching metadata is found.
    pub fn get_class_like_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a ClassLikeMetadata> {
        self.get_class_like_scope().and_then(|class_like| match class_like {
            ClassLikeScope::Class(name) => get_class(context.codebase, context.interner, name),
            ClassLikeScope::Interface(name) => get_interface(context.codebase, context.interner, name),
            ClassLikeScope::Trait(name) => get_trait(context.codebase, context.interner, name),
            ClassLikeScope::Enum(name) => get_enum(context.codebase, context.interner, name),
            ClassLikeScope::AnonymousClass(span) => get_anonymous_class(context.codebase, context.interner, *span),
        })
    }

    /// Retrieves the metadata for the most recent class scope.
    ///
    /// This method searches the scope stack (in reverse order) for the first entry that
    /// is a `Class` variant in a class-like scope, and returns its metadata using the
    /// given context.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeMetadata)` if a class metadata is found.
    /// - `None` if no class scope is present in the stack.
    pub fn get_class_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a ClassLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(ClassLikeScope::Class(name)) => context
                .codebase
                .class_likes
                .get(&context.interner.lowered(name))
                .filter(|metadata| metadata.kind.is_class()),
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent interface scope.
    ///
    /// This method searches the scope stack for the first occurrence of an `Interface` variant
    /// within a class-like scope and returns its metadata using the provided context.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeMetadata)` if an interface metadata is found.
    /// - `None` if no interface scope is present.
    pub fn get_interface_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a ClassLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(ClassLikeScope::Interface(name)) => {
                get_interface(context.codebase, context.interner, name)
            }
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent trait scope.
    ///
    /// This method searches the scope stack for the first occurrence of a `Trait` variant
    /// within a class-like scope and returns its metadata using the provided context.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeMetadata)` if a trait metadata is found.
    /// - `None` if no trait scope is present.
    pub fn get_trait_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a ClassLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(ClassLikeScope::Trait(name)) => get_trait(context.codebase, context.interner, name),
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent enum scope.
    ///
    /// This method searches the scope stack for the first occurrence of an `Enum` variant
    /// within a class-like scope and returns its metadata using the provided context.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeMetadata)` if an enum metadata is found.
    /// - `None` if no enum scope is present.
    pub fn get_enum_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a ClassLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(ClassLikeScope::Enum(name)) => get_enum(context.codebase, context.interner, name),
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent anonymous class scope.
    ///
    /// This method searches the scope stack for the first occurrence of an `AnonymousClass` variant
    /// within a class-like scope and returns its metadata using the provided context.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeMetadata)` if an anonymous class metadata is found.
    /// - `None` if no anonymous class scope is present.
    pub fn get_anonymous_class_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a ClassLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(ClassLikeScope::AnonymousClass(span)) => {
                get_anonymous_class(context.codebase, context.interner, *span)
            }
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent function-like scope,
    /// regardless of its specific variant (function, method, closure, or arrow function).
    ///
    /// This method uses the provided context to fetch the corresponding metadata from the codebase.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeMetadata)` if a function-like metadata is found.
    /// - `None` if no function-like scope is present.
    pub fn get_function_like_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a FunctionLikeMetadata> {
        self.get_function_like_scope().and_then(|function_like| match function_like {
            FunctionLikeScope::Function(name) => get_function(context.codebase, context.interner, name),
            FunctionLikeScope::Closure(position) => get_closure(context.codebase, context.interner, position),
            FunctionLikeScope::Method(name) => {
                let class_like = self.get_class_like_metadata(context)?;

                get_method(context.codebase, context.interner, &class_like.name, name)
            }
        })
    }

    /// Retrieves the metadata for the most recent function scope.
    ///
    /// This method searches the scope stack for the first occurrence of a `Function` variant
    /// within a function-like scope and returns its metadata using the provided context.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeMetadata)` if a function metadata is found.
    /// - `None` if no function scope is present.
    pub fn get_function_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a FunctionLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::FunctionLike(FunctionLikeScope::Function(name)) => {
                get_function(context.codebase, context.interner, name)
            }
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent closure scope.
    ///
    /// This method searches the scope stack for the first occurrence of a `Closure` variant
    /// within a function-like scope and returns its metadata using the provided context.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeMetadata)` if a closure metadata is found.
    /// - `None` if no closure scope is present.
    pub fn get_closure_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a FunctionLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::FunctionLike(FunctionLikeScope::Closure(position)) => {
                get_closure(context.codebase, context.interner, position)
            }
            _ => None,
        })
    }

    /// Retrieves the metadata for the most recent method scope.
    ///
    /// This method searches the scope stack for the first occurrence of a `Method` variant
    /// within a function-like scope. It then retrieves the metadata for the containing
    /// class-like scope and uses it to obtain the method metadata from the context's codebase.
    ///
    /// # Parameters
    ///
    /// - `context`: The linting context containing the codebase and interner.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeMetadata)` if a method metadata is found.
    /// - `None` if no method scope is present or if the containing class-like metadata is not available.
    pub fn get_method_metadata<'a>(&self, context: &'a LintContext) -> Option<&'a FunctionLikeMetadata> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::FunctionLike(FunctionLikeScope::Method(name)) => {
                let class_like = self.get_class_like_metadata(context)?;

                get_method(context.codebase, context.interner, &class_like.name, name)
            }
            _ => None,
        })
    }
}
