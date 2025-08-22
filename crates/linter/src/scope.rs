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
    /// A `function` scope, containing the function name.
    Function(&'arena str),
    /// A `method` scope, containing the method name.
    Method(&'arena str),
    /// An `fn()` arrow function scope, containing its span.
    ArrowFunction(Span),
    /// A `function()` closure scope, containing its span.
    Closure(Span),
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
    stack: Vec<Scope<'arena>>,
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
                    .map(|n| n.value())
                    .map(|n| if let Some(n) = n.strip_prefix('\\') { n } else { n })
                    .unwrap_or("");

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

                Scope::FunctionLike(FunctionLikeScope::Function(function_name))
            }
            Node::Method(method) => Scope::FunctionLike(FunctionLikeScope::Method(method.name.value)),
            Node::Closure(closure) => {
                let span = closure.span();

                Scope::FunctionLike(FunctionLikeScope::Closure(span))
            }
            Node::ArrowFunction(arrow_function) => {
                let span = arrow_function.span();

                Scope::FunctionLike(FunctionLikeScope::ArrowFunction(span))
            }
            _ => {
                return None;
            }
        })
    }
}

impl<'arena> ScopeStack<'arena> {
    /// Creates a new, empty scope stack.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
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
    pub fn get_class_like_scope(&self) -> Option<ClassLikeScope<'arena>> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::ClassLike(class_like) => Some(*class_like),
            _ => None,
        })
    }

    /// Searches the stack and returns the innermost `FunctionLikeScope`.
    pub fn get_function_like_scope(&self) -> Option<FunctionLikeScope<'arena>> {
        self.stack.iter().rev().find_map(|scope| match scope {
            Scope::FunctionLike(function_like) => Some(*function_like),
            _ => None,
        })
    }
}

impl Default for ScopeStack<'_> {
    fn default() -> Self {
        Self::new()
    }
}
