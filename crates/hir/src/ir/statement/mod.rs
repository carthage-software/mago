use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::identifier::Identifier;
use crate::ir::name::Name;
use crate::ir::statement::annotation::VariableBindingAnnotation;
use crate::ir::statement::definition::DefinitionStatement;
use crate::ir::r#type::Type;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use crate::ir::variable::Variable;

pub mod annotation;
pub mod definition;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Statement<'arena, S, D, E> {
    pub meta: S,
    pub span: Span,
    pub kind: StatementKind<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum StatementKind<'arena, S, D, E> {
    Inline(&'arena [u8]),
    Namespace(&'arena Namespace<'arena, S, D, E>),
    Sequence(&'arena [Statement<'arena, S, D, E>]),
    Definition(&'arena DefinitionStatement<'arena, S, D, E>),
    Declare(&'arena Declare<'arena, S, D, E>),
    Goto(Name<'arena>),
    Label(Name<'arena>),
    Try(&'arena Try<'arena, S, D, E>),
    Foreach(&'arena Foreach<'arena, S, D, E>),
    For(&'arena For<'arena, S, D, E>),
    While(&'arena While<'arena, S, D, E>),
    DoWhile(&'arena DoWhile<'arena, S, D, E>),
    Continue(Option<&'arena Expression<'arena, S, D, E>>),
    Break(Option<&'arena Expression<'arena, S, D, E>>),
    Switch(&'arena Switch<'arena, S, D, E>),
    If(&'arena If<'arena, S, D, E>),
    Return(Option<&'arena Expression<'arena, S, D, E>>),
    Expression(&'arena Expression<'arena, S, D, E>),
    Echo(&'arena [Expression<'arena, S, D, E>]),
    Global(&'arena [GlobalItem<'arena, S, D, E>]),
    Static(&'arena [StaticItem<'arena, S, D, E>]),
    VariableBindingAnnotation(&'arena VariableBindingAnnotation<'arena>),
    HaltCompiler,
    Unset(&'arena [Expression<'arena, S, D, E>]),
    Noop,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Switch<'arena, S, D, E> {
    pub subject: &'arena Expression<'arena, S, D, E>,
    pub cases: &'arena [SwitchCase<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum SwitchCase<'arena, S, D, E> {
    Expression(&'arena Expression<'arena, S, D, E>, &'arena Statement<'arena, S, D, E>),
    Default(&'arena Statement<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct If<'arena, S, D, E> {
    pub condition: &'arena Expression<'arena, S, D, E>,
    pub then: &'arena Statement<'arena, S, D, E>,
    pub r#else: Option<&'arena Statement<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DoWhile<'arena, S, D, E> {
    pub statement: &'arena Statement<'arena, S, D, E>,
    pub condition: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct While<'arena, S, D, E> {
    pub condition: &'arena Expression<'arena, S, D, E>,
    pub statement: &'arena Statement<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct For<'arena, S, D, E> {
    pub initializations: &'arena [Expression<'arena, S, D, E>],
    pub conditions: &'arena [Expression<'arena, S, D, E>],
    pub increments: &'arena [Expression<'arena, S, D, E>],
    pub statement: &'arena Statement<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Foreach<'arena, S, D, E> {
    pub expression: &'arena Expression<'arena, S, D, E>,
    pub key: Option<&'arena Expression<'arena, S, D, E>>,
    pub value: &'arena Expression<'arena, S, D, E>,
    pub statement: &'arena Statement<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Try<'arena, S, D, E> {
    pub statement: &'arena Statement<'arena, S, D, E>,
    pub catch_clauses: &'arena [TryCatchClause<'arena, S, D, E>],
    pub finally_clause: Option<&'arena Statement<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TryCatchClause<'arena, S, D, E> {
    pub r#type: &'arena Type<'arena>,
    pub variable: Option<DirectVariable<'arena>>,
    pub statement: &'arena Statement<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Namespace<'arena, S, D, E> {
    pub name: Option<&'arena Identifier<'arena>>,
    pub statement: &'arena Statement<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StaticItem<'arena, S, D, E> {
    pub variable: DirectVariable<'arena>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub value: Option<&'arena Expression<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GlobalItem<'arena, S, D, E> {
    pub variable: Variable<'arena, S, D, E>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DeclareItem<'arena, S, D, E> {
    pub name: Name<'arena>,
    pub value: Option<&'arena Expression<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Declare<'arena, S, D, E> {
    pub items: &'arena [DeclareItem<'arena, S, D, E>],
    pub statement: &'arena Statement<'arena, S, D, E>,
}

impl<S, D, E> HasSpan for Statement<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}
