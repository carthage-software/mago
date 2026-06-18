#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::delimited::Delimited;
use crate::ir::expression::annotation::Annotation;
use crate::ir::expression::operator::AssignmentOperator;
use crate::ir::expression::operator::BinaryOperator;
use crate::ir::expression::operator::UnaryPostfixOperator;
use crate::ir::expression::operator::UnaryPrefixOperator;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::identifier::Identifier;
use crate::ir::item::expression::ItemExpression;
use crate::ir::literal::Literal;
use crate::ir::variable::Variable;

pub mod annotation;
pub mod operator;
pub mod selector;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Expression<'arena, I, S, E> {
    pub span: Span,
    pub meta: E,
    pub kind: ExpressionKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ExpressionKind<'arena, I, S, E> {
    Parenthesized(&'arena Expression<'arena, I, S, E>),
    Binary(&'arena Binary<'arena, I, S, E>),
    UnaryPrefix(&'arena UnaryPrefix<'arena, I, S, E>),
    UnaryPostfix(&'arena UnaryPostfix<'arena, I, S, E>),
    Literal(&'arena Literal<'arena>),
    CompositeString(&'arena [CompositeStringPart<'arena, I, S, E>]),
    ShellExecute(&'arena [CompositeStringPart<'arena, I, S, E>]),
    Assignment(&'arena Assignment<'arena, I, S, E>),
    Annotation(&'arena Annotation<'arena, I, S, E>),
    Conditional(&'arena Conditional<'arena, I, S, E>),
    Array(Delimited<'arena, ArrayElement<'arena, I, S, E>>),
    List(Delimited<'arena, ArrayElement<'arena, I, S, E>>),
    ArrayAppend(&'arena Expression<'arena, I, S, E>),
    Item(&'arena ItemExpression<'arena, I, S, E>),
    Call(&'arena Call<'arena, I, S, E>),
    PartialApplication(&'arena PartialApplication<'arena, I, S, E>),
    Access(&'arena Access<'arena, I, S, E>),
    Clone(&'arena Expression<'arena, I, S, E>),
    Empty(&'arena Expression<'arena, I, S, E>),
    Eval(&'arena Expression<'arena, I, S, E>),
    Include(&'arena Expression<'arena, I, S, E>),
    IncludeOnce(&'arena Expression<'arena, I, S, E>),
    Require(&'arena Expression<'arena, I, S, E>),
    RequireOnce(&'arena Expression<'arena, I, S, E>),
    Print(&'arena Expression<'arena, I, S, E>),
    Isset(Delimited<'arena, Expression<'arena, I, S, E>>),
    Exit(Option<Delimited<'arena, Argument<'arena, I, S, E>>>),
    MagicConstant(MagicConstant),
    Constant(Identifier<'arena>),
    Instantiation(&'arena Instantiation<'arena, I, S, E>),
    Variable(Variable<'arena, I, S, E>),
    Yield(&'arena Yield<'arena, I, S, E>),
    Throw(&'arena Expression<'arena, I, S, E>),
    Parent,
    Self_,
    Static,
    Match(&'arena Match<'arena, I, S, E>),
    Identifier(Identifier<'arena>),
    Error(Span),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Assignment<'arena, I, S, E> {
    pub span: Span,
    pub left: &'arena Expression<'arena, I, S, E>,
    pub operator: Option<AssignmentOperator>,
    pub right: &'arena Expression<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Binary<'arena, I, S, E> {
    pub span: Span,
    pub left: &'arena Expression<'arena, I, S, E>,
    pub operator: BinaryOperator,
    pub right: &'arena Expression<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct UnaryPrefix<'arena, I, S, E> {
    pub span: Span,
    pub operator: UnaryPrefixOperator,
    pub operand: &'arena Expression<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct UnaryPostfix<'arena, I, S, E> {
    pub span: Span,
    pub operand: &'arena Expression<'arena, I, S, E>,
    pub operator: UnaryPostfixOperator,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Conditional<'arena, I, S, E> {
    pub span: Span,
    pub condition: &'arena Expression<'arena, I, S, E>,
    pub then: Option<&'arena Expression<'arena, I, S, E>>,
    pub r#else: &'arena Expression<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct MagicConstant {
    pub span: Span,
    pub kind: MagicConstantKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum MagicConstantKind {
    Line,
    File,
    Directory,
    Trait,
    Method,
    Function,
    Property,
    Namespace,
    Class,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Instantiation<'arena, I, S, E> {
    pub span: Span,
    pub class: &'arena Expression<'arena, I, S, E>,
    pub arguments: Option<Delimited<'arena, Argument<'arena, I, S, E>>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Callee<'arena, I, S, E> {
    pub span: Span,
    pub kind: CalleeKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum CalleeKind<'arena, I, S, E> {
    Function(&'arena Expression<'arena, I, S, E>),
    Method(&'arena Expression<'arena, I, S, E>, MemberSelector<'arena, I, S, E>),
    NullsafeMethod(&'arena Expression<'arena, I, S, E>, MemberSelector<'arena, I, S, E>),
    StaticMethod(&'arena Expression<'arena, I, S, E>, MemberSelector<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Call<'arena, I, S, E> {
    pub span: Span,
    pub callee: Callee<'arena, I, S, E>,
    pub arguments: Delimited<'arena, Argument<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct PartialApplication<'arena, I, S, E> {
    pub span: Span,
    pub callee: Callee<'arena, I, S, E>,
    pub arguments: Delimited<'arena, PartialArgument<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Access<'arena, I, S, E> {
    pub span: Span,
    pub kind: AccessKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum AccessKind<'arena, I, S, E> {
    Array(&'arena Expression<'arena, I, S, E>, &'arena Expression<'arena, I, S, E>),
    Property(&'arena Expression<'arena, I, S, E>, MemberSelector<'arena, I, S, E>),
    NullsafeProperty(&'arena Expression<'arena, I, S, E>, MemberSelector<'arena, I, S, E>),
    StaticProperty(&'arena Expression<'arena, I, S, E>, Variable<'arena, I, S, E>),
    ClassConstant(&'arena Expression<'arena, I, S, E>, ConstantSelector<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Yield<'arena, I, S, E> {
    pub span: Span,
    pub kind: YieldKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum YieldKind<'arena, I, S, E> {
    Nothing,
    Expression(&'arena Expression<'arena, I, S, E>),
    Pair(&'arena Expression<'arena, I, S, E>, &'arena Expression<'arena, I, S, E>),
    From(&'arena Expression<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Match<'arena, I, S, E> {
    pub span: Span,
    pub subject: &'arena Expression<'arena, I, S, E>,
    pub arms: Delimited<'arena, MatchArm<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct MatchArm<'arena, I, S, E> {
    pub span: Span,
    pub kind: MatchArmKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum MatchArmKind<'arena, I, S, E> {
    Expression(&'arena [Expression<'arena, I, S, E>], &'arena Expression<'arena, I, S, E>),
    Default(&'arena Expression<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ArrayElement<'arena, I, S, E> {
    pub span: Span,
    pub kind: ArrayElementKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum ArrayElementKind<'arena, I, S, E> {
    KeyValue(&'arena Expression<'arena, I, S, E>, &'arena Expression<'arena, I, S, E>),
    Value(&'arena Expression<'arena, I, S, E>),
    Variadic(&'arena Expression<'arena, I, S, E>),
    Missing,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum CompositeStringPart<'arena, I, S, E> {
    /// A literal run, already decoded to its final runtime bytes (escapes
    /// resolved, heredoc indentation stripped, trailing newline removed).
    Literal(&'arena [u8]),
    /// An interpolated expression.
    Expression(&'arena Expression<'arena, I, S, E>),
}

impl<I, S, E> CopyInto for Expression<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Expression<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Expression { span: self.span, meta: self.meta.copy_into(arena), kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for ExpressionKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ExpressionKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            ExpressionKind::Parenthesized(node) => ExpressionKind::Parenthesized(copy_ref_into(*node, arena)),
            ExpressionKind::Binary(node) => ExpressionKind::Binary(copy_ref_into(*node, arena)),
            ExpressionKind::UnaryPrefix(node) => ExpressionKind::UnaryPrefix(copy_ref_into(*node, arena)),
            ExpressionKind::UnaryPostfix(node) => ExpressionKind::UnaryPostfix(copy_ref_into(*node, arena)),
            ExpressionKind::Literal(node) => ExpressionKind::Literal(copy_ref_into(*node, arena)),
            ExpressionKind::CompositeString(parts) => ExpressionKind::CompositeString(copy_slice_into(parts, arena)),
            ExpressionKind::ShellExecute(parts) => ExpressionKind::ShellExecute(copy_slice_into(parts, arena)),
            ExpressionKind::Assignment(node) => ExpressionKind::Assignment(copy_ref_into(*node, arena)),
            ExpressionKind::Annotation(node) => ExpressionKind::Annotation(copy_ref_into(*node, arena)),
            ExpressionKind::Conditional(node) => ExpressionKind::Conditional(copy_ref_into(*node, arena)),
            ExpressionKind::Array(delimited) => ExpressionKind::Array(delimited.copy_into(arena)),
            ExpressionKind::List(delimited) => ExpressionKind::List(delimited.copy_into(arena)),
            ExpressionKind::ArrayAppend(node) => ExpressionKind::ArrayAppend(copy_ref_into(*node, arena)),
            ExpressionKind::Item(node) => ExpressionKind::Item(copy_ref_into(*node, arena)),
            ExpressionKind::Call(node) => ExpressionKind::Call(copy_ref_into(*node, arena)),
            ExpressionKind::PartialApplication(node) => ExpressionKind::PartialApplication(copy_ref_into(*node, arena)),
            ExpressionKind::Access(node) => ExpressionKind::Access(copy_ref_into(*node, arena)),
            ExpressionKind::Clone(node) => ExpressionKind::Clone(copy_ref_into(*node, arena)),
            ExpressionKind::Empty(node) => ExpressionKind::Empty(copy_ref_into(*node, arena)),
            ExpressionKind::Eval(node) => ExpressionKind::Eval(copy_ref_into(*node, arena)),
            ExpressionKind::Include(node) => ExpressionKind::Include(copy_ref_into(*node, arena)),
            ExpressionKind::IncludeOnce(node) => ExpressionKind::IncludeOnce(copy_ref_into(*node, arena)),
            ExpressionKind::Require(node) => ExpressionKind::Require(copy_ref_into(*node, arena)),
            ExpressionKind::RequireOnce(node) => ExpressionKind::RequireOnce(copy_ref_into(*node, arena)),
            ExpressionKind::Print(node) => ExpressionKind::Print(copy_ref_into(*node, arena)),
            ExpressionKind::Isset(delimited) => ExpressionKind::Isset(delimited.copy_into(arena)),
            ExpressionKind::Exit(arguments) => {
                ExpressionKind::Exit(arguments.as_ref().map(|node| node.copy_into(arena)))
            }
            ExpressionKind::MagicConstant(constant) => ExpressionKind::MagicConstant(constant.copy_into(arena)),
            ExpressionKind::Constant(identifier) => ExpressionKind::Constant(identifier.copy_into(arena)),
            ExpressionKind::Instantiation(node) => ExpressionKind::Instantiation(copy_ref_into(*node, arena)),
            ExpressionKind::Variable(variable) => ExpressionKind::Variable(variable.copy_into(arena)),
            ExpressionKind::Yield(node) => ExpressionKind::Yield(copy_ref_into(*node, arena)),
            ExpressionKind::Throw(node) => ExpressionKind::Throw(copy_ref_into(*node, arena)),
            ExpressionKind::Parent => ExpressionKind::Parent,
            ExpressionKind::Self_ => ExpressionKind::Self_,
            ExpressionKind::Static => ExpressionKind::Static,
            ExpressionKind::Match(node) => ExpressionKind::Match(copy_ref_into(*node, arena)),
            ExpressionKind::Identifier(identifier) => ExpressionKind::Identifier(identifier.copy_into(arena)),
            ExpressionKind::Error(span) => ExpressionKind::Error(*span),
        }
    }
}

impl<I, S, E> CopyInto for Assignment<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Assignment<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Assignment {
            span: self.span,
            left: copy_ref_into(self.left, arena),
            operator: self.operator,
            right: copy_ref_into(self.right, arena),
        }
    }
}

impl<I, S, E> CopyInto for Binary<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Binary<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Binary {
            span: self.span,
            left: copy_ref_into(self.left, arena),
            operator: self.operator,
            right: copy_ref_into(self.right, arena),
        }
    }
}

impl<I, S, E> CopyInto for UnaryPrefix<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = UnaryPrefix<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        UnaryPrefix { span: self.span, operator: self.operator, operand: copy_ref_into(self.operand, arena) }
    }
}

impl<I, S, E> CopyInto for UnaryPostfix<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = UnaryPostfix<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        UnaryPostfix { span: self.span, operand: copy_ref_into(self.operand, arena), operator: self.operator }
    }
}

impl<I, S, E> CopyInto for Conditional<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Conditional<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Conditional {
            span: self.span,
            condition: copy_ref_into(self.condition, arena),
            then: self.then.map(|node| copy_ref_into(node, arena)),
            r#else: copy_ref_into(self.r#else, arena),
        }
    }
}

impl CopyInto for MagicConstant {
    type Output<'arena> = MagicConstant;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        MagicConstant { span: self.span, kind: self.kind }
    }
}

impl CopyInto for MagicConstantKind {
    type Output<'arena> = MagicConstantKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl<I, S, E> CopyInto for Instantiation<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Instantiation<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Instantiation {
            span: self.span,
            class: copy_ref_into(self.class, arena),
            arguments: self.arguments.as_ref().map(|node| node.copy_into(arena)),
        }
    }
}

impl<I, S, E> CopyInto for Callee<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Callee<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Callee { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for CalleeKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = CalleeKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            CalleeKind::Function(expression) => CalleeKind::Function(copy_ref_into(*expression, arena)),
            CalleeKind::Method(expression, selector) => {
                CalleeKind::Method(copy_ref_into(*expression, arena), selector.copy_into(arena))
            }
            CalleeKind::NullsafeMethod(expression, selector) => {
                CalleeKind::NullsafeMethod(copy_ref_into(*expression, arena), selector.copy_into(arena))
            }
            CalleeKind::StaticMethod(expression, selector) => {
                CalleeKind::StaticMethod(copy_ref_into(*expression, arena), selector.copy_into(arena))
            }
        }
    }
}

impl<I, S, E> CopyInto for Call<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Call<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Call { span: self.span, callee: self.callee.copy_into(arena), arguments: self.arguments.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for PartialApplication<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = PartialApplication<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        PartialApplication {
            span: self.span,
            callee: self.callee.copy_into(arena),
            arguments: self.arguments.copy_into(arena),
        }
    }
}

impl<I, S, E> CopyInto for Access<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Access<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Access { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for AccessKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = AccessKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            AccessKind::Array(target, index) => {
                AccessKind::Array(copy_ref_into(*target, arena), copy_ref_into(*index, arena))
            }
            AccessKind::Property(target, selector) => {
                AccessKind::Property(copy_ref_into(*target, arena), selector.copy_into(arena))
            }
            AccessKind::NullsafeProperty(target, selector) => {
                AccessKind::NullsafeProperty(copy_ref_into(*target, arena), selector.copy_into(arena))
            }
            AccessKind::StaticProperty(target, variable) => {
                AccessKind::StaticProperty(copy_ref_into(*target, arena), variable.copy_into(arena))
            }
            AccessKind::ClassConstant(target, selector) => {
                AccessKind::ClassConstant(copy_ref_into(*target, arena), selector.copy_into(arena))
            }
        }
    }
}

