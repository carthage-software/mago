#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
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
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

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

impl<I, S, E> CopyInto for Statement<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Statement<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Statement { meta: self.meta.copy_into(arena), span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for StatementKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = StatementKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            StatementKind::Inline(bytes) => StatementKind::Inline(arena.alloc_slice_copy(bytes)),
            StatementKind::Namespace(node) => StatementKind::Namespace(copy_ref_into(*node, arena)),
            StatementKind::Sequence(statements) => StatementKind::Sequence(copy_slice_into(statements, arena)),
            StatementKind::Item(node) => StatementKind::Item(copy_ref_into(*node, arena)),
            StatementKind::Declare(node) => StatementKind::Declare(copy_ref_into(*node, arena)),
            StatementKind::Goto(name) => StatementKind::Goto(name.copy_into(arena)),
            StatementKind::Label(name) => StatementKind::Label(name.copy_into(arena)),
            StatementKind::Try(node) => StatementKind::Try(copy_ref_into(*node, arena)),
            StatementKind::Foreach(node) => StatementKind::Foreach(copy_ref_into(*node, arena)),
            StatementKind::For(node) => StatementKind::For(copy_ref_into(*node, arena)),
            StatementKind::While(node) => StatementKind::While(copy_ref_into(*node, arena)),
            StatementKind::DoWhile(node) => StatementKind::DoWhile(copy_ref_into(*node, arena)),
            StatementKind::Continue(expression) => {
                StatementKind::Continue(expression.map(|node| copy_ref_into(node, arena)))
            }
            StatementKind::Break(expression) => StatementKind::Break(expression.map(|node| copy_ref_into(node, arena))),
            StatementKind::Switch(node) => StatementKind::Switch(copy_ref_into(*node, arena)),
            StatementKind::If(node) => StatementKind::If(copy_ref_into(*node, arena)),
            StatementKind::Return(expression) => {
                StatementKind::Return(expression.map(|node| copy_ref_into(node, arena)))
            }
            StatementKind::Expression(node) => StatementKind::Expression(copy_ref_into(*node, arena)),
            StatementKind::Echo(expressions) => StatementKind::Echo(copy_slice_into(expressions, arena)),
            StatementKind::Global(items) => StatementKind::Global(copy_slice_into(items, arena)),
            StatementKind::Static(items) => StatementKind::Static(copy_slice_into(items, arena)),
            StatementKind::VariableBindingAnnotation(node) => {
                StatementKind::VariableBindingAnnotation(copy_ref_into(*node, arena))
            }
            StatementKind::HaltCompiler => StatementKind::HaltCompiler,
            StatementKind::Unset(delimited) => StatementKind::Unset(delimited.copy_into(arena)),
            StatementKind::Noop => StatementKind::Noop,
        }
    }
}

impl<I, S, E> CopyInto for Switch<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Switch<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Switch { span: self.span, subject: copy_ref_into(self.subject, arena), cases: self.cases.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for SwitchCase<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = SwitchCase<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            SwitchCase::Expression(expression, statement) => {
                SwitchCase::Expression(copy_ref_into(*expression, arena), copy_ref_into(*statement, arena))
            }
            SwitchCase::Default(statement) => SwitchCase::Default(copy_ref_into(*statement, arena)),
        }
    }
}

impl<I, S, E> CopyInto for If<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = If<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        If {
            span: self.span,
            condition: copy_ref_into(self.condition, arena),
            then: copy_ref_into(self.then, arena),
            r#else: self.r#else.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl<I, S, E> CopyInto for DoWhile<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = DoWhile<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        DoWhile {
            span: self.span,
            statement: copy_ref_into(self.statement, arena),
            condition: copy_ref_into(self.condition, arena),
        }
    }
}

impl<I, S, E> CopyInto for While<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = While<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        While {
            span: self.span,
            condition: copy_ref_into(self.condition, arena),
            statement: copy_ref_into(self.statement, arena),
        }
    }
}

impl<I, S, E> CopyInto for For<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = For<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        For {
            span: self.span,
            initializations: copy_slice_into(self.initializations, arena),
            conditions: copy_slice_into(self.conditions, arena),
            increments: copy_slice_into(self.increments, arena),
            statement: copy_ref_into(self.statement, arena),
        }
    }
}

impl<I, S, E> CopyInto for Foreach<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Foreach<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Foreach {
            span: self.span,
            expression: copy_ref_into(self.expression, arena),
            key: self.key.map(|node| copy_ref_into(node, arena)),
            value: copy_ref_into(self.value, arena),
            statement: copy_ref_into(self.statement, arena),
        }
    }
}

impl<I, S, E> CopyInto for Try<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Try<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Try {
            span: self.span,
            statement: copy_ref_into(self.statement, arena),
            catch_clauses: copy_slice_into(self.catch_clauses, arena),
            finally_clause: self.finally_clause.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl<I, S, E> CopyInto for TryCatchClause<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = TryCatchClause<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        TryCatchClause {
            span: self.span,
            r#type: copy_ref_into(self.r#type, arena),
            variable: self.variable.map(|node| node.copy_into(arena)),
            statement: copy_ref_into(self.statement, arena),
        }
    }
}

impl<I, S, E> CopyInto for Namespace<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Namespace<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Namespace {
            span: self.span,
            name: self.name.map(|node| copy_ref_into(node, arena)),
            statement: copy_ref_into(self.statement, arena),
        }
    }
}

impl<I, S, E> CopyInto for StaticItem<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = StaticItem<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        StaticItem {
            span: self.span,
            variable: self.variable.copy_into(arena),
            type_annotation: self.type_annotation.map(|node| copy_ref_into(node, arena)),
            value: self.value.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl<I, S, E> CopyInto for GlobalItem<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = GlobalItem<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        GlobalItem {
            span: self.span,
            variable: self.variable.copy_into(arena),
            type_annotation: self.type_annotation.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl<I, S, E> CopyInto for DeclareItem<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = DeclareItem<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        DeclareItem {
            span: self.span,
            name: self.name.copy_into(arena),
            value: self.value.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl<I, S, E> CopyInto for Declare<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Declare<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Declare { span: self.span, items: self.items.copy_into(arena), statement: copy_ref_into(self.statement, arena) }
    }
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
