use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::argument::ArgumentList;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Construct<'arena> {
    Isset(IssetConstruct<'arena>),
    Empty(EmptyConstruct<'arena>),
    Eval(EvalConstruct<'arena>),
    Include(IncludeConstruct<'arena>),
    IncludeOnce(IncludeOnceConstruct<'arena>),
    Require(RequireConstruct<'arena>),
    RequireOnce(RequireOnceConstruct<'arena>),
    Print(PrintConstruct<'arena>),
    Exit(ExitConstruct<'arena>),
    Die(DieConstruct<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IssetConstruct<'arena> {
    pub isset: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub values: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EmptyConstruct<'arena> {
    pub empty: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub value: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EvalConstruct<'arena> {
    pub eval: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub value: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IncludeConstruct<'arena> {
    pub include: Keyword<'arena>,
    pub value: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IncludeOnceConstruct<'arena> {
    pub include_once: Keyword<'arena>,
    pub value: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct RequireConstruct<'arena> {
    pub require: Keyword<'arena>,
    pub value: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct RequireOnceConstruct<'arena> {
    pub require_once: Keyword<'arena>,
    pub value: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PrintConstruct<'arena> {
    pub print: Keyword<'arena>,
    pub value: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ExitConstruct<'arena> {
    pub exit: Keyword<'arena>,
    pub arguments: Option<ArgumentList<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DieConstruct<'arena> {
    pub die: Keyword<'arena>,
    pub arguments: Option<ArgumentList<'arena>>,
}

impl<'arena> Construct<'arena> {
    #[must_use]
    #[inline]
    pub const fn is_import(&self) -> bool {
        matches!(
            self,
            Construct::Include(_) | Construct::IncludeOnce(_) | Construct::Require(_) | Construct::RequireOnce(_)
        )
    }

    #[must_use]
    #[inline]
    pub const fn has_bounds(&self) -> bool {
        !matches!(
            self,
            Construct::Include(_)
                | Construct::IncludeOnce(_)
                | Construct::Require(_)
                | Construct::RequireOnce(_)
                | Construct::Print(_)
        )
    }
}

impl HasSpan for Construct<'_> {
    fn span(&self) -> Span {
        match self {
            Construct::Isset(c) => c.span(),
            Construct::Empty(c) => c.span(),
            Construct::Eval(c) => c.span(),
            Construct::Include(c) => c.span(),
            Construct::IncludeOnce(c) => c.span(),
            Construct::Require(c) => c.span(),
            Construct::RequireOnce(c) => c.span(),
            Construct::Print(c) => c.span(),
            Construct::Exit(c) => c.span(),
            Construct::Die(c) => c.span(),
        }
    }
}

impl HasSpan for IssetConstruct<'_> {
    fn span(&self) -> Span {
        self.isset.span().join(self.right_parenthesis.span())
    }
}

impl HasSpan for EmptyConstruct<'_> {
    fn span(&self) -> Span {
        self.empty.span().join(self.right_parenthesis)
    }
}

impl HasSpan for EvalConstruct<'_> {
    fn span(&self) -> Span {
        self.eval.span().join(self.right_parenthesis)
    }
}

impl HasSpan for IncludeConstruct<'_> {
    fn span(&self) -> Span {
        self.include.span().join(self.value.span())
    }
}

impl HasSpan for IncludeOnceConstruct<'_> {
    fn span(&self) -> Span {
        self.include_once.span().join(self.value.span())
    }
}

impl HasSpan for RequireConstruct<'_> {
    fn span(&self) -> Span {
        self.require.span().join(self.value.span())
    }
}

impl HasSpan for RequireOnceConstruct<'_> {
    fn span(&self) -> Span {
        self.require_once.span().join(self.value.span())
    }
}

impl HasSpan for PrintConstruct<'_> {
    fn span(&self) -> Span {
        self.print.span().join(self.value.span())
    }
}

impl HasSpan for ExitConstruct<'_> {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.arguments { self.exit.span().join(arguments.span()) } else { self.exit.span() }
    }
}

impl HasSpan for DieConstruct<'_> {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.arguments { self.die.span().join(arguments.span()) } else { self.die.span() }
    }
}
