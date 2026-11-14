use mago_allocator::prelude::*;

use mago_twig_syntax::cst::Node;

/// A lexical scope that holds the macros imported by `{% from %}` tags.
#[derive(Debug)]
pub enum Scope<'arena, A>
where
    A: Arena,
{
    /// A template: its imports reach every nested block and macro.
    Template(Vec<'arena, &'arena [u8], A>),
    /// An `embed` tag: its body is a template of its own, but its template name and
    /// `with` variables still belong to the enclosing scope until the body starts.
    Embed { body: bool, imports: Vec<'arena, &'arena [u8], A> },
    /// A `block` or `macro` body: its imports stay inside it.
    Local(Vec<'arena, &'arena [u8], A>),
}

/// A stack that tracks the scopes the walker is currently in.
#[derive(Debug)]
pub struct ScopeStack<'arena, A>
where
    A: Arena,
{
    stack: Vec<'arena, Scope<'arena, A>, A>,
}

impl<'arena, A> Scope<'arena, A>
where
    A: Arena,
{
    /// Creates a `Scope` from a CST `Node` if that node defines a new scope.
    pub fn for_node(arena: &'arena A, node: Node<'_, 'arena>) -> Option<Self> {
        match node {
            Node::Template(_) => Some(Scope::Template(Vec::new_in(arena))),
            Node::Embed(_) => Some(Scope::Embed { body: false, imports: Vec::new_in(arena) }),
            Node::Block(_) | Node::Macro(_) => Some(Scope::Local(Vec::new_in(arena))),
            _ => None,
        }
    }

    fn is_entered(&self) -> bool {
        !matches!(self, Scope::Embed { body: false, .. })
    }

    fn is_template(&self) -> bool {
        matches!(self, Scope::Template(_) | Scope::Embed { body: true, .. })
    }

    fn imports(&self) -> &[&'arena [u8]] {
        match self {
            Scope::Template(imports) | Scope::Embed { imports, .. } | Scope::Local(imports) => imports,
        }
    }

    fn imports_mut(&mut self) -> &mut Vec<'arena, &'arena [u8], A> {
        match self {
            Scope::Template(imports) | Scope::Embed { imports, .. } | Scope::Local(imports) => imports,
        }
    }
}

impl<'arena, A> ScopeStack<'arena, A>
where
    A: Arena,
{
    pub fn new_in(arena: &'arena A) -> Self {
        Self { stack: Vec::with_capacity_in(8, arena) }
    }

    pub fn push(&mut self, scope: Scope<'arena, A>) {
        self.stack.push(scope);
    }

    pub fn pop(&mut self) -> Option<Scope<'arena, A>> {
        self.stack.pop()
    }

    /// Enters the body of the innermost `embed` tag.
    pub fn enter_embed_body(&mut self) {
        if let Some(Scope::Embed { body, .. }) = self.stack.last_mut() {
            *body = true;
        }
    }

    /// Records a macro imported into the current scope.
    pub fn import(&mut self, name: &'arena [u8]) {
        if let Some(scope) = self.stack.iter_mut().rev().find(|scope| scope.is_entered()) {
            scope.imports_mut().push(name);
        }
    }

    /// Checks whether `name` refers to a macro imported into the current scope or its template.
    #[must_use]
    pub fn is_imported(&self, name: &[u8]) -> bool {
        let mut scopes = self.stack.iter().rev().filter(|scope| scope.is_entered());
        let Some(current) = scopes.next() else {
            return false;
        };

        if current.imports().contains(&name) {
            return true;
        }

        !current.is_template() && scopes.find(|scope| scope.is_template()).is_some_and(|t| t.imports().contains(&name))
    }
}
