use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_project::module::Module;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Program;
use mago_syntax::walker::Walker;

use crate::internal::context::Context;
use crate::internal::walker::ReferenceFindingWalker;
use crate::query::Query;

pub mod query;

mod internal;

/// Represents the kind of reference that is found in code.
///
/// Each variant corresponds to a specific way in which a symbol might be
/// referred to in code. For instance, an import statement, a usage, or
/// a definition site.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ReferenceKind {
    /// Indicates where a symbol is imported into a scope.
    Import,

    /// Refers to places in code where the symbol is used.
    Usage,

    /// Points to the definition site(s) of the symbol.
    Definition,

    /// Identifies where a symbol (like an interface) is
    /// implemented, or where a trait is applied.
    Implementation,

    /// Tracks places where a class is extended.
    Extension,
}

/// Describes a single reference to a symbol in the source code.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Reference {
    /// The identifier for the referenced symbol, as stored in the interner.
    pub value: StringIdentifier,

    /// The kind of reference—import, usage, definition, etc. (see [`ReferenceKind`]).
    pub kind: ReferenceKind,

    /// The [`Span`] in the source code where this reference appears.
    pub span: Span,
}

/// Provides functionality for discovering references (imports, usages, definitions, etc.)
/// of a symbol within a module.
///
/// The [`ReferenceFinder`] can locate references by walking through the AST
/// (via a [`Walker`]) and collecting relevant `Reference` items.
#[derive(Debug, Clone)]
pub struct ReferenceFinder<'a> {
    /// The interner used to resolve symbol names (e.g., function names, class names, etc.).
    interner: &'a ThreadedInterner,
}

impl<'a> ReferenceFinder<'a> {
    /// Creates a new `ReferenceFinder` that uses the given [`ThreadedInterner`]
    /// for string lookups and resolutions.
    pub fn new(interner: &'a ThreadedInterner) -> Self {
        Self { interner }
    }

    /// Finds all references that match the given [`Query`] within the provided [`Module`].
    ///
    /// This method:
    ///
    /// 1. Creates a [`Context`] that holds the interner, the query, and the current module.
    /// 2. Uses a specialized [`Walker`] (`ReferenceFindingWalker`) to traverse the AST of the program.
    /// 3. Gathers references (e.g., [`ReferenceKind::Usage`], [`ReferenceKind::Definition`]) in the context.
    /// 4. Returns all discovered references as a `Vec<Reference>`.
    ///
    /// A list of [`Reference`] objects describing where and how the symbol is referenced
    /// in the code.
    pub fn find(&self, module: &Module, program: &Program, query: Query) -> Vec<Reference> {
        let mut context = Context::new(self.interner, &query, module);

        ReferenceFindingWalker.walk_program(program, &mut context);

        context.take_references()
    }
}

impl HasSpan for Reference {
    fn span(&self) -> Span {
        self.span
    }
}
