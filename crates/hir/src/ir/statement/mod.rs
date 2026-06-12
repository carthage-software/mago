#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::expression::Expression;
use crate::ir::identifier::Identifier;
use crate::ir::item::statement::ItemStatement;
use crate::ir::name::Name;
use crate::ir::statement::annotation::VariableBindingAnnotation;
use crate::ir::r#type::Type;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use crate::ir::variable::Variable;

pub mod annotation;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Statement<'arena, I, S, E> {
    pub meta: S,
    pub span: Span,
    pub kind: StatementKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum StatementKind<'arena, I, S, E> {
    Inline(&'arena [u8]),
    Namespace(&'arena Namespace<'arena, I, S, E>),
    Sequence(&'arena [Statement<'arena, I, S, E>]),
    Item(&'arena ItemStatement<'arena, I, S, E>),
    Declare(&'arena Declare<'arena, I, S, E>),
    Goto(Name<'arena>),
    Label(Name<'arena>),
    Try(&'arena Try<'arena, I, S, E>),
    Foreach(&'arena Foreach<'arena, I, S, E>),
    For(&'arena For<'arena, I, S, E>),
    While(&'arena While<'arena, I, S, E>),
    DoWhile(&'arena DoWhile<'arena, I, S, E>),
    Continue(Option<&'arena Expression<'arena, I, S, E>>),
    Break(Option<&'arena Expression<'arena, I, S, E>>),
    Switch(&'arena Switch<'arena, I, S, E>),
    If(&'arena If<'arena, I, S, E>),
    Return(Option<&'arena Expression<'arena, I, S, E>>),
    Expression(&'arena Expression<'arena, I, S, E>),
    Echo(&'arena [Expression<'arena, I, S, E>]),
    Global(&'arena [GlobalItem<'arena, I, S, E>]),
    Static(&'arena [StaticItem<'arena, I, S, E>]),
    VariableBindingAnnotation(&'arena VariableBindingAnnotation<'arena>),
    HaltCompiler,
    Unset(Delimited<'arena, Expression<'arena, I, S, E>>),
    Noop,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Switch<'arena, I, S, E> {
    pub span: Span,
    pub subject: &'arena Expression<'arena, I, S, E>,
    pub cases: Delimited<'arena, SwitchCase<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum SwitchCase<'arena, I, S, E> {
    Expression(&'arena Expression<'arena, I, S, E>, &'arena Statement<'arena, I, S, E>),
    Default(&'arena Statement<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct If<'arena, I, S, E> {
    pub span: Span,
    pub condition: &'arena Expression<'arena, I, S, E>,
    pub then: &'arena Statement<'arena, I, S, E>,
    pub r#else: Option<&'arena Statement<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct DoWhile<'arena, I, S, E> {
    pub span: Span,
    pub statement: &'arena Statement<'arena, I, S, E>,
    pub condition: &'arena Expression<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct While<'arena, I, S, E> {
    pub span: Span,
    pub condition: &'arena Expression<'arena, I, S, E>,
    pub statement: &'arena Statement<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct For<'arena, I, S, E> {
    pub span: Span,
    pub initializations: &'arena [Expression<'arena, I, S, E>],
    pub conditions: &'arena [Expression<'arena, I, S, E>],
    pub increments: &'arena [Expression<'arena, I, S, E>],
    pub statement: &'arena Statement<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Foreach<'arena, I, S, E> {
    pub span: Span,
    pub expression: &'arena Expression<'arena, I, S, E>,
    pub key: Option<&'arena Expression<'arena, I, S, E>>,
    pub value: &'arena Expression<'arena, I, S, E>,
    pub statement: &'arena Statement<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Try<'arena, I, S, E> {
    pub span: Span,
    pub statement: &'arena Statement<'arena, I, S, E>,
    pub catch_clauses: &'arena [TryCatchClause<'arena, I, S, E>],
    pub finally_clause: Option<&'arena Statement<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TryCatchClause<'arena, I, S, E> {
    pub span: Span,
    pub r#type: &'arena Type<'arena>,
    pub variable: Option<DirectVariable<'arena>>,
    pub statement: &'arena Statement<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Namespace<'arena, I, S, E> {
    pub span: Span,
    pub name: Option<&'arena Identifier<'arena>>,
    pub statement: &'arena Statement<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct StaticItem<'arena, I, S, E> {
    pub span: Span,
    pub variable: DirectVariable<'arena>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub value: Option<&'arena Expression<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct GlobalItem<'arena, I, S, E> {
    pub span: Span,
    pub variable: Variable<'arena, I, S, E>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct DeclareItem<'arena, I, S, E> {
    pub span: Span,
    pub name: Name<'arena>,
    pub value: Option<&'arena Expression<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Declare<'arena, I, S, E> {
    pub span: Span,
    pub items: Delimited<'arena, DeclareItem<'arena, I, S, E>>,
    pub statement: &'arena Statement<'arena, I, S, E>,
}

impl<I, S, E> HasSpan for Statement<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Switch<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for If<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for DoWhile<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for While<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for For<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Foreach<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Try<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for TryCatchClause<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Namespace<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for StaticItem<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for GlobalItem<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for DeclareItem<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Declare<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for SwitchCase<'_, I, S, E> {
    fn span(&self) -> Span {
        match self {
            SwitchCase::Expression(expression, statement) => expression.span().join(statement.span()),
            SwitchCase::Default(statement) => statement.span(),
        }
    }
}
