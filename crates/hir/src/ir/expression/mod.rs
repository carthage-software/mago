#[cfg(feature = "serde")]
use serde::Serialize;

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
pub struct CompositeStringPart<'arena, I, S, E> {
    pub span: Span,
    pub kind: CompositeStringPartKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum CompositeStringPartKind<'arena, I, S, E> {
    Literal(&'arena [u8]),
    Expression(&'arena Expression<'arena, I, S, E>),
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

impl<I, S, E> HasSpan for CompositeStringPart<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
