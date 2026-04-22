//! Universal walker enum over every Twig AST node.

use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Apply;
use crate::ast::Argument;
use crate::ast::ArgumentList;
use crate::ast::ArrayElement;
use crate::ast::ArrowFunction;
use crate::ast::Autoescape;
use crate::ast::Binary;
use crate::ast::Block;
use crate::ast::BlockAlias;
use crate::ast::Bool;
use crate::ast::Cache;
use crate::ast::CacheOption;
use crate::ast::Call;
use crate::ast::Conditional;
use crate::ast::Deprecated;
use crate::ast::DeprecatedOption;
use crate::ast::Do;
use crate::ast::ElseBranch;
use crate::ast::Embed;
use crate::ast::Expression;
use crate::ast::Extends;
use crate::ast::Filter;
use crate::ast::FilterApplication;
use crate::ast::Flush;
use crate::ast::For;
use crate::ast::ForIfClause;
use crate::ast::From;
use crate::ast::GetAttribute;
use crate::ast::GetItem;
use crate::ast::Guard;
use crate::ast::HashMap;
use crate::ast::HashMapEntry;
use crate::ast::Identifier;
use crate::ast::If;
use crate::ast::IfBranch;
use crate::ast::Import;
use crate::ast::ImportedMacro;
use crate::ast::Include;
use crate::ast::InterpolatedString;
use crate::ast::Interpolation;
use crate::ast::Keyword;
use crate::ast::Macro;
use crate::ast::MacroArgument;
use crate::ast::MethodCall;
use crate::ast::MissingArrayElement;
use crate::ast::Name;
use crate::ast::NamedArgument;
use crate::ast::Null;
use crate::ast::Number;
use crate::ast::Parenthesized;
use crate::ast::PositionalArgument;
use crate::ast::Print;
use crate::ast::Sandbox;
use crate::ast::Set;
use crate::ast::Slice;
use crate::ast::Statement;
use crate::ast::StringLiteral;
use crate::ast::Template;
use crate::ast::Test;
use crate::ast::Text;
use crate::ast::Types;
use crate::ast::Unary;
use crate::ast::Unknown;
use crate::ast::Use;
use crate::ast::ValueArrayElement;
use crate::ast::VariadicArrayElement;
use crate::ast::Verbatim;
use crate::ast::With;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
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
    Array(&'ast crate::ast::Array<'arena>),
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