impl<I, S, E> CopyInto for Yield<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Yield<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Yield { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for YieldKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = YieldKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            YieldKind::Nothing => YieldKind::Nothing,
            YieldKind::Expression(expression) => YieldKind::Expression(copy_ref_into(*expression, arena)),
            YieldKind::Pair(key, value) => YieldKind::Pair(copy_ref_into(*key, arena), copy_ref_into(*value, arena)),
            YieldKind::From(expression) => YieldKind::From(copy_ref_into(*expression, arena)),
        }
    }
}

impl<I, S, E> CopyInto for Match<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Match<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Match { span: self.span, subject: copy_ref_into(self.subject, arena), arms: self.arms.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for MatchArm<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = MatchArm<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        MatchArm { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for MatchArmKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = MatchArmKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            MatchArmKind::Expression(conditions, body) => {
                MatchArmKind::Expression(copy_slice_into(conditions, arena), copy_ref_into(*body, arena))
            }
            MatchArmKind::Default(body) => MatchArmKind::Default(copy_ref_into(*body, arena)),
        }
    }
}

impl<I, S, E> CopyInto for ArrayElement<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ArrayElement<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ArrayElement { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for ArrayElementKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ArrayElementKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            ArrayElementKind::KeyValue(key, value) => {
                ArrayElementKind::KeyValue(copy_ref_into(*key, arena), copy_ref_into(*value, arena))
            }
            ArrayElementKind::Value(value) => ArrayElementKind::Value(copy_ref_into(*value, arena)),
            ArrayElementKind::Variadic(value) => ArrayElementKind::Variadic(copy_ref_into(*value, arena)),
            ArrayElementKind::Missing => ArrayElementKind::Missing,
        }
    }
}

impl<I, S, E> CopyInto for CompositeStringPart<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = CompositeStringPart<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            CompositeStringPart::Literal(bytes) => CompositeStringPart::Literal(arena.alloc_slice_copy(bytes)),
            CompositeStringPart::Expression(expression) => {
                CompositeStringPart::Expression(copy_ref_into(*expression, arena))
            }
        }
    }
}

impl<I, S, E> HasSpan for Expression<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Assignment<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Binary<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for UnaryPrefix<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for UnaryPostfix<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Conditional<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Instantiation<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Call<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for PartialApplication<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Match<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Callee<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Access<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Yield<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for MatchArm<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for ArrayElement<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
