//! Universal walker enum over every Twig AST node.

use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::Apply;
use crate::cst::Argument;
use crate::cst::ArgumentList;
use crate::cst::ArrayElement;
use crate::cst::ArrowFunction;
use crate::cst::Autoescape;
use crate::cst::Binary;
use crate::cst::Block;
use crate::cst::BlockAlias;
use crate::cst::Bool;
use crate::cst::Cache;
use crate::cst::CacheOption;
use crate::cst::Call;
use crate::cst::Conditional;
use crate::cst::Deprecated;
use crate::cst::DeprecatedOption;
use crate::cst::Do;
use crate::cst::ElseBranch;
use crate::cst::Embed;
use crate::cst::Expression;
use crate::cst::Extends;
use crate::cst::Filter;
use crate::cst::FilterApplication;
use crate::cst::Flush;
use crate::cst::For;
use crate::cst::ForIfClause;
use crate::cst::From;
use crate::cst::GetAttribute;
use crate::cst::GetItem;
use crate::cst::Guard;
use crate::cst::HashMap;
use crate::cst::HashMapEntry;
use crate::cst::Identifier;
use crate::cst::If;
use crate::cst::IfBranch;
use crate::cst::Import;
use crate::cst::ImportedMacro;
use crate::cst::Include;
use crate::cst::InterpolatedString;
use crate::cst::Interpolation;
use crate::cst::Keyword;
use crate::cst::Macro;
use crate::cst::MacroArgument;
use crate::cst::MethodCall;
use crate::cst::MissingArrayElement;
use crate::cst::Name;
use crate::cst::NamedArgument;
use crate::cst::Null;
use crate::cst::Number;
use crate::cst::Parenthesized;
use crate::cst::PositionalArgument;
use crate::cst::Print;
use crate::cst::Sandbox;
use crate::cst::Set;
use crate::cst::Slice;
use crate::cst::Statement;
use crate::cst::StringLiteral;
use crate::cst::Template;
use crate::cst::Test;
use crate::cst::Text;
use crate::cst::Types;
use crate::cst::Unary;
use crate::cst::Unknown;
use crate::cst::Use;
use crate::cst::ValueArrayElement;
use crate::cst::VariadicArrayElement;
use crate::cst::Verbatim;
use crate::cst::With;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
#[repr(u8)]
#[non_exhaustive]
pub enum NodeKind {
    Template,
    Statement,
    Expression,
    Keyword,
    Identifier,
    Text,
    Print,
    Verbatim,
    If,
    IfBranch,
    ElseBranch,
    For,
    ForIfClause,
    Set,
    Block,
    Extends,
    Use,
    BlockAlias,
    Include,
    Embed,
    Import,
    From,
    ImportedMacro,
    Macro,
    MacroArgument,
    With,
    Apply,
    FilterApplication,
    Autoescape,
    Sandbox,
    Deprecated,
    DeprecatedOption,
    Do,
    Flush,
    Guard,
    Cache,
    CacheOption,
    Types,
    Unknown,
    Name,
    Number,
    StringLiteral,
    InterpolatedString,
    Interpolation,
    Bool,
    Null,
    Array,
    HashMap,
    HashMapEntry,
    Unary,
    Binary,
    Conditional,
    GetAttribute,
    GetItem,
    Slice,
    Call,
    MethodCall,
    Filter,
    Test,
    Parenthesized,
    ArrowFunction,
    ArgumentList,
    Argument,
    PositionalArgument,
    NamedArgument,
    ArrayElement,
    ValueArrayElement,
    VariadicArrayElement,
    MissingArrayElement,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
#[non_exhaustive]
pub enum Node<'ast, 'arena> {
    Template(&'ast Template<'arena>),
    Statement(&'ast Statement<'arena>),
    Expression(&'ast Expression<'arena>),
    Keyword(&'ast Keyword<'arena>),
    Identifier(&'ast Identifier<'arena>),
    Text(&'ast Text<'arena>),
    Print(&'ast Print<'arena>),
    Verbatim(&'ast Verbatim<'arena>),
    If(&'ast If<'arena>),
    IfBranch(&'ast IfBranch<'arena>),
    ElseBranch(&'ast ElseBranch<'arena>),
    For(&'ast For<'arena>),
    ForIfClause(&'ast ForIfClause<'arena>),
    Set(&'ast Set<'arena>),
    Block(&'ast Block<'arena>),
    Extends(&'ast Extends<'arena>),
    Use(&'ast Use<'arena>),
    BlockAlias(&'ast BlockAlias<'arena>),
    Include(&'ast Include<'arena>),
    Embed(&'ast Embed<'arena>),
    Import(&'ast Import<'arena>),
    From(&'ast From<'arena>),
    ImportedMacro(&'ast ImportedMacro<'arena>),
    Macro(&'ast Macro<'arena>),
    MacroArgument(&'ast MacroArgument<'arena>),
    With(&'ast With<'arena>),
    Apply(&'ast Apply<'arena>),
    FilterApplication(&'ast FilterApplication<'arena>),
    Autoescape(&'ast Autoescape<'arena>),
    Sandbox(&'ast Sandbox<'arena>),
    Deprecated(&'ast Deprecated<'arena>),
    DeprecatedOption(&'ast DeprecatedOption<'arena>),
    Do(&'ast Do<'arena>),
    Flush(&'ast Flush<'arena>),
    Guard(&'ast Guard<'arena>),
    Cache(&'ast Cache<'arena>),
    CacheOption(&'ast CacheOption<'arena>),
    Types(&'ast Types<'arena>),
    Unknown(&'ast Unknown<'arena>),
    Name(&'ast Name<'arena>),
    Number(&'ast Number<'arena>),
    StringLiteral(&'ast StringLiteral<'arena>),
    InterpolatedString(&'ast InterpolatedString<'arena>),
    Interpolation(&'ast Interpolation<'arena>),
    Bool(&'ast Bool),
    Null(&'ast Null),
    Array(&'ast crate::cst::Array<'arena>),
    HashMap(&'ast HashMap<'arena>),
    HashMapEntry(&'ast HashMapEntry<'arena>),
    Unary(&'ast Unary<'arena>),
    Binary(&'ast Binary<'arena>),
    Conditional(&'ast Conditional<'arena>),
    GetAttribute(&'ast GetAttribute<'arena>),
    GetItem(&'ast GetItem<'arena>),
    Slice(&'ast Slice<'arena>),
    Call(&'ast Call<'arena>),
    MethodCall(&'ast MethodCall<'arena>),
    Filter(&'ast Filter<'arena>),
    Test(&'ast Test<'arena>),
    Parenthesized(&'ast Parenthesized<'arena>),
    ArrowFunction(&'ast ArrowFunction<'arena>),
    ArgumentList(&'ast ArgumentList<'arena>),
    Argument(&'ast Argument<'arena>),
    PositionalArgument(&'ast PositionalArgument<'arena>),
    NamedArgument(&'ast NamedArgument<'arena>),
    ArrayElement(&'ast ArrayElement<'arena>),
    ValueArrayElement(&'ast ValueArrayElement<'arena>),
    VariadicArrayElement(&'ast VariadicArrayElement<'arena>),
    MissingArrayElement(&'ast MissingArrayElement),
}

impl HasSpan for Node<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Node::Template(n) => n.span(),
            Node::Statement(n) => n.span(),
            Node::Expression(n) => n.span(),
            Node::Keyword(n) => n.span(),
            Node::Identifier(n) => n.span(),
            Node::Text(n) => n.span(),
            Node::Print(n) => n.span(),
            Node::Verbatim(n) => n.span(),
            Node::If(n) => n.span(),
            Node::IfBranch(n) => n.span(),
            Node::ElseBranch(n) => n.span(),
            Node::For(n) => n.span(),
            Node::ForIfClause(n) => n.span(),
            Node::Set(n) => n.span(),
            Node::Block(n) => n.span(),
            Node::Extends(n) => n.span(),
            Node::Use(n) => n.span(),
            Node::BlockAlias(n) => n.span(),
            Node::Include(n) => n.span(),
            Node::Embed(n) => n.span(),
            Node::Import(n) => n.span(),
            Node::From(n) => n.span(),
            Node::ImportedMacro(n) => n.span(),
            Node::Macro(n) => n.span(),
            Node::MacroArgument(n) => n.span(),
            Node::With(n) => n.span(),
            Node::Apply(n) => n.span(),
            Node::FilterApplication(n) => n.span(),
            Node::Autoescape(n) => n.span(),
            Node::Sandbox(n) => n.span(),
            Node::Deprecated(n) => n.span(),
            Node::DeprecatedOption(n) => n.span(),
            Node::Do(n) => n.span(),
            Node::Flush(n) => n.span(),
            Node::Guard(n) => n.span(),
            Node::Cache(n) => n.span(),
            Node::CacheOption(n) => n.span(),
            Node::Types(n) => n.span(),
            Node::Unknown(n) => n.span(),
            Node::Name(n) => n.span(),
            Node::Number(n) => n.span(),
            Node::StringLiteral(n) => n.span(),
            Node::InterpolatedString(n) => n.span(),
            Node::Interpolation(n) => n.span(),
            Node::Bool(n) => n.span(),
            Node::Null(n) => n.span(),
            Node::Array(n) => n.span(),
            Node::HashMap(n) => n.span(),
            Node::HashMapEntry(n) => n.span(),
            Node::Unary(n) => n.span(),
            Node::Binary(n) => n.span(),
            Node::Conditional(n) => n.span(),
            Node::GetAttribute(n) => n.span(),
            Node::GetItem(n) => n.span(),
            Node::Slice(n) => n.span(),
            Node::Call(n) => n.span(),
            Node::MethodCall(n) => n.span(),
            Node::Filter(n) => n.span(),
            Node::Test(n) => n.span(),
            Node::Parenthesized(n) => n.span(),
            Node::ArrowFunction(n) => n.span(),
            Node::ArgumentList(n) => n.span(),
            Node::Argument(n) => n.span(),
            Node::PositionalArgument(n) => n.span(),
            Node::NamedArgument(n) => n.span(),
            Node::ArrayElement(n) => n.span(),
            Node::ValueArrayElement(n) => n.span(),
            Node::VariadicArrayElement(n) => n.span(),
            Node::MissingArrayElement(n) => n.span(),
        }
    }
}
