use bumpalo::Bump;
use bumpalo::collections::Vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Node;

use crate::context::LintContext;

/// Represents a class-like lexical scope.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClassLikeScope<'arena> {
    /// A `class` scope, containing the class name.
    Class(&'arena str),
    /// An `interface` scope, containing the interface name.
    Interface(&'arena str),
    /// A `trait` scope, containing the trait name.
    Trait(&'arena str),
    /// An `enum` scope, containing the enum name.
    Enum(&'arena str),
    /// An anonymous `class` scope, containing the span of the `new class` expression.
    AnonymousClass(Span),
}

/// Represents a function-like lexical scope.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionLikeScope<'arena> {
    /// A `function` scope, containing the function name, and if the function returns by-ref.
    Function(&'arena str, bool),
    /// A `method` scope, containing the method name, and if the method returns by-ref.
    Method(&'arena str, bool),
    /// An `fn()` arrow function scope, containing its span, and if it returns by-ref.
    ArrowFunction(Span, bool),
    /// A `function()` closure scope, containing its span, and if it returns by-ref.
    Closure(Span, bool),
}

/// Represents a single level of lexical scope within the AST.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Scope<'arena> {
    /// A `namespace` scope.
    Namespace(&'arena str),
    /// Any class-like scope (`class`, `interface`, `trait`, `enum`).
    ClassLike(ClassLikeScope<'arena>),
    /// Any function-like scope (`function`, `method`, `closure`).
    FunctionLike(FunctionLikeScope<'arena>),
}

/// A stack that tracks the current nesting of lexical scopes during AST traversal.
///
/// As the node walker descends into scope-defining nodes (like classes or functions),
/// it pushes a new `Scope` onto this stack. When it exits that node, it pops the
/// scope off. This allows rules to query the current context at any point.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScopeStack<'arena> {
    stack: Vec<'arena, Scope<'arena>>,
}

impl FunctionLikeScope<'_> {
    #[must_use]
    pub const fn is_by_ref(&self) -> bool {
        match self {
            FunctionLikeScope::Function(_, by_ref)
            | FunctionLikeScope::Method(_, by_ref)
            | FunctionLikeScope::ArrowFunction(_, by_ref)
            | FunctionLikeScope::Closure(_, by_ref) => *by_ref,
        }
    }
}

impl<'arena> Scope<'arena> {
    /// Creates a `Scope` from an AST `Node` if that node defines a new scope.
    ///
    /// Returns `None` if the node does not define a scope.
    pub fn for_node<'ast>(ctx: &LintContext<'_, 'arena>, node: Node<'ast, 'arena>) -> Option<Self> {
        Some(match node {
            Node::Namespace(namespace) => {
                let namespace_name = namespace
                    .name
                    .as_ref()
                    .map(mago_syntax::ast::Identifier::value)
                    .map_or("", |n| if let Some(n) = n.strip_prefix('\\') { n } else { n });

                Scope::Namespace(namespace_name)
            }
            Node::Class(class) => {
                let class_name = ctx.lookup_name(&class.name);

                Scope::ClassLike(ClassLikeScope::Class(class_name))
            }
            Node::Interface(interface) => {
                let interface_name = ctx.lookup_name(&interface.name);

                Scope::ClassLike(ClassLikeScope::Interface(interface_name))
            }
            Node::Trait(trait_node) => {
                let trait_name = ctx.lookup_name(&trait_node.name);

                Scope::ClassLike(ClassLikeScope::Trait(trait_name))
            }
            Node::Enum(enum_node) => Scope::ClassLike(ClassLikeScope::Enum(enum_node.name.value)),
            Node::AnonymousClass(anonymous_class) => {
                let span = anonymous_class.span();

                Scope::ClassLike(ClassLikeScope::AnonymousClass(span))
            }
            Node::Function(function) => {
                let function_name = ctx.lookup_name(&function.name);

                Scope::FunctionLike(FunctionLikeScope::Function(function_name, function.ampersand.is_some()))
            }
            Node::Method(method) => {
                Scope::FunctionLike(FunctionLikeScope::Method(method.name.value, method.ampersand.is_some()))
            }
            Node::Closure(closure) => {
                let span = closure.span();

                Scope::FunctionLike(FunctionLikeScope::Closure(span, closure.ampersand.is_some()))
            }
            Node::ArrowFunction(arrow_function) => {
                let span = arrow_function.span();

                Scope::FunctionLike(FunctionLikeScope::ArrowFunction(span, arrow_function.ampersand.is_some()))
            }
            _ => {
                return None;
            }
        })
    }
}

impl<'arena> ScopeStack<'arena> {
    /// Creates a new, empty scope stack.
    #[must_use]
    pub fn new_in(arena: &'arena Bump) -> Self {
        Self { stack: Vec::with_capacity_in(4, arena) }
    }

    /// Pushes a new scope onto the stack.
    ///
    /// This is called by the walker when it enters a scope-defining node.
    pub fn push(&mut self, scope: Scope<'arena>) {
        self.stack.push(scope);
    }

    /// Pops the current scope from the stack.
    ///
    /// This is called by the walker when it exits a scope-defining node.
    pub fn pop(&mut self) -> Option<Scope<'arena>> {
        self.stack.pop()
    }

    /// Searches the stack and returns the name of the current namespace.
    ///
    /// Returns an empty string if in the global scope.
    #[must_use]
    pub fn get_namespace(&self) -> &'arena str {
        self.stack
            .iter()
            .rev()
            .find_map(|scope| match scope {
                Scope::Namespace(namespace) => Some(*namespace),
                _ => None,
            })
            .unwrap_or("")
    }

    /// Searches the stack and returns the innermost `ClassLikeScope`.
    #[must_use]
    pub fn get_class_like_scope(&self) -> Option<ClassLikeScope<'arena>> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(class_like) => Some(*class_like),
            _ => None,
        })
    }

    /// Searches the stack and returns the innermost `FunctionLikeScope`.
    #[must_use]
    pub fn get_function_like_scope(&self) -> Option<FunctionLikeScope<'arena>> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::FunctionLike(function_like) => Some(*function_like),
            _ => None,
        })
    }
}
