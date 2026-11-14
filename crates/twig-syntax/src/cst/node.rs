//! Universal walker enum over every Twig CST node.

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
use crate::cst::BlockBody;
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
use crate::cst::SetBody;
use crate::cst::Slice;
use crate::cst::Statement;
use crate::cst::StringConcatenation;
use crate::cst::StringLiteral;
use crate::cst::StringPart;
use crate::cst::Template;
use crate::cst::Test;
use crate::cst::TestArguments;
use crate::cst::Text;
use crate::cst::TypeDeclaration;
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
    TypeDeclaration,
    Unknown,
    Name,
    Number,
    StringLiteral,
    InterpolatedString,
    StringConcatenation,
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
    TypeDeclaration(&'ast TypeDeclaration<'arena>),
    Unknown(&'ast Unknown<'arena>),
    Name(&'ast Name<'arena>),
    Number(&'ast Number<'arena>),
    StringLiteral(&'ast StringLiteral<'arena>),
    InterpolatedString(&'ast InterpolatedString<'arena>),
    StringConcatenation(&'ast StringConcatenation<'arena>),
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

impl<'ast, 'arena> Node<'ast, 'arena> {
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        match self {
            Node::Template(_) => NodeKind::Template,
            Node::Statement(_) => NodeKind::Statement,
            Node::Expression(_) => NodeKind::Expression,
            Node::Keyword(_) => NodeKind::Keyword,
            Node::Identifier(_) => NodeKind::Identifier,
            Node::Text(_) => NodeKind::Text,
            Node::Print(_) => NodeKind::Print,
            Node::Verbatim(_) => NodeKind::Verbatim,
            Node::If(_) => NodeKind::If,
            Node::IfBranch(_) => NodeKind::IfBranch,
            Node::ElseBranch(_) => NodeKind::ElseBranch,
            Node::For(_) => NodeKind::For,
            Node::ForIfClause(_) => NodeKind::ForIfClause,
            Node::Set(_) => NodeKind::Set,
            Node::Block(_) => NodeKind::Block,
            Node::Extends(_) => NodeKind::Extends,
            Node::Use(_) => NodeKind::Use,
            Node::BlockAlias(_) => NodeKind::BlockAlias,
            Node::Include(_) => NodeKind::Include,
            Node::Embed(_) => NodeKind::Embed,
            Node::Import(_) => NodeKind::Import,
            Node::From(_) => NodeKind::From,
            Node::ImportedMacro(_) => NodeKind::ImportedMacro,
            Node::Macro(_) => NodeKind::Macro,
            Node::MacroArgument(_) => NodeKind::MacroArgument,
            Node::With(_) => NodeKind::With,
            Node::Apply(_) => NodeKind::Apply,
            Node::FilterApplication(_) => NodeKind::FilterApplication,
            Node::Autoescape(_) => NodeKind::Autoescape,
            Node::Sandbox(_) => NodeKind::Sandbox,
            Node::Deprecated(_) => NodeKind::Deprecated,
            Node::DeprecatedOption(_) => NodeKind::DeprecatedOption,
            Node::Do(_) => NodeKind::Do,
            Node::Flush(_) => NodeKind::Flush,
            Node::Guard(_) => NodeKind::Guard,
            Node::Cache(_) => NodeKind::Cache,
            Node::CacheOption(_) => NodeKind::CacheOption,
            Node::Types(_) => NodeKind::Types,
            Node::TypeDeclaration(_) => NodeKind::TypeDeclaration,
            Node::Unknown(_) => NodeKind::Unknown,
            Node::Name(_) => NodeKind::Name,
            Node::Number(_) => NodeKind::Number,
            Node::StringLiteral(_) => NodeKind::StringLiteral,
            Node::InterpolatedString(_) => NodeKind::InterpolatedString,
            Node::StringConcatenation(_) => NodeKind::StringConcatenation,
            Node::Interpolation(_) => NodeKind::Interpolation,
            Node::Bool(_) => NodeKind::Bool,
            Node::Null(_) => NodeKind::Null,
            Node::Array(_) => NodeKind::Array,
            Node::HashMap(_) => NodeKind::HashMap,
            Node::HashMapEntry(_) => NodeKind::HashMapEntry,
            Node::Unary(_) => NodeKind::Unary,
            Node::Binary(_) => NodeKind::Binary,
            Node::Conditional(_) => NodeKind::Conditional,
            Node::GetAttribute(_) => NodeKind::GetAttribute,
            Node::GetItem(_) => NodeKind::GetItem,
            Node::Slice(_) => NodeKind::Slice,
            Node::Call(_) => NodeKind::Call,
            Node::MethodCall(_) => NodeKind::MethodCall,
            Node::Filter(_) => NodeKind::Filter,
            Node::Test(_) => NodeKind::Test,
            Node::Parenthesized(_) => NodeKind::Parenthesized,
            Node::ArrowFunction(_) => NodeKind::ArrowFunction,
            Node::ArgumentList(_) => NodeKind::ArgumentList,
            Node::Argument(_) => NodeKind::Argument,
            Node::PositionalArgument(_) => NodeKind::PositionalArgument,
            Node::NamedArgument(_) => NodeKind::NamedArgument,
            Node::ArrayElement(_) => NodeKind::ArrayElement,
            Node::ValueArrayElement(_) => NodeKind::ValueArrayElement,
            Node::VariadicArrayElement(_) => NodeKind::VariadicArrayElement,
            Node::MissingArrayElement(_) => NodeKind::MissingArrayElement,
        }
    }

    /// Calls `visit` for each direct child, in source order.
    pub fn visit_children(&self, mut visit: impl FnMut(Node<'ast, 'arena>)) {
        match *self {
            Node::Template(template) => template.statements.iter().for_each(|s| visit(Node::Statement(s))),
            Node::Statement(statement) => visit(match statement {
                Statement::Text(n) => Node::Text(n),
                Statement::Print(n) => Node::Print(n),
                Statement::Verbatim(n) => Node::Verbatim(n),
                Statement::If(n) => Node::If(n),
                Statement::For(n) => Node::For(n),
                Statement::Set(n) => Node::Set(n),
                Statement::Block(n) => Node::Block(n),
                Statement::Extends(n) => Node::Extends(n),
                Statement::Use(n) => Node::Use(n),
                Statement::Include(n) => Node::Include(n),
                Statement::Embed(n) => Node::Embed(n),
                Statement::Import(n) => Node::Import(n),
                Statement::From(n) => Node::From(n),
                Statement::Macro(n) => Node::Macro(n),
                Statement::With(n) => Node::With(n),
                Statement::Apply(n) => Node::Apply(n),
                Statement::Autoescape(n) => Node::Autoescape(n),
                Statement::Sandbox(n) => Node::Sandbox(n),
                Statement::Deprecated(n) => Node::Deprecated(n),
                Statement::Do(n) => Node::Do(n),
                Statement::Flush(n) => Node::Flush(n),
                Statement::Guard(n) => Node::Guard(n),
                Statement::Cache(n) => Node::Cache(n),
                Statement::Types(n) => Node::Types(n),
                Statement::Unknown(n) => Node::Unknown(n),
            }),
            Node::Expression(expression) => visit(match expression {
                Expression::Name(n) => Node::Name(n),
                Expression::Number(n) => Node::Number(n),
                Expression::String(n) => Node::StringLiteral(n),
                Expression::InterpolatedString(n) => Node::InterpolatedString(n),
                Expression::StringConcatenation(n) => Node::StringConcatenation(n),
                Expression::Bool(n) => Node::Bool(n),
                Expression::Null(n) => Node::Null(n),
                Expression::Array(n) => Node::Array(n),
                Expression::HashMap(n) => Node::HashMap(n),
                Expression::Unary(n) => Node::Unary(n),
                Expression::Binary(n) => Node::Binary(n),
                Expression::Conditional(n) => Node::Conditional(n),
                Expression::GetAttribute(n) => Node::GetAttribute(n),
                Expression::GetItem(n) => Node::GetItem(n),
                Expression::Slice(n) => Node::Slice(n),
                Expression::Call(n) => Node::Call(n),
                Expression::MethodCall(n) => Node::MethodCall(n),
                Expression::Filter(n) => Node::Filter(n),
                Expression::Test(n) => Node::Test(n),
                Expression::Parenthesized(n) => Node::Parenthesized(n),
                Expression::ArrowFunction(n) => Node::ArrowFunction(n),
            }),
            Node::Keyword(_)
            | Node::Identifier(_)
            | Node::Text(_)
            | Node::Verbatim(_)
            | Node::BlockAlias(_)
            | Node::ImportedMacro(_)
            | Node::Flush(_)
            | Node::Unknown(_)
            | Node::Name(_)
            | Node::Number(_)
            | Node::StringLiteral(_)
            | Node::Bool(_)
            | Node::Null(_)
            | Node::MissingArrayElement(_) => {}
            Node::Print(print) => visit(Node::Expression(&print.expression)),
            Node::If(r#if) => {
                r#if.branches.iter().for_each(|branch| visit(Node::IfBranch(branch)));
                if let Some(branch) = &r#if.else_branch {
                    visit(Node::ElseBranch(branch));
                }
            }
            Node::IfBranch(branch) => {
                visit(Node::Expression(&branch.condition));
                branch.body.iter().for_each(|s| visit(Node::Statement(s)));
            }
            Node::ElseBranch(branch) => branch.body.iter().for_each(|s| visit(Node::Statement(s))),
            Node::For(r#for) => {
                visit(Node::Expression(&r#for.sequence));
                if let Some(clause) = &r#for.if_clause {
                    visit(Node::ForIfClause(clause));
                }
                r#for.body.iter().for_each(|s| visit(Node::Statement(s)));
                if let Some(branch) = &r#for.else_branch {
                    visit(Node::ElseBranch(branch));
                }
            }
            Node::ForIfClause(clause) => visit(Node::Expression(&clause.condition)),
            Node::Set(set) => match &set.body {
                SetBody::Inline(inline) => inline.values.iter().for_each(|e| visit(Node::Expression(e))),
                SetBody::Capture(capture) => capture.body.iter().for_each(|s| visit(Node::Statement(s))),
            },
            Node::Block(block) => match &block.body {
                BlockBody::Short(short) => visit(Node::Expression(&short.expression)),
                BlockBody::Long(long) => long.body.iter().for_each(|s| visit(Node::Statement(s))),
            },
            Node::Extends(extends) => visit(Node::Expression(&extends.template)),
            Node::Use(r#use) => {
                visit(Node::Expression(&r#use.template));
                r#use.aliases.iter().for_each(|alias| visit(Node::BlockAlias(alias)));
            }
            Node::Include(include) => {
                visit(Node::Expression(&include.template));
                if let Some(clause) = &include.with_clause {
                    visit(Node::Expression(&clause.variables));
                }
            }
            Node::Embed(embed) => {
                visit(Node::Expression(&embed.template));
                if let Some(clause) = &embed.with_clause {
                    visit(Node::Expression(&clause.variables));
                }
                embed.body.iter().for_each(|s| visit(Node::Statement(s)));
            }
            Node::Import(import) => visit(Node::Expression(&import.template)),
            Node::From(from) => {
                visit(Node::Expression(&from.template));
                from.names.iter().for_each(|name| visit(Node::ImportedMacro(name)));
            }
            Node::Macro(r#macro) => {
                r#macro.arguments.iter().for_each(|argument| visit(Node::MacroArgument(argument)));
                r#macro.body.iter().for_each(|s| visit(Node::Statement(s)));
            }
            Node::MacroArgument(argument) => {
                if let Some(default) = &argument.default {
                    visit(Node::Expression(default));
                }
            }
            Node::With(with) => {
                if let Some(variables) = &with.variables {
                    visit(Node::Expression(variables));
                }
                with.body.iter().for_each(|s| visit(Node::Statement(s)));
            }
            Node::Apply(apply) => {
                apply.filters.iter().for_each(|filter| visit(Node::FilterApplication(filter)));
                apply.body.iter().for_each(|s| visit(Node::Statement(s)));
            }
            Node::FilterApplication(filter) => {
                if let Some(list) = &filter.argument_list {
                    visit(Node::ArgumentList(list));
                }
            }
            Node::Autoescape(autoescape) => {
                if let Some(strategy) = &autoescape.strategy {
                    visit(Node::Expression(strategy));
                }
                autoescape.body.iter().for_each(|s| visit(Node::Statement(s)));
            }
            Node::Sandbox(sandbox) => sandbox.body.iter().for_each(|s| visit(Node::Statement(s))),
            Node::Deprecated(deprecated) => {
                visit(Node::Expression(&deprecated.message));
                deprecated.options.iter().for_each(|option| visit(Node::DeprecatedOption(option)));
            }
            Node::DeprecatedOption(option) => visit(Node::Expression(&option.value)),
            Node::Do(r#do) => visit(Node::Expression(&r#do.expression)),
            Node::Guard(guard) => {
                guard.body.iter().for_each(|s| visit(Node::Statement(s)));
                if let Some(branch) = &guard.else_branch {
                    visit(Node::ElseBranch(branch));
                }
            }
            Node::Cache(cache) => {
                visit(Node::Expression(&cache.key));
                cache.options().for_each(|option| visit(Node::CacheOption(option)));
                cache.body.iter().for_each(|s| visit(Node::Statement(s)));
            }
            Node::CacheOption(option) => visit(Node::Expression(&option.value)),
            Node::Types(types) => types.declarations.iter().for_each(|d| visit(Node::TypeDeclaration(d))),
            Node::TypeDeclaration(declaration) => {
                visit(Node::Identifier(&declaration.name));
                visit(Node::StringLiteral(&declaration.value));
            }
            Node::InterpolatedString(string) => {
                for part in string.parts {
                    if let StringPart::Interpolation(interpolation) = part {
                        visit(Node::Interpolation(interpolation));
                    }
                }
            }
            Node::StringConcatenation(concatenation) => {
                visit(Node::Expression(concatenation.lhs));
                visit(Node::Expression(concatenation.rhs));
            }
            Node::Interpolation(interpolation) => visit(Node::Expression(interpolation.expression)),
            Node::Array(array) => array.elements.iter().for_each(|element| visit(Node::ArrayElement(element))),
            Node::ArrayElement(element) => visit(match element {
                ArrayElement::Value(e) => Node::ValueArrayElement(e),
                ArrayElement::Variadic(e) => Node::VariadicArrayElement(e),
                ArrayElement::Missing(e) => Node::MissingArrayElement(e),
            }),
            Node::ValueArrayElement(element) => visit(Node::Expression(element.value)),
            Node::VariadicArrayElement(element) => visit(Node::Expression(element.value)),
            Node::HashMap(map) => map.entries.iter().for_each(|entry| visit(Node::HashMapEntry(entry))),
            Node::HashMapEntry(entry) => {
                if let Some(key) = &entry.key {
                    visit(Node::Expression(key));
                }
                visit(Node::Expression(&entry.value));
            }
            Node::Unary(unary) => visit(Node::Expression(unary.operand)),
            Node::Binary(binary) => {
                visit(Node::Expression(binary.lhs));
                visit(Node::Expression(binary.rhs));
            }
            Node::Conditional(conditional) => {
                visit(Node::Expression(conditional.condition));
                if let Some(then) = conditional.then {
                    visit(Node::Expression(then));
                }
                if let Some(r#else) = conditional.r#else {
                    visit(Node::Expression(r#else));
                }
            }
            Node::GetAttribute(access) => {
                visit(Node::Expression(access.object));
                visit(Node::Expression(access.attribute));
            }
            Node::GetItem(access) => {
                visit(Node::Expression(access.object));
                visit(Node::Expression(access.index));
            }
            Node::Slice(slice) => {
                visit(Node::Expression(slice.object));
                if let Some(start) = slice.start {
                    visit(Node::Expression(start));
                }
                if let Some(length) = slice.length {
                    visit(Node::Expression(length));
                }
            }
            Node::Call(call) => {
                visit(Node::Expression(call.callee));
                visit(Node::ArgumentList(&call.argument_list));
            }
            Node::MethodCall(call) => {
                visit(Node::Expression(call.object));
                visit(Node::ArgumentList(&call.argument_list));
            }
            Node::Filter(filter) => {
                visit(Node::Expression(filter.operand));
                if let Some(list) = &filter.argument_list {
                    visit(Node::ArgumentList(list));
                }
            }
            Node::Test(test) => {
                visit(Node::Expression(test.operand));
                match &test.arguments {
                    TestArguments::Parenthesised(list) => visit(Node::ArgumentList(list)),
                    TestArguments::Bare(e) => visit(Node::Expression(e)),
                    TestArguments::None => {}
                }
            }
            Node::Parenthesized(parenthesized) => visit(Node::Expression(parenthesized.inner)),
            Node::ArrowFunction(function) => visit(Node::Expression(function.body)),
            Node::ArgumentList(list) => list.arguments.iter().for_each(|argument| visit(Node::Argument(argument))),
            Node::Argument(argument) => visit(match argument {
                Argument::Positional(a) => Node::PositionalArgument(a),
                Argument::Named(a) => Node::NamedArgument(a),
            }),
            Node::PositionalArgument(argument) => visit(Node::Expression(argument.value)),
            Node::NamedArgument(argument) => visit(Node::Expression(argument.value)),
        }
    }
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
            Node::TypeDeclaration(n) => n.span(),
            Node::Unknown(n) => n.span(),
            Node::Name(n) => n.span(),
            Node::Number(n) => n.span(),
            Node::StringLiteral(n) => n.span(),
            Node::InterpolatedString(n) => n.span(),
            Node::StringConcatenation(n) => n.span(),
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
