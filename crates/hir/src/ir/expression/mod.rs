use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::expression::definition::DefinitionExpression;
use crate::ir::expression::operator::AssignmentOperator;
use crate::ir::expression::operator::BinaryOperator;
use crate::ir::expression::operator::PostfixUnaryOperator;
use crate::ir::expression::operator::PrefixUnaryOperator;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::identifier::Identifier;
use crate::ir::literal::Literal;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::Variable;

pub mod definition;
pub mod operator;
pub mod selector;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Expression<'arena, S, D, E> {
    pub meta: E,
    pub span: Span,
    pub kind: ExpressionKind<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum ExpressionKind<'arena, S, D, E> {
    Binary(&'arena Binary<'arena, S, D, E>),
    UnaryPrefix(&'arena PrefixUnary<'arena, S, D, E>),
    UnaryPostfix(&'arena PostfixUnary<'arena, S, D, E>),
    Literal(&'arena Literal<'arena>),
    CompositeString(&'arena [CompositeStringPart<'arena, S, D, E>]),
    Assignment(&'arena Assignment<'arena, S, D, E>),
    Conditional(&'arena Conditional<'arena, S, D, E>),
    Array(&'arena [ArrayElement<'arena, S, D, E>]),
    List(&'arena [ArrayElement<'arena, S, D, E>]),
    ArrayAppend(&'arena Expression<'arena, S, D, E>),
    Definition(&'arena DefinitionExpression<'arena, S, D, E>),
    Call(&'arena Call<'arena, S, D, E>),
    PartialApplication(&'arena PartialApplication<'arena, S, D, E>),
    Access(&'arena Access<'arena, S, D, E>),
    Clone(&'arena Expression<'arena, S, D, E>),
    Empty(&'arena Expression<'arena, S, D, E>),
    Eval(&'arena Expression<'arena, S, D, E>),
    Include(&'arena Expression<'arena, S, D, E>),
    IncludeOnce(&'arena Expression<'arena, S, D, E>),
    Require(&'arena Expression<'arena, S, D, E>),
    RequireOnce(&'arena Expression<'arena, S, D, E>),
    Print(&'arena Expression<'arena, S, D, E>),
    Isset(&'arena [Expression<'arena, S, D, E>]),
    Exit(&'arena [Argument<'arena, S, D, E>]),
    MagicConstant(MagicConstant),
    Constant(Identifier<'arena>),
    Instantiation(&'arena Instantiation<'arena, S, D, E>),
    Variable(Variable<'arena, S, D, E>),
    Yield(&'arena Yield<'arena, S, D, E>),
    Throw(&'arena Expression<'arena, S, D, E>),
    Parent,
    Self_,
    Static,
    Match(&'arena Match<'arena, S, D, E>),
    Identifier(Identifier<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Assignment<'arena, S, D, E> {
    pub left: &'arena Expression<'arena, S, D, E>,
    pub operator: Option<AssignmentOperator>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub right: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Binary<'arena, S, D, E> {
    pub left: &'arena Expression<'arena, S, D, E>,
    pub operator: BinaryOperator,
    pub right: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PrefixUnary<'arena, S, D, E> {
    pub operator: PrefixUnaryOperator,
    pub operand: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PostfixUnary<'arena, S, D, E> {
    pub operand: &'arena Expression<'arena, S, D, E>,
    pub operator: PostfixUnaryOperator,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Conditional<'arena, S, D, E> {
    pub condition: &'arena Expression<'arena, S, D, E>,
    pub then: Option<&'arena Expression<'arena, S, D, E>>,
    pub r#else: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum MagicConstant {
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Instantiation<'arena, S, D, E> {
    pub class: &'arena Expression<'arena, S, D, E>,
    pub arguments: &'arena [Argument<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum Callee<'arena, S, D, E> {
    Function(&'arena Expression<'arena, S, D, E>),
    Method(&'arena Expression<'arena, S, D, E>, MemberSelector<'arena, S, D, E>),
    NullsafeMethod(&'arena Expression<'arena, S, D, E>, MemberSelector<'arena, S, D, E>),
    StaticMethod(&'arena Expression<'arena, S, D, E>, MemberSelector<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Call<'arena, S, D, E> {
    pub callee: Callee<'arena, S, D, E>,
    pub arguments: &'arena [Argument<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PartialApplication<'arena, S, D, E> {
    pub callee: Callee<'arena, S, D, E>,
    pub arguments: &'arena [PartialArgument<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum Access<'arena, S, D, E> {
    Array(&'arena Expression<'arena, S, D, E>, &'arena Expression<'arena, S, D, E>),
    Property(&'arena Expression<'arena, S, D, E>, MemberSelector<'arena, S, D, E>),
    NullsafeProperty(&'arena Expression<'arena, S, D, E>, MemberSelector<'arena, S, D, E>),
    StaticProperty(&'arena Expression<'arena, S, D, E>, Variable<'arena, S, D, E>),
    ClassConstant(&'arena Expression<'arena, S, D, E>, ConstantSelector<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum Yield<'arena, S, D, E> {
    Expression(&'arena Expression<'arena, S, D, E>),
    Pair(&'arena Expression<'arena, S, D, E>, &'arena Expression<'arena, S, D, E>),
    From(&'arena Expression<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Match<'arena, S, D, E> {
    pub subject: &'arena Expression<'arena, S, D, E>,
    pub arms: &'arena [MatchArm<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum MatchArm<'arena, S, D, E> {
    Expression(&'arena [Expression<'arena, S, D, E>], &'arena Expression<'arena, S, D, E>),
    Default(&'arena Expression<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum ArrayElement<'arena, S, D, E> {
    KeyValue(&'arena Expression<'arena, S, D, E>, &'arena Expression<'arena, S, D, E>),
    Value(&'arena Expression<'arena, S, D, E>),
    Variadic(&'arena Expression<'arena, S, D, E>),
    Missing,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum CompositeStringPart<'arena, S, D, E> {
    Literal(&'arena [u8]),
    Expression(&'arena Expression<'arena, S, D, E>),
}

impl<S, D, E> HasSpan for Expression<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}
