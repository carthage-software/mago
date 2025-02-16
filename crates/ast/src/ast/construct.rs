use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::argument::ArgumentList;
use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Construct<'a> {
    Isset(IssetConstruct<'a>),
    Empty(EmptyConstruct<'a>),
    Eval(EvalConstruct<'a>),
    Include(IncludeConstruct<'a>),
    IncludeOnce(IncludeOnceConstruct<'a>),
    Require(RequireConstruct<'a>),
    RequireOnce(RequireOnceConstruct<'a>),
    Print(PrintConstruct<'a>),
    Exit(ExitConstruct<'a>),
    Die(DieConstruct<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IssetConstruct<'a> {
    pub isset: Keyword,
    pub left_parenthesis: Span,
    pub values: TokenSeparatedSequence<'a, Expression<'a>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct EmptyConstruct<'a> {
    pub empty: Keyword,
    pub left_parenthesis: Span,
    pub value: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct EvalConstruct<'a> {
    pub eval: Keyword,
    pub left_parenthesis: Span,
    pub value: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IncludeConstruct<'a> {
    pub include: Keyword,
    pub value: Box<'a, Expression<'a>>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IncludeOnceConstruct<'a> {
    pub include_once: Keyword,
    pub value: Box<'a, Expression<'a>>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct RequireConstruct<'a> {
    pub require: Keyword,
    pub value: Box<'a, Expression<'a>>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct RequireOnceConstruct<'a> {
    pub require_once: Keyword,
    pub value: Box<'a, Expression<'a>>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct PrintConstruct<'a> {
    pub print: Keyword,
    pub value: Box<'a, Expression<'a>>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ExitConstruct<'a> {
    pub exit: Keyword,
    pub arguments: Option<ArgumentList<'a>>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct DieConstruct<'a> {
    pub die: Keyword,
    pub arguments: Option<ArgumentList<'a>>,
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
        if let Some(arguments) = &self.arguments {
            self.exit.span().join(arguments.span())
        } else {
            self.exit.span()
        }
    }
}

impl HasSpan for DieConstruct<'_> {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.arguments {
            self.die.span().join(arguments.span())
        } else {
            self.die.span()
        }
    }
}
