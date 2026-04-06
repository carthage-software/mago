use std::fmt::Debug;

use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Program;
use crate::ast::ast::Access;
use crate::ast::ast::AnonymousClass;
use crate::ast::ast::Argument;
use crate::ast::ast::ArgumentList;
use crate::ast::ast::Array;
use crate::ast::ast::ArrayAccess;
use crate::ast::ast::ArrayAppend;
use crate::ast::ast::ArrayElement;
use crate::ast::ast::ArrowFunction;
use crate::ast::ast::Assignment;
use crate::ast::ast::AssignmentOperator;
use crate::ast::ast::Attribute;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Binary;
use crate::ast::ast::BinaryOperator;
use crate::ast::ast::Block;
use crate::ast::ast::BracedExpressionStringPart;
use crate::ast::ast::Break;
use crate::ast::ast::Call;
use crate::ast::ast::Class;
use crate::ast::ast::ClassConstantAccess;
use crate::ast::ast::ClassLikeConstant;
use crate::ast::ast::ClassLikeConstantItem;
use crate::ast::ast::ClassLikeConstantSelector;
use crate::ast::ast::ClassLikeMember;
use crate::ast::ast::ClassLikeMemberExpressionSelector;
use crate::ast::ast::ClassLikeMemberSelector;
use crate::ast::ast::Clone;
use crate::ast::ast::ClosingTag;
use crate::ast::ast::Closure;
use crate::ast::ast::ClosureUseClause;
use crate::ast::ast::ClosureUseClauseVariable;
use crate::ast::ast::CompositeString;
use crate::ast::ast::Conditional;
use crate::ast::ast::Constant;
use crate::ast::ast::ConstantAccess;
use crate::ast::ast::ConstantItem;
use crate::ast::ast::Construct;
use crate::ast::ast::Continue;
use crate::ast::ast::Declare;
use crate::ast::ast::DeclareBody;
use crate::ast::ast::DeclareColonDelimitedBody;
use crate::ast::ast::DeclareItem;
use crate::ast::ast::DieConstruct;
use crate::ast::ast::DirectVariable;
use crate::ast::ast::DoWhile;
use crate::ast::ast::DocumentString;
use crate::ast::ast::Echo;
use crate::ast::ast::EchoTag;
use crate::ast::ast::EmptyConstruct;
use crate::ast::ast::Enum;
use crate::ast::ast::EnumBackingTypeHint;
use crate::ast::ast::EnumCase;
use crate::ast::ast::EnumCaseBackedItem;
use crate::ast::ast::EnumCaseItem;
use crate::ast::ast::EnumCaseUnitItem;
use crate::ast::ast::EvalConstruct;
use crate::ast::ast::ExitConstruct;
use crate::ast::ast::Expression;
use crate::ast::ast::ExpressionStatement;
use crate::ast::ast::Extends;
use crate::ast::ast::For;
use crate::ast::ast::ForBody;
use crate::ast::ast::ForColonDelimitedBody;
use crate::ast::ast::Foreach;
use crate::ast::ast::ForeachBody;
use crate::ast::ast::ForeachColonDelimitedBody;
use crate::ast::ast::ForeachKeyValueTarget;
use crate::ast::ast::ForeachTarget;
use crate::ast::ast::ForeachValueTarget;
use crate::ast::ast::FullOpeningTag;
use crate::ast::ast::FullyQualifiedIdentifier;
use crate::ast::ast::Function;
use crate::ast::ast::FunctionCall;
use crate::ast::ast::FunctionLikeParameter;
use crate::ast::ast::FunctionLikeParameterDefaultValue;
use crate::ast::ast::FunctionLikeParameterList;
use crate::ast::ast::FunctionLikeReturnTypeHint;
use crate::ast::ast::FunctionPartialApplication;
use crate::ast::ast::Global;
use crate::ast::ast::Goto;
use crate::ast::ast::HaltCompiler;
use crate::ast::ast::Hint;
use crate::ast::ast::HookedProperty;
use crate::ast::ast::Identifier;
use crate::ast::ast::If;
use crate::ast::ast::IfBody;
use crate::ast::ast::IfColonDelimitedBody;
use crate::ast::ast::IfColonDelimitedBodyElseClause;
use crate::ast::ast::IfColonDelimitedBodyElseIfClause;
use crate::ast::ast::IfStatementBody;
use crate::ast::ast::IfStatementBodyElseClause;
use crate::ast::ast::IfStatementBodyElseIfClause;
use crate::ast::ast::Implements;
use crate::ast::ast::IncludeConstruct;
use crate::ast::ast::IncludeOnceConstruct;
use crate::ast::ast::IndirectVariable;
use crate::ast::ast::Inline;
use crate::ast::ast::Instantiation;
use crate::ast::ast::Interface;
use crate::ast::ast::InterpolatedString;
use crate::ast::ast::IntersectionHint;
use crate::ast::ast::IssetConstruct;
use crate::ast::ast::KeyValueArrayElement;
use crate::ast::ast::Keyword;
use crate::ast::ast::Label;
use crate::ast::ast::LegacyArray;
use crate::ast::ast::List;
use crate::ast::ast::Literal;
use crate::ast::ast::LiteralFloat;
use crate::ast::ast::LiteralInteger;
use crate::ast::ast::LiteralString;
use crate::ast::ast::LiteralStringPart;
use crate::ast::ast::LocalIdentifier;
use crate::ast::ast::MagicConstant;
use crate::ast::ast::Match;
use crate::ast::ast::MatchArm;
use crate::ast::ast::MatchDefaultArm;
use crate::ast::ast::MatchExpressionArm;
use crate::ast::ast::MaybeTypedUseItem;
use crate::ast::ast::Method;
use crate::ast::ast::MethodAbstractBody;
use crate::ast::ast::MethodBody;
use crate::ast::ast::MethodCall;
use crate::ast::ast::MethodPartialApplication;
use crate::ast::ast::MissingArrayElement;
use crate::ast::ast::MixedUseItemList;
use crate::ast::ast::Modifier;
use crate::ast::ast::NamedArgument;
use crate::ast::ast::NamedPlaceholderArgument;
use crate::ast::ast::Namespace;
use crate::ast::ast::NamespaceBody;
use crate::ast::ast::NamespaceImplicitBody;
use crate::ast::ast::NestedVariable;
use crate::ast::ast::NullSafeMethodCall;
use crate::ast::ast::NullSafePropertyAccess;
use crate::ast::ast::NullableHint;
use crate::ast::ast::OpeningTag;
use crate::ast::ast::Parenthesized;
use crate::ast::ast::ParenthesizedHint;
use crate::ast::ast::PartialApplication;
use crate::ast::ast::PartialArgument;
use crate::ast::ast::PartialArgumentList;
use crate::ast::ast::Pipe;
use crate::ast::ast::PlaceholderArgument;
use crate::ast::ast::PlainProperty;
use crate::ast::ast::PositionalArgument;
use crate::ast::ast::PrintConstruct;
use crate::ast::ast::Property;
use crate::ast::ast::PropertyAbstractItem;
use crate::ast::ast::PropertyAccess;
use crate::ast::ast::PropertyConcreteItem;
use crate::ast::ast::PropertyHook;
use crate::ast::ast::PropertyHookAbstractBody;
use crate::ast::ast::PropertyHookBody;
use crate::ast::ast::PropertyHookConcreteBody;
use crate::ast::ast::PropertyHookConcreteExpressionBody;
use crate::ast::ast::PropertyHookList;
use crate::ast::ast::PropertyItem;
use crate::ast::ast::QualifiedIdentifier;
use crate::ast::ast::RequireConstruct;
use crate::ast::ast::RequireOnceConstruct;
use crate::ast::ast::Return;
use crate::ast::ast::ShellExecuteString;
use crate::ast::ast::ShortOpeningTag;
use crate::ast::ast::Statement;
use crate::ast::ast::Static;
use crate::ast::ast::StaticAbstractItem;
use crate::ast::ast::StaticConcreteItem;
use crate::ast::ast::StaticItem;
use crate::ast::ast::StaticMethodCall;
use crate::ast::ast::StaticMethodPartialApplication;
use crate::ast::ast::StaticPropertyAccess;
use crate::ast::ast::StringPart;
use crate::ast::ast::Switch;
use crate::ast::ast::SwitchBody;
use crate::ast::ast::SwitchBraceDelimitedBody;
use crate::ast::ast::SwitchCase;
use crate::ast::ast::SwitchCaseSeparator;
use crate::ast::ast::SwitchColonDelimitedBody;
use crate::ast::ast::SwitchDefaultCase;
use crate::ast::ast::SwitchExpressionCase;
use crate::ast::ast::Terminator;
use crate::ast::ast::Throw;
use crate::ast::ast::Trait;
use crate::ast::ast::TraitUse;
use crate::ast::ast::TraitUseAbsoluteMethodReference;
use crate::ast::ast::TraitUseAbstractSpecification;
use crate::ast::ast::TraitUseAdaptation;
use crate::ast::ast::TraitUseAliasAdaptation;
use crate::ast::ast::TraitUseConcreteSpecification;
use crate::ast::ast::TraitUseMethodReference;
use crate::ast::ast::TraitUsePrecedenceAdaptation;
use crate::ast::ast::TraitUseSpecification;
use crate::ast::ast::Try;
use crate::ast::ast::TryCatchClause;
use crate::ast::ast::TryFinallyClause;
use crate::ast::ast::TypedUseItemList;
use crate::ast::ast::TypedUseItemSequence;
use crate::ast::ast::UnaryPostfix;
use crate::ast::ast::UnaryPostfixOperator;
use crate::ast::ast::UnaryPrefix;
use crate::ast::ast::UnaryPrefixOperator;
use crate::ast::ast::UnionHint;
use crate::ast::ast::Unset;
use crate::ast::ast::Use;
use crate::ast::ast::UseItem;
use crate::ast::ast::UseItemAlias;
use crate::ast::ast::UseItemSequence;
use crate::ast::ast::UseItems;
use crate::ast::ast::UseType;
use crate::ast::ast::ValueArrayElement;
use crate::ast::ast::Variable;
use crate::ast::ast::VariadicArrayElement;
use crate::ast::ast::VariadicPlaceholderArgument;
use crate::ast::ast::While;
use crate::ast::ast::WhileBody;
use crate::ast::ast::WhileColonDelimitedBody;
use crate::ast::ast::Yield;
use crate::ast::ast::YieldFrom;
use crate::ast::ast::YieldPair;
use crate::ast::ast::YieldValue;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
#[non_exhaustive]
pub enum NodeKind {
    Program,
    ConstantAccess,
    Access,
    ClassConstantAccess,
    NullSafePropertyAccess,
    PropertyAccess,
    StaticPropertyAccess,
    Argument,
    ArgumentList,
    PartialArgument,
    PartialArgumentList,
    NamedArgument,
    NamedPlaceholderArgument,
    PlaceholderArgument,
    PositionalArgument,
    VariadicPlaceholderArgument,
    Array,
    ArrayAccess,
    ArrayAppend,
    ArrayElement,
    KeyValueArrayElement,
    LegacyArray,
    List,
    MissingArrayElement,
    ValueArrayElement,
    VariadicArrayElement,
    Attribute,
    AttributeList,
    Block,
    Call,
    FunctionCall,
    MethodCall,
    NullSafeMethodCall,
    StaticMethodCall,
    PartialApplication,
    FunctionPartialApplication,
    MethodPartialApplication,
    StaticMethodPartialApplication,
    ClassLikeConstant,
    ClassLikeConstantItem,
    EnumCase,
    EnumCaseBackedItem,
    EnumCaseItem,
    EnumCaseUnitItem,
    Extends,
    Implements,
    ClassLikeConstantSelector,
    ClassLikeMember,
    ClassLikeMemberExpressionSelector,
    ClassLikeMemberSelector,
    Method,
    MethodAbstractBody,
    MethodBody,
    HookedProperty,
    PlainProperty,
    Property,
    PropertyAbstractItem,
    PropertyConcreteItem,
    PropertyHook,
    PropertyHookAbstractBody,
    PropertyHookBody,
    PropertyHookConcreteBody,
    PropertyHookConcreteExpressionBody,
    PropertyHookList,
    PropertyItem,
    TraitUse,
    TraitUseAbsoluteMethodReference,
    TraitUseAbstractSpecification,
    TraitUseAdaptation,
    TraitUseAliasAdaptation,
    TraitUseConcreteSpecification,
    TraitUseMethodReference,
    TraitUsePrecedenceAdaptation,
    TraitUseSpecification,
    AnonymousClass,
    Class,
    Enum,
    EnumBackingTypeHint,
    Interface,
    Trait,
    Clone,
    Constant,
    ConstantItem,
    Construct,
    DieConstruct,
    EmptyConstruct,
    EvalConstruct,
    ExitConstruct,
    IncludeConstruct,
    IncludeOnceConstruct,
    IssetConstruct,
    PrintConstruct,
    RequireConstruct,
    RequireOnceConstruct,
    If,
    IfBody,
    IfColonDelimitedBody,
    IfColonDelimitedBodyElseClause,
    IfColonDelimitedBodyElseIfClause,
    IfStatementBody,
    IfStatementBodyElseClause,
    IfStatementBodyElseIfClause,
    Match,
    MatchArm,
    MatchDefaultArm,
    MatchExpressionArm,
    Switch,
    SwitchBody,
    SwitchBraceDelimitedBody,
    SwitchCase,
    SwitchCaseSeparator,
    SwitchColonDelimitedBody,
    SwitchDefaultCase,
    SwitchExpressionCase,
    Declare,
    DeclareBody,
    DeclareColonDelimitedBody,
    DeclareItem,
    EchoTag,
    Echo,
    Expression,
    Binary,
    BinaryOperator,
    UnaryPrefix,
    UnaryPrefixOperator,
    UnaryPostfix,
    UnaryPostfixOperator,
    Parenthesized,
    ArrowFunction,
    Closure,
    ClosureUseClause,
    ClosureUseClauseVariable,
    Function,
    FunctionLikeParameter,
    FunctionLikeParameterDefaultValue,
    FunctionLikeParameterList,
    FunctionLikeReturnTypeHint,
    Global,
    Goto,
    Label,
    HaltCompiler,
    FullyQualifiedIdentifier,
    Identifier,
    LocalIdentifier,
    QualifiedIdentifier,
    Inline,
    Instantiation,
    Keyword,
    Literal,
    Pipe,
    LiteralFloat,
    LiteralInteger,
    LiteralString,
    MagicConstant,
    Modifier,
    Namespace,
    NamespaceBody,
    NamespaceImplicitBody,
    Assignment,
    AssignmentOperator,
    Conditional,
    DoWhile,
    Foreach,
    ForeachBody,
    ForeachColonDelimitedBody,
    ForeachKeyValueTarget,
    ForeachTarget,
    ForeachValueTarget,
    For,
    ForBody,
    ForColonDelimitedBody,
    While,
    WhileBody,
    WhileColonDelimitedBody,
    Break,
    Continue,
    Return,
    Static,
    StaticAbstractItem,
    StaticConcreteItem,
    StaticItem,
    Try,
    TryCatchClause,
    TryFinallyClause,
    MaybeTypedUseItem,
    MixedUseItemList,
    TypedUseItemList,
    TypedUseItemSequence,
    Use,
    UseItem,
    UseItemAlias,
    UseItemSequence,
    UseItems,
    UseType,
    Yield,
    YieldFrom,
    YieldPair,
    YieldValue,
    Statement,
    ExpressionStatement,
    BracedExpressionStringPart,
    DocumentString,
    InterpolatedString,
    LiteralStringPart,
    ShellExecuteString,
    CompositeString,
    StringPart,
    ClosingTag,
    FullOpeningTag,
    OpeningTag,
    ShortOpeningTag,
    Terminator,
    Throw,
    Hint,
    IntersectionHint,
    NullableHint,
    ParenthesizedHint,
    UnionHint,
    Unset,
    DirectVariable,
    IndirectVariable,
    NestedVariable,
    Variable,
    Error,
    MissingTerminator,
    ClassLikeMemberMissingSelector,
    ClassLikeConstantMissingSelector,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
#[non_exhaustive]
pub enum Node<'ast, 'arena> {
    Program(&'ast Program<'arena>),
    Access(&'ast Access<'arena>),
    ConstantAccess(&'ast ConstantAccess<'arena>),
    ClassConstantAccess(&'ast ClassConstantAccess<'arena>),
    NullSafePropertyAccess(&'ast NullSafePropertyAccess<'arena>),
    PropertyAccess(&'ast PropertyAccess<'arena>),
    StaticPropertyAccess(&'ast StaticPropertyAccess<'arena>),
    Argument(&'ast Argument<'arena>),
    ArgumentList(&'ast ArgumentList<'arena>),
    PartialArgument(&'ast PartialArgument<'arena>),
    PartialArgumentList(&'ast PartialArgumentList<'arena>),
    NamedArgument(&'ast NamedArgument<'arena>),
    NamedPlaceholderArgument(&'ast NamedPlaceholderArgument<'arena>),
    PlaceholderArgument(&'ast PlaceholderArgument),
    PositionalArgument(&'ast PositionalArgument<'arena>),
    VariadicPlaceholderArgument(&'ast VariadicPlaceholderArgument),
    Array(&'ast Array<'arena>),
    ArrayAccess(&'ast ArrayAccess<'arena>),
    ArrayAppend(&'ast ArrayAppend<'arena>),
    ArrayElement(&'ast ArrayElement<'arena>),
    KeyValueArrayElement(&'ast KeyValueArrayElement<'arena>),
    LegacyArray(&'ast LegacyArray<'arena>),
    List(&'ast List<'arena>),
    MissingArrayElement(&'ast MissingArrayElement),
    ValueArrayElement(&'ast ValueArrayElement<'arena>),
    VariadicArrayElement(&'ast VariadicArrayElement<'arena>),
    Attribute(&'ast Attribute<'arena>),
    AttributeList(&'ast AttributeList<'arena>),
    Block(&'ast Block<'arena>),
    Call(&'ast Call<'arena>),
    FunctionCall(&'ast FunctionCall<'arena>),
    MethodCall(&'ast MethodCall<'arena>),
    NullSafeMethodCall(&'ast NullSafeMethodCall<'arena>),
    StaticMethodCall(&'ast StaticMethodCall<'arena>),
    PartialApplication(&'ast PartialApplication<'arena>),
    FunctionPartialApplication(&'ast FunctionPartialApplication<'arena>),
    MethodPartialApplication(&'ast MethodPartialApplication<'arena>),
    StaticMethodPartialApplication(&'ast StaticMethodPartialApplication<'arena>),
    ClassLikeConstant(&'ast ClassLikeConstant<'arena>),
    ClassLikeConstantItem(&'ast ClassLikeConstantItem<'arena>),
    EnumCase(&'ast EnumCase<'arena>),
    EnumCaseBackedItem(&'ast EnumCaseBackedItem<'arena>),
    EnumCaseItem(&'ast EnumCaseItem<'arena>),
    EnumCaseUnitItem(&'ast EnumCaseUnitItem<'arena>),
    Extends(&'ast Extends<'arena>),
    Implements(&'ast Implements<'arena>),
    ClassLikeConstantSelector(&'ast ClassLikeConstantSelector<'arena>),
    ClassLikeMember(&'ast ClassLikeMember<'arena>),
    ClassLikeMemberExpressionSelector(&'ast ClassLikeMemberExpressionSelector<'arena>),
    ClassLikeMemberSelector(&'ast ClassLikeMemberSelector<'arena>),
    Method(&'ast Method<'arena>),
    MethodAbstractBody(&'ast MethodAbstractBody),
    MethodBody(&'ast MethodBody<'arena>),
    HookedProperty(&'ast HookedProperty<'arena>),
    PlainProperty(&'ast PlainProperty<'arena>),
    Property(&'ast Property<'arena>),
    PropertyAbstractItem(&'ast PropertyAbstractItem<'arena>),
    PropertyConcreteItem(&'ast PropertyConcreteItem<'arena>),
    PropertyHook(&'ast PropertyHook<'arena>),
    PropertyHookAbstractBody(&'ast PropertyHookAbstractBody),
    PropertyHookBody(&'ast PropertyHookBody<'arena>),
    PropertyHookConcreteBody(&'ast PropertyHookConcreteBody<'arena>),
    PropertyHookConcreteExpressionBody(&'ast PropertyHookConcreteExpressionBody<'arena>),
    PropertyHookList(&'ast PropertyHookList<'arena>),
    PropertyItem(&'ast PropertyItem<'arena>),
    TraitUse(&'ast TraitUse<'arena>),
    TraitUseAbsoluteMethodReference(&'ast TraitUseAbsoluteMethodReference<'arena>),
    TraitUseAbstractSpecification(&'ast TraitUseAbstractSpecification<'arena>),
    TraitUseAdaptation(&'ast TraitUseAdaptation<'arena>),
    TraitUseAliasAdaptation(&'ast TraitUseAliasAdaptation<'arena>),
    TraitUseConcreteSpecification(&'ast TraitUseConcreteSpecification<'arena>),
    TraitUseMethodReference(&'ast TraitUseMethodReference<'arena>),
    TraitUsePrecedenceAdaptation(&'ast TraitUsePrecedenceAdaptation<'arena>),
    TraitUseSpecification(&'ast TraitUseSpecification<'arena>),
    AnonymousClass(&'ast AnonymousClass<'arena>),
    Class(&'ast Class<'arena>),
    Enum(&'ast Enum<'arena>),
    EnumBackingTypeHint(&'ast EnumBackingTypeHint<'arena>),
    Interface(&'ast Interface<'arena>),
    Trait(&'ast Trait<'arena>),
    Clone(&'ast Clone<'arena>),
    Constant(&'ast Constant<'arena>),
    ConstantItem(&'ast ConstantItem<'arena>),
    Construct(&'ast Construct<'arena>),
    DieConstruct(&'ast DieConstruct<'arena>),
    EmptyConstruct(&'ast EmptyConstruct<'arena>),
    EvalConstruct(&'ast EvalConstruct<'arena>),
    ExitConstruct(&'ast ExitConstruct<'arena>),
    IncludeConstruct(&'ast IncludeConstruct<'arena>),
    IncludeOnceConstruct(&'ast IncludeOnceConstruct<'arena>),
    IssetConstruct(&'ast IssetConstruct<'arena>),
    PrintConstruct(&'ast PrintConstruct<'arena>),
    RequireConstruct(&'ast RequireConstruct<'arena>),
    RequireOnceConstruct(&'ast RequireOnceConstruct<'arena>),
    If(&'ast If<'arena>),
    IfBody(&'ast IfBody<'arena>),
    IfColonDelimitedBody(&'ast IfColonDelimitedBody<'arena>),
    IfColonDelimitedBodyElseClause(&'ast IfColonDelimitedBodyElseClause<'arena>),
    IfColonDelimitedBodyElseIfClause(&'ast IfColonDelimitedBodyElseIfClause<'arena>),
    IfStatementBody(&'ast IfStatementBody<'arena>),
    IfStatementBodyElseClause(&'ast IfStatementBodyElseClause<'arena>),
    IfStatementBodyElseIfClause(&'ast IfStatementBodyElseIfClause<'arena>),
    Match(&'ast Match<'arena>),
    MatchArm(&'ast MatchArm<'arena>),
    MatchDefaultArm(&'ast MatchDefaultArm<'arena>),
    MatchExpressionArm(&'ast MatchExpressionArm<'arena>),
    Switch(&'ast Switch<'arena>),
    SwitchBody(&'ast SwitchBody<'arena>),
    SwitchBraceDelimitedBody(&'ast SwitchBraceDelimitedBody<'arena>),
    SwitchCase(&'ast SwitchCase<'arena>),
    SwitchCaseSeparator(&'ast SwitchCaseSeparator),
    SwitchColonDelimitedBody(&'ast SwitchColonDelimitedBody<'arena>),
    SwitchDefaultCase(&'ast SwitchDefaultCase<'arena>),
    SwitchExpressionCase(&'ast SwitchExpressionCase<'arena>),
    Declare(&'ast Declare<'arena>),
    DeclareBody(&'ast DeclareBody<'arena>),
    DeclareColonDelimitedBody(&'ast DeclareColonDelimitedBody<'arena>),
    DeclareItem(&'ast DeclareItem<'arena>),
    EchoTag(&'ast EchoTag<'arena>),
    Echo(&'ast Echo<'arena>),
    Expression(&'ast Expression<'arena>),
    Binary(&'ast Binary<'arena>),
    BinaryOperator(&'ast BinaryOperator<'arena>),
    UnaryPrefix(&'ast UnaryPrefix<'arena>),
    UnaryPrefixOperator(&'ast UnaryPrefixOperator<'arena>),
    UnaryPostfix(&'ast UnaryPostfix<'arena>),
    UnaryPostfixOperator(&'ast UnaryPostfixOperator),
    Parenthesized(&'ast Parenthesized<'arena>),
    ArrowFunction(&'ast ArrowFunction<'arena>),
    Closure(&'ast Closure<'arena>),
    ClosureUseClause(&'ast ClosureUseClause<'arena>),
    ClosureUseClauseVariable(&'ast ClosureUseClauseVariable<'arena>),
    Function(&'ast Function<'arena>),
    FunctionLikeParameter(&'ast FunctionLikeParameter<'arena>),
    FunctionLikeParameterDefaultValue(&'ast FunctionLikeParameterDefaultValue<'arena>),
    FunctionLikeParameterList(&'ast FunctionLikeParameterList<'arena>),
    FunctionLikeReturnTypeHint(&'ast FunctionLikeReturnTypeHint<'arena>),
    Global(&'ast Global<'arena>),
    Goto(&'ast Goto<'arena>),
    Label(&'ast Label<'arena>),
    HaltCompiler(&'ast HaltCompiler<'arena>),
    FullyQualifiedIdentifier(&'ast FullyQualifiedIdentifier<'arena>),
    Identifier(&'ast Identifier<'arena>),
    LocalIdentifier(&'ast LocalIdentifier<'arena>),
    QualifiedIdentifier(&'ast QualifiedIdentifier<'arena>),
    Inline(&'ast Inline<'arena>),
    Instantiation(&'ast Instantiation<'arena>),
    Keyword(&'ast Keyword<'arena>),
    Literal(&'ast Literal<'arena>),
    LiteralFloat(&'ast LiteralFloat<'arena>),
    LiteralInteger(&'ast LiteralInteger<'arena>),
    LiteralString(&'ast LiteralString<'arena>),
    MagicConstant(&'ast MagicConstant<'arena>),
    Modifier(&'ast Modifier<'arena>),
    Namespace(&'ast Namespace<'arena>),
    NamespaceBody(&'ast NamespaceBody<'arena>),
    NamespaceImplicitBody(&'ast NamespaceImplicitBody<'arena>),
    Assignment(&'ast Assignment<'arena>),
    AssignmentOperator(&'ast AssignmentOperator),
    Conditional(&'ast Conditional<'arena>),
    DoWhile(&'ast DoWhile<'arena>),
    Foreach(&'ast Foreach<'arena>),
    ForeachBody(&'ast ForeachBody<'arena>),
    ForeachColonDelimitedBody(&'ast ForeachColonDelimitedBody<'arena>),
    ForeachKeyValueTarget(&'ast ForeachKeyValueTarget<'arena>),
    ForeachTarget(&'ast ForeachTarget<'arena>),
    ForeachValueTarget(&'ast ForeachValueTarget<'arena>),
    For(&'ast For<'arena>),
    ForBody(&'ast ForBody<'arena>),
    ForColonDelimitedBody(&'ast ForColonDelimitedBody<'arena>),
    While(&'ast While<'arena>),
    WhileBody(&'ast WhileBody<'arena>),
    WhileColonDelimitedBody(&'ast WhileColonDelimitedBody<'arena>),
    Break(&'ast Break<'arena>),
    Continue(&'ast Continue<'arena>),
    Return(&'ast Return<'arena>),
    Static(&'ast Static<'arena>),
    StaticAbstractItem(&'ast StaticAbstractItem<'arena>),
    StaticConcreteItem(&'ast StaticConcreteItem<'arena>),
    StaticItem(&'ast StaticItem<'arena>),
    Try(&'ast Try<'arena>),
    TryCatchClause(&'ast TryCatchClause<'arena>),
    TryFinallyClause(&'ast TryFinallyClause<'arena>),
    MaybeTypedUseItem(&'ast MaybeTypedUseItem<'arena>),
    MixedUseItemList(&'ast MixedUseItemList<'arena>),
    TypedUseItemList(&'ast TypedUseItemList<'arena>),
    TypedUseItemSequence(&'ast TypedUseItemSequence<'arena>),
    Use(&'ast Use<'arena>),
    UseItem(&'ast UseItem<'arena>),
    UseItemAlias(&'ast UseItemAlias<'arena>),
    UseItemSequence(&'ast UseItemSequence<'arena>),
    UseItems(&'ast UseItems<'arena>),
    UseType(&'ast UseType<'arena>),
    Yield(&'ast Yield<'arena>),
    YieldFrom(&'ast YieldFrom<'arena>),
    YieldPair(&'ast YieldPair<'arena>),
    YieldValue(&'ast YieldValue<'arena>),
    Statement(&'ast Statement<'arena>),
    ExpressionStatement(&'ast ExpressionStatement<'arena>),
    BracedExpressionStringPart(&'ast BracedExpressionStringPart<'arena>),
    DocumentString(&'ast DocumentString<'arena>),
    InterpolatedString(&'ast InterpolatedString<'arena>),
    LiteralStringPart(&'ast LiteralStringPart<'arena>),
    ShellExecuteString(&'ast ShellExecuteString<'arena>),
    CompositeString(&'ast CompositeString<'arena>),
    StringPart(&'ast StringPart<'arena>),
    ClosingTag(&'ast ClosingTag),
    FullOpeningTag(&'ast FullOpeningTag<'arena>),
    OpeningTag(&'ast OpeningTag<'arena>),
    ShortOpeningTag(&'ast ShortOpeningTag),
    Terminator(&'ast Terminator<'arena>),
    Throw(&'ast Throw<'arena>),
    Hint(&'ast Hint<'arena>),
    IntersectionHint(&'ast IntersectionHint<'arena>),
    NullableHint(&'ast NullableHint<'arena>),
    ParenthesizedHint(&'ast ParenthesizedHint<'arena>),
    UnionHint(&'ast UnionHint<'arena>),
    Unset(&'ast Unset<'arena>),
    DirectVariable(&'ast DirectVariable<'arena>),
    IndirectVariable(&'ast IndirectVariable<'arena>),
    NestedVariable(&'ast NestedVariable<'arena>),
    Variable(&'ast Variable<'arena>),
    Pipe(&'ast Pipe<'arena>),
    Error(Span),
    MissingTerminator(Span),
    ClassLikeMemberMissingSelector(Span),
    ClassLikeConstantMissingSelector(Span),
}

impl<'ast, 'arena> Node<'ast, 'arena> {
    #[inline]
    pub fn filter_map<F, T: 'ast>(&self, f: F) -> Vec<T>
    where
        F: Fn(&Node<'ast, 'arena>) -> Option<T>,
    {
        let mut result = vec![];
        self.filter_map_internal(&f, &mut result);
        result
    }

    #[inline]
    fn filter_map_internal<F, T: 'ast>(&self, f: &F, result: &mut Vec<T>)
    where
        F: Fn(&Node<'ast, 'arena>) -> Option<T>,
    {
        self.visit_children(|child| child.filter_map_internal(f, result));

        if let Some(item) = f(self) {
            result.push(item);
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_declaration(&self) -> bool {
        matches!(
            self,
            Self::Class(_) | Self::Interface(_) | Self::Trait(_) | Self::Enum(_) | Self::Function(_) | Self::Method(_)
        )
    }

    #[inline]
    #[must_use]
    pub const fn is_statement(&self) -> bool {
        matches!(
            self,
            Self::Statement(_)
                | Self::OpeningTag(_)
                | Self::FullOpeningTag(_)
                | Self::ShortOpeningTag(_)
                | Self::ClosingTag(_)
                | Self::Inline(_)
                | Self::Namespace(_)
                | Self::Use(_)
                | Self::Class(_)
                | Self::Interface(_)
                | Self::Trait(_)
                | Self::Enum(_)
                | Self::Block(_)
                | Self::Constant(_)
                | Self::Function(_)
                | Self::Declare(_)
                | Self::Goto(_)
                | Self::Label(_)
                | Self::Try(_)
                | Self::Foreach(_)
                | Self::For(_)
                | Self::While(_)
                | Self::DoWhile(_)
                | Self::Continue(_)
                | Self::Break(_)
                | Self::Switch(_)
                | Self::If(_)
                | Self::Return(_)
                | Self::ExpressionStatement(_)
                | Self::Echo(_)
                | Self::EchoTag(_)
                | Self::Global(_)
                | Self::Static(_)
                | Self::HaltCompiler(_)
                | Self::Unset(_)
        )
    }

    #[inline]
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        match &self {
            Self::Program(_) => NodeKind::Program,
            Self::Access(_) => NodeKind::Access,
            Self::ConstantAccess(_) => NodeKind::ConstantAccess,
            Self::ClassConstantAccess(_) => NodeKind::ClassConstantAccess,
            Self::NullSafePropertyAccess(_) => NodeKind::NullSafePropertyAccess,
            Self::PropertyAccess(_) => NodeKind::PropertyAccess,
            Self::StaticPropertyAccess(_) => NodeKind::StaticPropertyAccess,
            Self::Argument(_) => NodeKind::Argument,
            Self::ArgumentList(_) => NodeKind::ArgumentList,
            Self::PartialArgument(_) => NodeKind::PartialArgument,
            Self::PartialArgumentList(_) => NodeKind::PartialArgumentList,
            Self::NamedArgument(_) => NodeKind::NamedArgument,
            Self::NamedPlaceholderArgument(_) => NodeKind::NamedPlaceholderArgument,
            Self::PlaceholderArgument(_) => NodeKind::PlaceholderArgument,
            Self::PositionalArgument(_) => NodeKind::PositionalArgument,
            Self::VariadicPlaceholderArgument(_) => NodeKind::VariadicPlaceholderArgument,
            Self::Array(_) => NodeKind::Array,
            Self::ArrayAccess(_) => NodeKind::ArrayAccess,
            Self::ArrayAppend(_) => NodeKind::ArrayAppend,
            Self::ArrayElement(_) => NodeKind::ArrayElement,
            Self::KeyValueArrayElement(_) => NodeKind::KeyValueArrayElement,
            Self::LegacyArray(_) => NodeKind::LegacyArray,
            Self::List(_) => NodeKind::List,
            Self::MissingArrayElement(_) => NodeKind::MissingArrayElement,
            Self::ValueArrayElement(_) => NodeKind::ValueArrayElement,
            Self::VariadicArrayElement(_) => NodeKind::VariadicArrayElement,
            Self::Attribute(_) => NodeKind::Attribute,
            Self::AttributeList(_) => NodeKind::AttributeList,
            Self::Block(_) => NodeKind::Block,
            Self::Call(_) => NodeKind::Call,
            Self::FunctionCall(_) => NodeKind::FunctionCall,
            Self::MethodCall(_) => NodeKind::MethodCall,
            Self::NullSafeMethodCall(_) => NodeKind::NullSafeMethodCall,
            Self::StaticMethodCall(_) => NodeKind::StaticMethodCall,
            Self::PartialApplication(_) => NodeKind::PartialApplication,
            Self::FunctionPartialApplication(_) => NodeKind::FunctionPartialApplication,
            Self::MethodPartialApplication(_) => NodeKind::MethodPartialApplication,
            Self::StaticMethodPartialApplication(_) => NodeKind::StaticMethodPartialApplication,
            Self::ClassLikeConstant(_) => NodeKind::ClassLikeConstant,
            Self::ClassLikeConstantItem(_) => NodeKind::ClassLikeConstantItem,
            Self::EnumCase(_) => NodeKind::EnumCase,
            Self::EnumCaseBackedItem(_) => NodeKind::EnumCaseBackedItem,
            Self::EnumCaseItem(_) => NodeKind::EnumCaseItem,
            Self::EnumCaseUnitItem(_) => NodeKind::EnumCaseUnitItem,
            Self::Extends(_) => NodeKind::Extends,
            Self::Implements(_) => NodeKind::Implements,
            Self::ClassLikeConstantSelector(_) => NodeKind::ClassLikeConstantSelector,
            Self::ClassLikeMember(_) => NodeKind::ClassLikeMember,
            Self::ClassLikeMemberExpressionSelector(_) => NodeKind::ClassLikeMemberExpressionSelector,
            Self::ClassLikeMemberSelector(_) => NodeKind::ClassLikeMemberSelector,
            Self::Method(_) => NodeKind::Method,
            Self::MethodAbstractBody(_) => NodeKind::MethodAbstractBody,
            Self::MethodBody(_) => NodeKind::MethodBody,
            Self::HookedProperty(_) => NodeKind::HookedProperty,
            Self::PlainProperty(_) => NodeKind::PlainProperty,
            Self::Property(_) => NodeKind::Property,
            Self::PropertyAbstractItem(_) => NodeKind::PropertyAbstractItem,
            Self::PropertyConcreteItem(_) => NodeKind::PropertyConcreteItem,
            Self::PropertyHook(_) => NodeKind::PropertyHook,
            Self::PropertyHookAbstractBody(_) => NodeKind::PropertyHookAbstractBody,
            Self::PropertyHookBody(_) => NodeKind::PropertyHookBody,
            Self::PropertyHookConcreteBody(_) => NodeKind::PropertyHookConcreteBody,
            Self::PropertyHookConcreteExpressionBody(_) => NodeKind::PropertyHookConcreteExpressionBody,
            Self::PropertyHookList(_) => NodeKind::PropertyHookList,
            Self::PropertyItem(_) => NodeKind::PropertyItem,
            Self::TraitUse(_) => NodeKind::TraitUse,
            Self::TraitUseAbsoluteMethodReference(_) => NodeKind::TraitUseAbsoluteMethodReference,
            Self::TraitUseAbstractSpecification(_) => NodeKind::TraitUseAbstractSpecification,
            Self::TraitUseAdaptation(_) => NodeKind::TraitUseAdaptation,
            Self::TraitUseAliasAdaptation(_) => NodeKind::TraitUseAliasAdaptation,
            Self::TraitUseConcreteSpecification(_) => NodeKind::TraitUseConcreteSpecification,
            Self::TraitUseMethodReference(_) => NodeKind::TraitUseMethodReference,
            Self::TraitUsePrecedenceAdaptation(_) => NodeKind::TraitUsePrecedenceAdaptation,
            Self::TraitUseSpecification(_) => NodeKind::TraitUseSpecification,
            Self::AnonymousClass(_) => NodeKind::AnonymousClass,
            Self::Class(_) => NodeKind::Class,
            Self::Enum(_) => NodeKind::Enum,
            Self::EnumBackingTypeHint(_) => NodeKind::EnumBackingTypeHint,
            Self::Interface(_) => NodeKind::Interface,
            Self::Trait(_) => NodeKind::Trait,
            Self::Clone(_) => NodeKind::Clone,
            Self::Constant(_) => NodeKind::Constant,
            Self::ConstantItem(_) => NodeKind::ConstantItem,
            Self::Construct(_) => NodeKind::Construct,
            Self::DieConstruct(_) => NodeKind::DieConstruct,
            Self::EmptyConstruct(_) => NodeKind::EmptyConstruct,
            Self::EvalConstruct(_) => NodeKind::EvalConstruct,
            Self::ExitConstruct(_) => NodeKind::ExitConstruct,
            Self::IncludeConstruct(_) => NodeKind::IncludeConstruct,
            Self::IncludeOnceConstruct(_) => NodeKind::IncludeOnceConstruct,
            Self::IssetConstruct(_) => NodeKind::IssetConstruct,
            Self::PrintConstruct(_) => NodeKind::PrintConstruct,
            Self::RequireConstruct(_) => NodeKind::RequireConstruct,
            Self::RequireOnceConstruct(_) => NodeKind::RequireOnceConstruct,
            Self::If(_) => NodeKind::If,
            Self::IfBody(_) => NodeKind::IfBody,
            Self::IfColonDelimitedBody(_) => NodeKind::IfColonDelimitedBody,
            Self::IfColonDelimitedBodyElseClause(_) => NodeKind::IfColonDelimitedBodyElseClause,
            Self::IfColonDelimitedBodyElseIfClause(_) => NodeKind::IfColonDelimitedBodyElseIfClause,
            Self::IfStatementBody(_) => NodeKind::IfStatementBody,
            Self::IfStatementBodyElseClause(_) => NodeKind::IfStatementBodyElseClause,
            Self::IfStatementBodyElseIfClause(_) => NodeKind::IfStatementBodyElseIfClause,
            Self::Match(_) => NodeKind::Match,
            Self::MatchArm(_) => NodeKind::MatchArm,
            Self::MatchDefaultArm(_) => NodeKind::MatchDefaultArm,
            Self::MatchExpressionArm(_) => NodeKind::MatchExpressionArm,
            Self::Switch(_) => NodeKind::Switch,
            Self::SwitchBody(_) => NodeKind::SwitchBody,
            Self::SwitchBraceDelimitedBody(_) => NodeKind::SwitchBraceDelimitedBody,
            Self::SwitchCase(_) => NodeKind::SwitchCase,
            Self::SwitchCaseSeparator(_) => NodeKind::SwitchCaseSeparator,
            Self::SwitchColonDelimitedBody(_) => NodeKind::SwitchColonDelimitedBody,
            Self::SwitchDefaultCase(_) => NodeKind::SwitchDefaultCase,
            Self::SwitchExpressionCase(_) => NodeKind::SwitchExpressionCase,
            Self::Declare(_) => NodeKind::Declare,
            Self::DeclareBody(_) => NodeKind::DeclareBody,
            Self::DeclareColonDelimitedBody(_) => NodeKind::DeclareColonDelimitedBody,
            Self::DeclareItem(_) => NodeKind::DeclareItem,
            Self::Echo(_) => NodeKind::Echo,
            Self::Expression(_) => NodeKind::Expression,
            Self::Binary(_) => NodeKind::Binary,
            Self::BinaryOperator(_) => NodeKind::BinaryOperator,
            Self::UnaryPrefix(_) => NodeKind::UnaryPrefix,
            Self::UnaryPrefixOperator(_) => NodeKind::UnaryPrefixOperator,
            Self::UnaryPostfix(_) => NodeKind::UnaryPostfix,
            Self::UnaryPostfixOperator(_) => NodeKind::UnaryPostfixOperator,
            Self::Parenthesized(_) => NodeKind::Parenthesized,
            Self::ArrowFunction(_) => NodeKind::ArrowFunction,
            Self::Closure(_) => NodeKind::Closure,
            Self::ClosureUseClause(_) => NodeKind::ClosureUseClause,
            Self::ClosureUseClauseVariable(_) => NodeKind::ClosureUseClauseVariable,
            Self::Function(_) => NodeKind::Function,
            Self::FunctionLikeParameter(_) => NodeKind::FunctionLikeParameter,
            Self::FunctionLikeParameterDefaultValue(_) => NodeKind::FunctionLikeParameterDefaultValue,
            Self::FunctionLikeParameterList(_) => NodeKind::FunctionLikeParameterList,
            Self::FunctionLikeReturnTypeHint(_) => NodeKind::FunctionLikeReturnTypeHint,
            Self::Global(_) => NodeKind::Global,
            Self::Goto(_) => NodeKind::Goto,
            Self::Label(_) => NodeKind::Label,
            Self::HaltCompiler(_) => NodeKind::HaltCompiler,
            Self::FullyQualifiedIdentifier(_) => NodeKind::FullyQualifiedIdentifier,
            Self::Identifier(_) => NodeKind::Identifier,
            Self::LocalIdentifier(_) => NodeKind::LocalIdentifier,
            Self::QualifiedIdentifier(_) => NodeKind::QualifiedIdentifier,
            Self::Inline(_) => NodeKind::Inline,
            Self::Instantiation(_) => NodeKind::Instantiation,
            Self::Keyword(_) => NodeKind::Keyword,
            Self::Literal(_) => NodeKind::Literal,
            Self::LiteralFloat(_) => NodeKind::LiteralFloat,
            Self::LiteralInteger(_) => NodeKind::LiteralInteger,
            Self::LiteralString(_) => NodeKind::LiteralString,
            Self::MagicConstant(_) => NodeKind::MagicConstant,
            Self::Modifier(_) => NodeKind::Modifier,
            Self::Namespace(_) => NodeKind::Namespace,
            Self::NamespaceBody(_) => NodeKind::NamespaceBody,
            Self::NamespaceImplicitBody(_) => NodeKind::NamespaceImplicitBody,
            Self::Assignment(_) => NodeKind::Assignment,
            Self::AssignmentOperator(_) => NodeKind::AssignmentOperator,
            Self::Conditional(_) => NodeKind::Conditional,
            Self::DoWhile(_) => NodeKind::DoWhile,
            Self::Foreach(_) => NodeKind::Foreach,
            Self::ForeachBody(_) => NodeKind::ForeachBody,
            Self::ForeachColonDelimitedBody(_) => NodeKind::ForeachColonDelimitedBody,
            Self::ForeachKeyValueTarget(_) => NodeKind::ForeachKeyValueTarget,
            Self::ForeachTarget(_) => NodeKind::ForeachTarget,
            Self::ForeachValueTarget(_) => NodeKind::ForeachValueTarget,
            Self::For(_) => NodeKind::For,
            Self::ForBody(_) => NodeKind::ForBody,
            Self::ForColonDelimitedBody(_) => NodeKind::ForColonDelimitedBody,
            Self::While(_) => NodeKind::While,
            Self::WhileBody(_) => NodeKind::WhileBody,
            Self::WhileColonDelimitedBody(_) => NodeKind::WhileColonDelimitedBody,
            Self::Break(_) => NodeKind::Break,
            Self::Continue(_) => NodeKind::Continue,
            Self::Return(_) => NodeKind::Return,
            Self::Static(_) => NodeKind::Static,
            Self::StaticAbstractItem(_) => NodeKind::StaticAbstractItem,
            Self::StaticConcreteItem(_) => NodeKind::StaticConcreteItem,
            Self::StaticItem(_) => NodeKind::StaticItem,
            Self::Try(_) => NodeKind::Try,
            Self::TryCatchClause(_) => NodeKind::TryCatchClause,
            Self::TryFinallyClause(_) => NodeKind::TryFinallyClause,
            Self::MaybeTypedUseItem(_) => NodeKind::MaybeTypedUseItem,
            Self::MixedUseItemList(_) => NodeKind::MixedUseItemList,
            Self::TypedUseItemList(_) => NodeKind::TypedUseItemList,
            Self::TypedUseItemSequence(_) => NodeKind::TypedUseItemSequence,
            Self::Use(_) => NodeKind::Use,
            Self::UseItem(_) => NodeKind::UseItem,
            Self::UseItemAlias(_) => NodeKind::UseItemAlias,
            Self::UseItemSequence(_) => NodeKind::UseItemSequence,
            Self::UseItems(_) => NodeKind::UseItems,
            Self::UseType(_) => NodeKind::UseType,
            Self::Yield(_) => NodeKind::Yield,
            Self::YieldFrom(_) => NodeKind::YieldFrom,
            Self::YieldPair(_) => NodeKind::YieldPair,
            Self::YieldValue(_) => NodeKind::YieldValue,
            Self::Statement(_) => NodeKind::Statement,
            Self::ExpressionStatement(_) => NodeKind::ExpressionStatement,
            Self::BracedExpressionStringPart(_) => NodeKind::BracedExpressionStringPart,
            Self::DocumentString(_) => NodeKind::DocumentString,
            Self::InterpolatedString(_) => NodeKind::InterpolatedString,
            Self::LiteralStringPart(_) => NodeKind::LiteralStringPart,
            Self::ShellExecuteString(_) => NodeKind::ShellExecuteString,
            Self::CompositeString(_) => NodeKind::CompositeString,
            Self::StringPart(_) => NodeKind::StringPart,
            Self::ClosingTag(_) => NodeKind::ClosingTag,
            Self::EchoTag(_) => NodeKind::EchoTag,
            Self::FullOpeningTag(_) => NodeKind::FullOpeningTag,
            Self::OpeningTag(_) => NodeKind::OpeningTag,
            Self::ShortOpeningTag(_) => NodeKind::ShortOpeningTag,
            Self::Terminator(_) => NodeKind::Terminator,
            Self::Throw(_) => NodeKind::Throw,
            Self::Hint(_) => NodeKind::Hint,
            Self::IntersectionHint(_) => NodeKind::IntersectionHint,
            Self::NullableHint(_) => NodeKind::NullableHint,
            Self::ParenthesizedHint(_) => NodeKind::ParenthesizedHint,
            Self::UnionHint(_) => NodeKind::UnionHint,
            Self::Unset(_) => NodeKind::Unset,
            Self::DirectVariable(_) => NodeKind::DirectVariable,
            Self::IndirectVariable(_) => NodeKind::IndirectVariable,
            Self::NestedVariable(_) => NodeKind::NestedVariable,
            Self::Variable(_) => NodeKind::Variable,
            Self::Pipe(_) => NodeKind::Pipe,
            Self::Error(_) => NodeKind::Error,
            Self::MissingTerminator(_) => NodeKind::MissingTerminator,
            Self::ClassLikeMemberMissingSelector(_) => NodeKind::ClassLikeMemberMissingSelector,
            Self::ClassLikeConstantMissingSelector(_) => NodeKind::ClassLikeConstantMissingSelector,
        }
    }

    pub fn visit_children<F: FnMut(Node<'ast, 'arena>)>(&self, mut f: F) {
        match &self {
            Node::Program(node) => {
                for node in node.statements.as_slice() {
                    f(Node::Statement(node));
                }
            }
            Node::Access(node) => match &node {
                Access::Property(node) => f(Node::PropertyAccess(node)),
                Access::NullSafeProperty(node) => f(Node::NullSafePropertyAccess(node)),
                Access::StaticProperty(node) => f(Node::StaticPropertyAccess(node)),
                Access::ClassConstant(node) => f(Node::ClassConstantAccess(node)),
            },
            Node::ConstantAccess(node) => {
                f(Node::Identifier(&node.name));
            }
            Node::ClassConstantAccess(node) => {
                f(Node::Expression(node.class));
                f(Node::ClassLikeConstantSelector(&node.constant));
            }
            Node::NullSafePropertyAccess(node) => {
                f(Node::Expression(node.object));
                f(Node::ClassLikeMemberSelector(&node.property));
            }
            Node::PropertyAccess(node) => {
                f(Node::Expression(node.object));
                f(Node::ClassLikeMemberSelector(&node.property));
            }
            Node::StaticPropertyAccess(node) => {
                f(Node::Expression(node.class));
                f(Node::Variable(&node.property));
            }
            Node::Argument(node) => match &node {
                Argument::Named(node) => f(Node::NamedArgument(node)),
                Argument::Positional(node) => f(Node::PositionalArgument(node)),
            },
            Node::ArgumentList(node) => {
                for node in node.arguments.as_slice() {
                    f(Node::Argument(node));
                }
            }
            Node::PartialArgument(node) => match &node {
                PartialArgument::Named(node) => f(Node::NamedArgument(node)),
                PartialArgument::NamedPlaceholder(node) => f(Node::NamedPlaceholderArgument(node)),
                PartialArgument::Placeholder(node) => f(Node::PlaceholderArgument(node)),
                PartialArgument::Positional(node) => f(Node::PositionalArgument(node)),
                PartialArgument::VariadicPlaceholder(node) => f(Node::VariadicPlaceholderArgument(node)),
            },
            Node::PartialArgumentList(node) => {
                for node in node.arguments.as_slice() {
                    f(Node::PartialArgument(node));
                }
            }
            Node::NamedArgument(node) => {
                f(Node::LocalIdentifier(&node.name));
                f(Node::Expression(node.value));
            }
            Node::NamedPlaceholderArgument(node) => {
                f(Node::LocalIdentifier(&node.name));
            }
            Node::PlaceholderArgument(_) => {}
            Node::PositionalArgument(node) => f(Node::Expression(node.value)),
            Node::VariadicPlaceholderArgument(_) => {}
            Node::Array(node) => {
                for node in node.elements.as_slice() {
                    f(Node::ArrayElement(node));
                }
            }
            Node::ArrayAccess(node) => {
                f(Node::Expression(node.array));
                f(Node::Expression(node.index));
            }
            Node::ArrayAppend(node) => {
                f(Node::Expression(node.array));
            }
            Node::ArrayElement(node) => match &node {
                ArrayElement::KeyValue(node) => f(Node::KeyValueArrayElement(node)),
                ArrayElement::Missing(node) => f(Node::MissingArrayElement(node)),
                ArrayElement::Value(node) => f(Node::ValueArrayElement(node)),
                ArrayElement::Variadic(node) => f(Node::VariadicArrayElement(node)),
            },
            Node::KeyValueArrayElement(node) => {
                f(Node::Expression(node.key));
                f(Node::Expression(node.value));
            }
            Node::LegacyArray(node) => {
                for item in node.elements.iter() {
                    f(Node::ArrayElement(item));
                }
            }
            Node::List(node) => {
                for item in node.elements.iter() {
                    f(Node::ArrayElement(item));
                }
            }
            Node::MissingArrayElement(_) => {}
            Node::ValueArrayElement(node) => f(Node::Expression(node.value)),
            Node::VariadicArrayElement(node) => f(Node::Expression(node.value)),
            Node::Attribute(node) => {
                f(Node::Identifier(&node.name));
                if let Some(arguments) = &node.argument_list {
                    f(Node::ArgumentList(arguments));
                }
            }
            Node::AttributeList(node) => {
                for item in node.attributes.iter() {
                    f(Node::Attribute(item));
                }
            }
            Node::Block(node) => {
                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
            }
            Node::Call(node) => match node {
                Call::Function(node) => f(Node::FunctionCall(node)),
                Call::Method(node) => f(Node::MethodCall(node)),
                Call::NullSafeMethod(node) => f(Node::NullSafeMethodCall(node)),
                Call::StaticMethod(node) => f(Node::StaticMethodCall(node)),
            },
            Node::FunctionCall(node) => {
                f(Node::Expression(node.function));
                f(Node::ArgumentList(&node.argument_list));
            }
            Node::MethodCall(node) => {
                f(Node::Expression(node.object));
                f(Node::ClassLikeMemberSelector(&node.method));
                f(Node::ArgumentList(&node.argument_list));
            }
            Node::NullSafeMethodCall(node) => {
                f(Node::Expression(node.object));
                f(Node::ClassLikeMemberSelector(&node.method));
                f(Node::ArgumentList(&node.argument_list));
            }
            Node::StaticMethodCall(node) => {
                f(Node::Expression(node.class));
                f(Node::ClassLikeMemberSelector(&node.method));
                f(Node::ArgumentList(&node.argument_list));
            }
            Node::PartialApplication(node) => match node {
                PartialApplication::Function(node) => f(Node::FunctionPartialApplication(node)),
                PartialApplication::Method(node) => f(Node::MethodPartialApplication(node)),
                PartialApplication::StaticMethod(node) => f(Node::StaticMethodPartialApplication(node)),
            },
            Node::FunctionPartialApplication(node) => {
                f(Node::Expression(node.function));
                f(Node::PartialArgumentList(&node.argument_list));
            }
            Node::MethodPartialApplication(node) => {
                f(Node::Expression(node.object));
                f(Node::ClassLikeMemberSelector(&node.method));
                f(Node::PartialArgumentList(&node.argument_list));
            }
            Node::StaticMethodPartialApplication(node) => {
                f(Node::Expression(node.class));
                f(Node::ClassLikeMemberSelector(&node.method));
                f(Node::PartialArgumentList(&node.argument_list));
            }
            Node::ClassLikeConstant(node) => {
                for attr in &node.attribute_lists {
                    f(Node::AttributeList(attr));
                }

                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                f(Node::Keyword(&node.r#const));
                if let Some(hint) = &node.hint {
                    f(Node::Hint(hint));
                }

                for item in node.items.iter() {
                    f(Node::ClassLikeConstantItem(item));
                }
                f(Node::Terminator(&node.terminator));
            }
            Node::ClassLikeConstantItem(node) => {
                f(Node::LocalIdentifier(&node.name));
                f(Node::Expression(node.value));
            }
            Node::EnumCase(node) => {
                for attr in &node.attribute_lists {
                    f(Node::AttributeList(attr));
                }

                f(Node::Keyword(&node.case));
                f(Node::EnumCaseItem(&node.item));
                f(Node::Terminator(&node.terminator));
            }
            Node::EnumCaseBackedItem(node) => {
                f(Node::LocalIdentifier(&node.name));
                f(Node::Expression(node.value));
            }
            Node::EnumCaseItem(node) => match &node {
                EnumCaseItem::Backed(node) => f(Node::EnumCaseBackedItem(node)),
                EnumCaseItem::Unit(node) => f(Node::EnumCaseUnitItem(node)),
            },
            Node::EnumCaseUnitItem(node) => f(Node::LocalIdentifier(&node.name)),
            Node::Extends(node) => {
                f(Node::Keyword(&node.extends));
                for item in node.types.iter() {
                    f(Node::Identifier(item));
                }
            }
            Node::Implements(node) => {
                f(Node::Keyword(&node.implements));
                for item in node.types.iter() {
                    f(Node::Identifier(item));
                }
            }
            Node::ClassLikeConstantSelector(node) => match node {
                ClassLikeConstantSelector::Identifier(node) => f(Node::LocalIdentifier(node)),
                ClassLikeConstantSelector::Expression(node) => {
                    f(Node::ClassLikeMemberExpressionSelector(node));
                }
                ClassLikeConstantSelector::Missing(span) => f(Node::ClassLikeConstantMissingSelector(*span)),
            },
            Node::ClassLikeMember(node) => match node {
                ClassLikeMember::TraitUse(node) => f(Node::TraitUse(node)),
                ClassLikeMember::Constant(node) => f(Node::ClassLikeConstant(node)),
                ClassLikeMember::Property(node) => f(Node::Property(node)),
                ClassLikeMember::EnumCase(node) => f(Node::EnumCase(node)),
                ClassLikeMember::Method(node) => f(Node::Method(node)),
            },
            Node::ClassLikeMemberExpressionSelector(node) => f(Node::Expression(node.expression)),
            Node::ClassLikeMemberSelector(node) => match node {
                ClassLikeMemberSelector::Identifier(node) => f(Node::LocalIdentifier(node)),
                ClassLikeMemberSelector::Variable(node) => f(Node::Variable(node)),
                ClassLikeMemberSelector::Expression(node) => {
                    f(Node::ClassLikeMemberExpressionSelector(node));
                }
                ClassLikeMemberSelector::Missing(span) => f(Node::ClassLikeMemberMissingSelector(*span)),
            },
            Node::Method(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                f(Node::Keyword(&node.function));
                f(Node::LocalIdentifier(&node.name));
                f(Node::FunctionLikeParameterList(&node.parameter_list));
                for item in node.return_type_hint.iter() {
                    f(Node::FunctionLikeReturnTypeHint(item));
                }
                f(Node::MethodBody(&node.body));
            }
            Node::MethodAbstractBody(_) => {}
            Node::MethodBody(node) => match node {
                MethodBody::Abstract(node) => f(Node::MethodAbstractBody(node)),
                MethodBody::Concrete(node) => f(Node::Block(node)),
            },
            Node::HookedProperty(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                for item in node.var.iter() {
                    f(Node::Keyword(item));
                }
                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                for item in node.hint.iter() {
                    f(Node::Hint(item));
                }
                f(Node::PropertyItem(&node.item));
                f(Node::PropertyHookList(&node.hook_list));
            }
            Node::PlainProperty(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                for item in node.var.iter() {
                    f(Node::Keyword(item));
                }
                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                for item in node.hint.iter() {
                    f(Node::Hint(item));
                }
                for item in node.items.iter() {
                    f(Node::PropertyItem(item));
                }
            }
            Node::Property(node) => match node {
                Property::Plain(node) => f(Node::PlainProperty(node)),
                Property::Hooked(node) => f(Node::HookedProperty(node)),
            },
            Node::PropertyAbstractItem(node) => {
                f(Node::DirectVariable(&node.variable));
            }
            Node::PropertyConcreteItem(node) => {
                f(Node::DirectVariable(&node.variable));
                f(Node::Expression(node.value));
            }
            Node::PropertyHook(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                f(Node::LocalIdentifier(&node.name));
                for item in node.parameter_list.iter() {
                    f(Node::FunctionLikeParameterList(item));
                }
                f(Node::PropertyHookBody(&node.body));
            }
            Node::PropertyHookAbstractBody(_) => {}
            Node::PropertyHookBody(node) => f(match node {
                PropertyHookBody::Abstract(node) => Node::PropertyHookAbstractBody(node),
                PropertyHookBody::Concrete(node) => Node::PropertyHookConcreteBody(node),
            }),
            Node::PropertyHookConcreteBody(node) => f(match node {
                PropertyHookConcreteBody::Expression(node) => Node::PropertyHookConcreteExpressionBody(node),
                PropertyHookConcreteBody::Block(node) => Node::Block(node),
            }),
            Node::PropertyHookConcreteExpressionBody(node) => f(Node::Expression(node.expression)),
            Node::PropertyHookList(node) => {
                for item in node.hooks.iter() {
                    f(Node::PropertyHook(item));
                }
            }
            Node::PropertyItem(node) => match node {
                PropertyItem::Abstract(node) => f(Node::PropertyAbstractItem(node)),
                PropertyItem::Concrete(node) => f(Node::PropertyConcreteItem(node)),
            },
            Node::TraitUse(node) => {
                f(Node::Keyword(&node.r#use));
                for item in node.trait_names.iter() {
                    f(Node::Identifier(item));
                }
                f(Node::TraitUseSpecification(&node.specification));
            }
            Node::TraitUseAbsoluteMethodReference(node) => {
                f(Node::Identifier(&node.trait_name));
                f(Node::LocalIdentifier(&node.method_name));
            }
            Node::TraitUseAbstractSpecification(node) => f(Node::Terminator(&node.0)),
            Node::TraitUseAdaptation(node) => match node {
                TraitUseAdaptation::Precedence(adaptation) => {
                    f(Node::TraitUseAbsoluteMethodReference(&adaptation.method_reference));
                    f(Node::Keyword(&adaptation.insteadof));

                    for item in adaptation.trait_names.iter() {
                        f(Node::Identifier(item));
                    }
                    f(Node::Terminator(&adaptation.terminator));
                }
                TraitUseAdaptation::Alias(adaptation) => {
                    f(Node::TraitUseMethodReference(&adaptation.method_reference));
                    f(Node::Keyword(&adaptation.r#as));

                    if let Some(visibility) = &adaptation.visibility {
                        f(Node::Modifier(visibility));
                    }

                    if let Some(alias) = &adaptation.alias {
                        f(Node::LocalIdentifier(alias));
                    }

                    f(Node::Terminator(&adaptation.terminator));
                }
            },
            Node::TraitUseAliasAdaptation(node) => {
                f(Node::TraitUseMethodReference(&node.method_reference));
                f(Node::Keyword(&node.r#as));

                if let Some(visibility) = &node.visibility {
                    f(Node::Modifier(visibility));
                }

                if let Some(alias) = &node.alias {
                    f(Node::LocalIdentifier(alias));
                }

                f(Node::Terminator(&node.terminator));
            }
            Node::TraitUseConcreteSpecification(node) => {
                for adaptation in node.adaptations.as_slice() {
                    f(Node::TraitUseAdaptation(adaptation));
                }
            }
            Node::TraitUseMethodReference(node) => match node {
                TraitUseMethodReference::Identifier(identifier) => {
                    f(Node::LocalIdentifier(identifier));
                }
                TraitUseMethodReference::Absolute(reference) => {
                    f(Node::TraitUseAbsoluteMethodReference(reference));
                }
            },
            Node::TraitUsePrecedenceAdaptation(node) => {
                f(Node::TraitUseAbsoluteMethodReference(&node.method_reference));
                f(Node::Keyword(&node.insteadof));

                for item in node.trait_names.iter() {
                    f(Node::Identifier(item));
                }
                f(Node::Terminator(&node.terminator));
            }
            Node::TraitUseSpecification(node) => match node {
                TraitUseSpecification::Abstract(specification) => {
                    f(Node::TraitUseAbstractSpecification(specification));
                }
                TraitUseSpecification::Concrete(specification) => {
                    f(Node::TraitUseConcreteSpecification(specification));
                }
            },
            Node::AnonymousClass(node) => {
                f(Node::Keyword(&node.new));
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                f(Node::Keyword(&node.class));
                if let Some(argument_list) = &node.argument_list {
                    f(Node::ArgumentList(argument_list));
                }
                for item in node.extends.iter() {
                    f(Node::Extends(item));
                }
                for item in node.implements.iter() {
                    f(Node::Implements(item));
                }
                for item in node.members.iter() {
                    f(Node::ClassLikeMember(item));
                }
            }
            Node::Class(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                f(Node::Keyword(&node.class));
                f(Node::LocalIdentifier(&node.name));
                for item in node.extends.iter() {
                    f(Node::Extends(item));
                }
                for item in node.implements.iter() {
                    f(Node::Implements(item));
                }
                for item in node.members.iter() {
                    f(Node::ClassLikeMember(item));
                }
            }
            Node::Enum(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                f(Node::Keyword(&node.r#enum));
                f(Node::LocalIdentifier(&node.name));
                for item in node.backing_type_hint.iter() {
                    f(Node::EnumBackingTypeHint(item));
                }
                for item in node.implements.iter() {
                    f(Node::Implements(item));
                }
                for item in node.members.iter() {
                    f(Node::ClassLikeMember(item));
                }
            }
            Node::EnumBackingTypeHint(node) => {
                f(Node::Hint(&node.hint));
            }
            Node::Interface(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                f(Node::Keyword(&node.interface));
                f(Node::LocalIdentifier(&node.name));
                for item in node.extends.iter() {
                    f(Node::Extends(item));
                }
                for item in node.members.iter() {
                    f(Node::ClassLikeMember(item));
                }
            }
            Node::Trait(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                f(Node::Keyword(&node.r#trait));
                f(Node::LocalIdentifier(&node.name));
                for item in node.members.iter() {
                    f(Node::ClassLikeMember(item));
                }
            }
            Node::Clone(node) => {
                f(Node::Keyword(&node.clone));
                f(Node::Expression(node.object));
            }
            Node::Constant(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                f(Node::Keyword(&node.r#const));
                for item in node.items.iter() {
                    f(Node::ConstantItem(item));
                }
                f(Node::Terminator(&node.terminator));
            }
            Node::ConstantItem(node) => {
                f(Node::LocalIdentifier(&node.name));
                f(Node::Expression(node.value));
            }
            Node::Construct(node) => f(match node {
                Construct::Isset(node) => Node::IssetConstruct(node),
                Construct::Empty(node) => Node::EmptyConstruct(node),
                Construct::Eval(node) => Node::EvalConstruct(node),
                Construct::Include(node) => Node::IncludeConstruct(node),
                Construct::IncludeOnce(node) => Node::IncludeOnceConstruct(node),
                Construct::Require(node) => Node::RequireConstruct(node),
                Construct::RequireOnce(node) => Node::RequireOnceConstruct(node),
                Construct::Print(node) => Node::PrintConstruct(node),
                Construct::Exit(node) => Node::ExitConstruct(node),
                Construct::Die(node) => Node::DieConstruct(node),
            }),
            Node::IssetConstruct(node) => {
                f(Node::Keyword(&node.isset));
                for e in node.values.iter() {
                    f(Node::Expression(e));
                }
            }
            Node::EmptyConstruct(node) => {
                f(Node::Keyword(&node.empty));
                f(Node::Expression(node.value));
            }
            Node::EvalConstruct(node) => {
                f(Node::Keyword(&node.eval));
                f(Node::Expression(node.value));
            }
            Node::IncludeConstruct(node) => {
                f(Node::Keyword(&node.include));
                f(Node::Expression(node.value));
            }
            Node::IncludeOnceConstruct(node) => {
                f(Node::Keyword(&node.include_once));
                f(Node::Expression(node.value));
            }
            Node::RequireConstruct(node) => {
                f(Node::Keyword(&node.require));
                f(Node::Expression(node.value));
            }
            Node::RequireOnceConstruct(node) => {
                f(Node::Keyword(&node.require_once));
                f(Node::Expression(node.value));
            }
            Node::PrintConstruct(node) => {
                f(Node::Keyword(&node.print));
                f(Node::Expression(node.value));
            }
            Node::ExitConstruct(node) => {
                f(Node::Keyword(&node.exit));
                if let Some(arguments) = &node.arguments {
                    f(Node::ArgumentList(arguments));
                }
            }
            Node::DieConstruct(node) => {
                f(Node::Keyword(&node.die));
                if let Some(arguments) = &node.arguments {
                    f(Node::ArgumentList(arguments));
                }
            }
            Node::If(node) => {
                f(Node::Keyword(&node.r#if));
                f(Node::Expression(node.condition));
                f(Node::IfBody(&node.body));
            }
            Node::IfBody(node) => match node {
                IfBody::Statement(statement_body) => f(Node::IfStatementBody(statement_body)),
                IfBody::ColonDelimited(colon_body) => f(Node::IfColonDelimitedBody(colon_body)),
            },
            Node::IfStatementBody(node) => {
                f(Node::Statement(node.statement));

                for item in node.else_if_clauses.iter() {
                    f(Node::IfStatementBodyElseIfClause(item));
                }
                if let Some(else_clause) = &node.else_clause {
                    f(Node::IfStatementBodyElseClause(else_clause));
                }
            }
            Node::IfStatementBodyElseIfClause(node) => {
                f(Node::Keyword(&node.elseif));
                f(Node::Expression(node.condition));
                f(Node::Statement(node.statement));
            }
            Node::IfStatementBodyElseClause(node) => {
                f(Node::Keyword(&node.r#else));
                f(Node::Statement(node.statement));
            }
            Node::IfColonDelimitedBody(node) => {
                for stmt in node.statements.as_slice() {
                    f(Node::Statement(stmt));
                }

                for item in node.else_if_clauses.iter() {
                    f(Node::IfColonDelimitedBodyElseIfClause(item));
                }

                if let Some(else_clause) = &node.else_clause {
                    f(Node::IfColonDelimitedBodyElseClause(else_clause));
                }

                f(Node::Keyword(&node.endif));
                f(Node::Terminator(&node.terminator));
            }
            Node::IfColonDelimitedBodyElseIfClause(node) => {
                f(Node::Keyword(&node.elseif));
                f(Node::Expression(node.condition));
                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
            }
            Node::IfColonDelimitedBodyElseClause(node) => {
                f(Node::Keyword(&node.r#else));

                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
            }
            Node::Match(node) => {
                f(Node::Keyword(&node.r#match));
                f(Node::Expression(node.expression));
                for item in node.arms.iter() {
                    f(Node::MatchArm(item));
                }
            }
            Node::MatchArm(node) => match node {
                MatchArm::Expression(expr_arm) => f(Node::MatchExpressionArm(expr_arm)),
                MatchArm::Default(default_arm) => f(Node::MatchDefaultArm(default_arm)),
            },
            Node::MatchExpressionArm(node) => {
                for e in node.conditions.iter() {
                    f(Node::Expression(e));
                }
                f(Node::Expression(node.expression));
            }
            Node::MatchDefaultArm(node) => {
                f(Node::Keyword(&node.default));
                f(Node::Expression(node.expression));
            }
            Node::Switch(node) => {
                f(Node::Keyword(&node.switch));
                f(Node::Expression(node.expression));
                f(Node::SwitchBody(&node.body));
            }
            Node::SwitchBody(node) => match node {
                SwitchBody::BraceDelimited(body) => f(Node::SwitchBraceDelimitedBody(body)),
                SwitchBody::ColonDelimited(body) => f(Node::SwitchColonDelimitedBody(body)),
            },
            Node::SwitchBraceDelimitedBody(node) => {
                if let Some(terminator) = &node.optional_terminator {
                    f(Node::Terminator(terminator));
                }

                for item in node.cases.iter() {
                    f(Node::SwitchCase(item));
                }
            }
            Node::SwitchColonDelimitedBody(node) => {
                if let Some(terminator) = &node.optional_terminator {
                    f(Node::Terminator(terminator));
                }

                for item in node.cases.iter() {
                    f(Node::SwitchCase(item));
                }
                f(Node::Keyword(&node.end_switch));
                f(Node::Terminator(&node.terminator));
            }
            Node::SwitchCase(node) => match node {
                SwitchCase::Expression(expression_case) => {
                    f(Node::SwitchExpressionCase(expression_case));
                }
                SwitchCase::Default(default_case) => f(Node::SwitchDefaultCase(default_case)),
            },
            Node::SwitchExpressionCase(node) => {
                f(Node::Keyword(&node.case));
                f(Node::Expression(node.expression));
                f(Node::SwitchCaseSeparator(&node.separator));

                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
            }
            Node::SwitchDefaultCase(node) => {
                f(Node::Keyword(&node.default));
                f(Node::SwitchCaseSeparator(&node.separator));
                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
            }
            Node::SwitchCaseSeparator(_) => {}
            Node::Declare(node) => {
                f(Node::Keyword(&node.declare));

                for item in node.items.iter() {
                    f(Node::DeclareItem(item));
                }
                f(Node::DeclareBody(&node.body));
            }
            Node::DeclareBody(node) => match node {
                DeclareBody::Statement(statement) => f(Node::Statement(statement)),
                DeclareBody::ColonDelimited(body) => f(Node::DeclareColonDelimitedBody(body)),
            },
            Node::DeclareColonDelimitedBody(node) => {
                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }

                f(Node::Keyword(&node.end_declare));
                f(Node::Terminator(&node.terminator));
            }
            Node::DeclareItem(node) => {
                f(Node::LocalIdentifier(&node.name));
                f(Node::Expression(node.value));
            }
            Node::EchoTag(node) => {
                for e in node.values.iter() {
                    f(Node::Expression(e));
                }
                f(Node::Terminator(&node.terminator));
            }
            Node::Echo(node) => {
                f(Node::Keyword(&node.echo));
                for e in node.values.iter() {
                    f(Node::Expression(e));
                }
                f(Node::Terminator(&node.terminator));
            }
            Node::Parenthesized(node) => f(Node::Expression(node.expression)),
            Node::Expression(node) => {
                let child = match node {
                    Expression::Binary(node) => Node::Binary(node),
                    Expression::UnaryPrefix(node) => Node::UnaryPrefix(node),
                    Expression::ConstantAccess(node) => Node::ConstantAccess(node),
                    Expression::UnaryPostfix(node) => Node::UnaryPostfix(node),
                    Expression::Parenthesized(node) => Node::Parenthesized(node),
                    Expression::Literal(node) => Node::Literal(node),
                    Expression::CompositeString(node) => Node::CompositeString(node),
                    Expression::Assignment(node) => Node::Assignment(node),
                    Expression::Conditional(node) => Node::Conditional(node),
                    Expression::Array(node) => Node::Array(node),
                    Expression::LegacyArray(node) => Node::LegacyArray(node),
                    Expression::List(node) => Node::List(node),
                    Expression::ArrayAccess(node) => Node::ArrayAccess(node),
                    Expression::ArrayAppend(node) => Node::ArrayAppend(node),
                    Expression::AnonymousClass(node) => Node::AnonymousClass(node),
                    Expression::Closure(node) => Node::Closure(node),
                    Expression::ArrowFunction(node) => Node::ArrowFunction(node),
                    Expression::Variable(node) => Node::Variable(node),
                    Expression::Identifier(node) => Node::Identifier(node),
                    Expression::Match(node) => Node::Match(node),
                    Expression::Yield(node) => Node::Yield(node),
                    Expression::Construct(node) => Node::Construct(node),
                    Expression::Throw(node) => Node::Throw(node),
                    Expression::Clone(node) => Node::Clone(node),
                    Expression::Call(node) => Node::Call(node),
                    Expression::PartialApplication(node) => Node::PartialApplication(node),
                    Expression::Access(node) => Node::Access(node),
                    Expression::Parent(node) => Node::Keyword(node),
                    Expression::Static(node) => Node::Keyword(node),
                    Expression::Self_(node) => Node::Keyword(node),
                    Expression::Instantiation(node) => Node::Instantiation(node),
                    Expression::MagicConstant(node) => Node::MagicConstant(node),
                    Expression::Pipe(node) => Node::Pipe(node),
                    Expression::Error(span) => Node::Error(*span),
                };
                f(child);
            }
            Node::Binary(node) => {
                f(Node::Expression(node.lhs));
                f(Node::BinaryOperator(&node.operator));
                f(Node::Expression(node.rhs));
            }
            Node::BinaryOperator(operator) => match operator {
                BinaryOperator::Addition(_) => {}
                BinaryOperator::Subtraction(_) => {}
                BinaryOperator::Multiplication(_) => {}
                BinaryOperator::Division(_) => {}
                BinaryOperator::Modulo(_) => {}
                BinaryOperator::Exponentiation(_) => {}
                BinaryOperator::BitwiseAnd(_) => {}
                BinaryOperator::BitwiseOr(_) => {}
                BinaryOperator::BitwiseXor(_) => {}
                BinaryOperator::LeftShift(_) => {}
                BinaryOperator::RightShift(_) => {}
                BinaryOperator::NullCoalesce(_) => {}
                BinaryOperator::Equal(_) => {}
                BinaryOperator::NotEqual(_) => {}
                BinaryOperator::Identical(_) => {}
                BinaryOperator::NotIdentical(_) => {}
                BinaryOperator::AngledNotEqual(_) => {}
                BinaryOperator::LessThan(_) => {}
                BinaryOperator::LessThanOrEqual(_) => {}
                BinaryOperator::GreaterThan(_) => {}
                BinaryOperator::GreaterThanOrEqual(_) => {}
                BinaryOperator::Spaceship(_) => {}
                BinaryOperator::StringConcat(_) => {}
                BinaryOperator::And(_) => {}
                BinaryOperator::Or(_) => {}
                BinaryOperator::Instanceof(keyword) => f(Node::Keyword(keyword)),
                BinaryOperator::LowAnd(keyword) => f(Node::Keyword(keyword)),
                BinaryOperator::LowOr(keyword) => f(Node::Keyword(keyword)),
                BinaryOperator::LowXor(keyword) => f(Node::Keyword(keyword)),
            },
            Node::UnaryPrefix(node) => {
                f(Node::UnaryPrefixOperator(&node.operator));
                f(Node::Expression(node.operand));
            }
            Node::UnaryPostfix(node) => {
                f(Node::Expression(node.operand));
                f(Node::UnaryPostfixOperator(&node.operator));
            }
            Node::UnaryPrefixOperator(_) | Node::UnaryPostfixOperator(_) => {}
            Node::ArrowFunction(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                if let Some(r#static) = &node.r#static {
                    f(Node::Keyword(r#static));
                }
                f(Node::Keyword(&node.r#fn));
                f(Node::FunctionLikeParameterList(&node.parameter_list));
                if let Some(return_type_hint) = &node.return_type_hint {
                    f(Node::FunctionLikeReturnTypeHint(return_type_hint));
                }
                f(Node::Expression(node.expression));
            }
            Node::Closure(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                f(Node::Keyword(&node.function));
                f(Node::FunctionLikeParameterList(&node.parameter_list));
                if let Some(use_clause) = &node.use_clause {
                    f(Node::ClosureUseClause(use_clause));
                }
                if let Some(return_type_hint) = &node.return_type_hint {
                    f(Node::FunctionLikeReturnTypeHint(return_type_hint));
                }
                f(Node::Block(&node.body));
            }
            Node::ClosureUseClause(node) => {
                f(Node::Keyword(&node.r#use));
                for item in node.variables.iter() {
                    f(Node::ClosureUseClauseVariable(item));
                }
            }
            Node::ClosureUseClauseVariable(node) => f(Node::DirectVariable(&node.variable)),
            Node::Function(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                f(Node::Keyword(&node.function));
                f(Node::LocalIdentifier(&node.name));
                f(Node::FunctionLikeParameterList(&node.parameter_list));
                if let Some(return_type_hint) = &node.return_type_hint {
                    f(Node::FunctionLikeReturnTypeHint(return_type_hint));
                }

                f(Node::Block(&node.body));
            }
            Node::FunctionLikeParameterList(node) => {
                for item in node.parameters.iter() {
                    f(Node::FunctionLikeParameter(item));
                }
            }
            Node::FunctionLikeParameter(node) => {
                for item in node.attribute_lists.iter() {
                    f(Node::AttributeList(item));
                }
                for item in node.modifiers.iter() {
                    f(Node::Modifier(item));
                }
                if let Some(hint) = &node.hint {
                    f(Node::Hint(hint));
                }
                f(Node::DirectVariable(&node.variable));
                if let Some(default_value) = &node.default_value {
                    f(Node::FunctionLikeParameterDefaultValue(default_value));
                }

                if let Some(hooks) = &node.hooks {
                    f(Node::PropertyHookList(hooks));
                }
            }
            Node::FunctionLikeParameterDefaultValue(node) => f(Node::Expression(node.value)),
            Node::FunctionLikeReturnTypeHint(hint) => f(Node::Hint(&hint.hint)),
            Node::Global(node) => {
                f(Node::Keyword(&node.r#global));
                for item in node.variables.iter() {
                    f(Node::Variable(item));
                }
            }
            Node::Goto(node) => {
                f(Node::Keyword(&node.r#goto));
                f(Node::LocalIdentifier(&node.label));
            }
            Node::Label(node) => {
                f(Node::LocalIdentifier(&node.name));
            }
            Node::HaltCompiler(node) => {
                f(Node::Keyword(&node.halt_compiler));
            }
            Node::FullyQualifiedIdentifier(_) => {}
            Node::Identifier(node) => f(match node {
                Identifier::Local(node) => Node::LocalIdentifier(node),
                Identifier::Qualified(node) => Node::QualifiedIdentifier(node),
                Identifier::FullyQualified(node) => Node::FullyQualifiedIdentifier(node),
            }),
            Node::LocalIdentifier(_) => {}
            Node::QualifiedIdentifier(_) => {}
            Node::Inline(_) => {}
            Node::Instantiation(node) => {
                f(Node::Keyword(&node.new));
                f(Node::Expression(node.class));

                if let Some(argument_list) = &node.argument_list {
                    f(Node::ArgumentList(argument_list));
                }
            }
            Node::Keyword(_) => {}
            Node::Literal(node) => f(match node {
                Literal::Float(node) => Node::LiteralFloat(node),
                Literal::Integer(node) => Node::LiteralInteger(node),
                Literal::String(node) => Node::LiteralString(node),
                Literal::True(node) => Node::Keyword(node),
                Literal::False(node) => Node::Keyword(node),
                Literal::Null(node) => Node::Keyword(node),
            }),
            Node::LiteralFloat(_) => {}
            Node::LiteralInteger(_) => {}
            Node::LiteralString(_) => {}
            Node::MagicConstant(node) => f(match node {
                MagicConstant::Class(node) => Node::LocalIdentifier(node),
                MagicConstant::Directory(node) => Node::LocalIdentifier(node),
                MagicConstant::File(node) => Node::LocalIdentifier(node),
                MagicConstant::Function(node) => Node::LocalIdentifier(node),
                MagicConstant::Line(node) => Node::LocalIdentifier(node),
                MagicConstant::Method(node) => Node::LocalIdentifier(node),
                MagicConstant::Namespace(node) => Node::LocalIdentifier(node),
                MagicConstant::Trait(node) => Node::LocalIdentifier(node),
                MagicConstant::Property(node) => Node::LocalIdentifier(node),
            }),
            Node::Modifier(node) => f(match node {
                Modifier::Abstract(node) => Node::Keyword(node),
                Modifier::Final(node) => Node::Keyword(node),
                Modifier::Private(node) => Node::Keyword(node),
                Modifier::Protected(node) => Node::Keyword(node),
                Modifier::Public(node) => Node::Keyword(node),
                Modifier::Static(node) => Node::Keyword(node),
                Modifier::Readonly(node) => Node::Keyword(node),
                Modifier::PrivateSet(node) => Node::Keyword(node),
                Modifier::ProtectedSet(node) => Node::Keyword(node),
                Modifier::PublicSet(node) => Node::Keyword(node),
            }),
            Node::Namespace(node) => {
                f(Node::Keyword(&node.r#namespace));

                if let Some(name) = &node.name {
                    f(Node::Identifier(name));
                }

                f(Node::NamespaceBody(&node.body));
            }
            Node::NamespaceBody(node) => {
                f(match node {
                    NamespaceBody::BraceDelimited(node) => Node::Block(node),
                    NamespaceBody::Implicit(node) => Node::NamespaceImplicitBody(node),
                });
            }
            Node::NamespaceImplicitBody(node) => {
                f(Node::Terminator(&node.terminator));

                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
            }
            Node::Assignment(node) => {
                f(Node::Expression(node.lhs));
                f(Node::AssignmentOperator(&node.operator));
                f(Node::Expression(node.rhs));
            }
            Node::AssignmentOperator(_) => {}
            Node::Conditional(node) => {
                f(Node::Expression(node.condition));

                if let Some(then) = &node.then {
                    f(Node::Expression(then));
                }

                f(Node::Expression(node.r#else));
            }
            Node::DoWhile(node) => {
                f(Node::Keyword(&node.r#do));
                f(Node::Statement(node.statement));
                f(Node::Keyword(&node.r#while));
                f(Node::Expression(node.condition));
                f(Node::Terminator(&node.terminator));
            }
            Node::Foreach(node) => {
                f(Node::Keyword(&node.r#foreach));
                f(Node::Expression(node.expression));
                f(Node::Keyword(&node.r#as));
                f(Node::ForeachTarget(&node.target));
                f(Node::ForeachBody(&node.body));
            }
            Node::ForeachBody(node) => f(match node {
                ForeachBody::Statement(node) => Node::Statement(node),
                ForeachBody::ColonDelimited(node) => Node::ForeachColonDelimitedBody(node),
            }),
            Node::ForeachColonDelimitedBody(node) => {
                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }

                f(Node::Keyword(&node.end_foreach));
                f(Node::Terminator(&node.terminator));
            }
            Node::ForeachKeyValueTarget(node) => {
                f(Node::Expression(node.key));
                f(Node::Expression(node.value));
            }
            Node::ForeachTarget(node) => f(match node {
                ForeachTarget::KeyValue(node) => Node::ForeachKeyValueTarget(node),
                ForeachTarget::Value(node) => Node::ForeachValueTarget(node),
            }),
            Node::ForeachValueTarget(node) => f(Node::Expression(node.value)),
            Node::For(node) => {
                f(Node::Keyword(&node.r#for));

                for e in node.initializations.iter() {
                    f(Node::Expression(e));
                }
                for e in node.conditions.iter() {
                    f(Node::Expression(e));
                }
                for e in node.increments.iter() {
                    f(Node::Expression(e));
                }
                f(Node::ForBody(&node.body));
            }
            Node::ForBody(node) => match node {
                ForBody::Statement(statement) => f(Node::Statement(statement)),
                ForBody::ColonDelimited(body) => f(Node::ForColonDelimitedBody(body)),
            },
            Node::ForColonDelimitedBody(node) => {
                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
                f(Node::Keyword(&node.end_for));
                f(Node::Terminator(&node.terminator));
            }
            Node::While(node) => {
                f(Node::Keyword(&node.r#while));
                f(Node::Expression(node.condition));
                f(Node::WhileBody(&node.body));
            }
            Node::WhileBody(node) => match node {
                WhileBody::Statement(statement) => f(Node::Statement(statement)),
                WhileBody::ColonDelimited(body) => f(Node::WhileColonDelimitedBody(body)),
            },
            Node::WhileColonDelimitedBody(node) => {
                for item in node.statements.iter() {
                    f(Node::Statement(item));
                }
                f(Node::Keyword(&node.end_while));
                f(Node::Terminator(&node.terminator));
            }
            Node::Break(node) => {
                f(Node::Keyword(&node.r#break));

                if let Some(level) = &node.level {
                    f(Node::Expression(level));
                }

                f(Node::Terminator(&node.terminator));
            }
            Node::Continue(node) => {
                f(Node::Keyword(&node.r#continue));

                if let Some(level) = &node.level {
                    f(Node::Expression(level));
                }

                f(Node::Terminator(&node.terminator));
            }
            Node::Return(node) => {
                f(Node::Keyword(&node.r#return));

                if let Some(value) = &node.value {
                    f(Node::Expression(value));
                }

                f(Node::Terminator(&node.terminator));
            }
            Node::Static(node) => {
                f(Node::Keyword(&node.r#static));

                for item in node.items.iter() {
                    f(Node::StaticItem(item));
                }
                f(Node::Terminator(&node.terminator));
            }
            Node::StaticItem(node) => f(match node {
                StaticItem::Abstract(item) => Node::StaticAbstractItem(item),
                StaticItem::Concrete(item) => Node::StaticConcreteItem(item),
            }),
            Node::StaticAbstractItem(node) => {
                f(Node::DirectVariable(&node.variable));
            }
            Node::StaticConcreteItem(node) => {
                f(Node::DirectVariable(&node.variable));
                f(Node::Expression(node.value));
            }
            Node::Try(node) => {
                f(Node::Keyword(&node.r#try));
                f(Node::Block(&node.block));
                for item in node.catch_clauses.iter() {
                    f(Node::TryCatchClause(item));
                }
                if let Some(finally) = &node.finally_clause {
                    f(Node::TryFinallyClause(finally));
                }
            }
            Node::TryCatchClause(node) => {
                f(Node::Keyword(&node.r#catch));
                f(Node::Hint(&node.hint));
                if let Some(variable) = &node.variable {
                    f(Node::DirectVariable(variable));
                }
                f(Node::Block(&node.block));
            }
            Node::TryFinallyClause(node) => {
                f(Node::Keyword(&node.r#finally));
                f(Node::Block(&node.block));
            }
            Node::MaybeTypedUseItem(node) => {
                if let Some(r#type) = &node.r#type {
                    f(Node::UseType(r#type));
                }

                f(Node::UseItem(&node.item));
            }
            Node::MixedUseItemList(node) => {
                f(Node::Identifier(&node.namespace));

                for item in node.items.iter() {
                    f(Node::MaybeTypedUseItem(item));
                }
            }
            Node::TypedUseItemList(node) => {
                f(Node::UseType(&node.r#type));
                f(Node::Identifier(&node.namespace));

                for item in node.items.iter() {
                    f(Node::UseItem(item));
                }
            }
            Node::TypedUseItemSequence(node) => {
                f(Node::UseType(&node.r#type));

                for item in node.items.iter() {
                    f(Node::UseItem(item));
                }
            }
            Node::Use(node) => {
                f(Node::Keyword(&node.r#use));
                f(Node::UseItems(&node.items));
                f(Node::Terminator(&node.terminator));
            }
            Node::UseItem(node) => {
                f(Node::Identifier(&node.name));

                if let Some(alias) = &node.alias {
                    f(Node::UseItemAlias(alias));
                }
            }
            Node::UseItemAlias(node) => {
                f(Node::Keyword(&node.r#as));
                f(Node::LocalIdentifier(&node.identifier));
            }
            Node::UseItemSequence(node) => {
                for item in &node.items {
                    f(Node::UseItem(item));
                }
            }
            Node::UseItems(node) => f(match node {
                UseItems::Sequence(node) => Node::UseItemSequence(node),
                UseItems::TypedList(node) => Node::TypedUseItemList(node),
                UseItems::MixedList(node) => Node::MixedUseItemList(node),
                UseItems::TypedSequence(node) => Node::TypedUseItemSequence(node),
            }),
            Node::UseType(node) => f(match node {
                UseType::Const(node) => Node::Keyword(node),
                UseType::Function(node) => Node::Keyword(node),
            }),
            Node::Yield(node) => f(match node {
                Yield::Value(node) => Node::YieldValue(node),
                Yield::Pair(node) => Node::YieldPair(node),
                Yield::From(node) => Node::YieldFrom(node),
            }),
            Node::YieldFrom(node) => {
                f(Node::Keyword(&node.r#yield));
                f(Node::Keyword(&node.from));
                f(Node::Expression(node.iterator));
            }
            Node::YieldPair(node) => {
                f(Node::Keyword(&node.r#yield));
                f(Node::Expression(node.key));
                f(Node::Expression(node.value));
            }
            Node::YieldValue(node) => {
                f(Node::Keyword(&node.r#yield));
                if let Some(value) = &node.value {
                    f(Node::Expression(value));
                }
            }
            Node::Statement(node) => match &node {
                Statement::OpeningTag(node) => f(Node::OpeningTag(node)),
                Statement::ClosingTag(node) => f(Node::ClosingTag(node)),
                Statement::Inline(node) => f(Node::Inline(node)),
                Statement::Namespace(node) => f(Node::Namespace(node)),
                Statement::Use(node) => f(Node::Use(node)),
                Statement::Class(node) => f(Node::Class(node)),
                Statement::Interface(node) => f(Node::Interface(node)),
                Statement::Trait(node) => f(Node::Trait(node)),
                Statement::Enum(node) => f(Node::Enum(node)),
                Statement::Block(node) => f(Node::Block(node)),
                Statement::Constant(node) => f(Node::Constant(node)),
                Statement::Function(node) => f(Node::Function(node)),
                Statement::Declare(node) => f(Node::Declare(node)),
                Statement::Goto(node) => f(Node::Goto(node)),
                Statement::Label(node) => f(Node::Label(node)),
                Statement::Try(node) => f(Node::Try(node)),
                Statement::Foreach(node) => f(Node::Foreach(node)),
                Statement::For(node) => f(Node::For(node)),
                Statement::While(node) => f(Node::While(node)),
                Statement::DoWhile(node) => f(Node::DoWhile(node)),
                Statement::Continue(node) => f(Node::Continue(node)),
                Statement::Break(node) => f(Node::Break(node)),
                Statement::Switch(node) => f(Node::Switch(node)),
                Statement::If(node) => f(Node::If(node)),
                Statement::Return(node) => f(Node::Return(node)),
                Statement::Expression(node) => f(Node::ExpressionStatement(node)),
                Statement::EchoTag(node) => f(Node::EchoTag(node)),
                Statement::Echo(node) => f(Node::Echo(node)),
                Statement::Global(node) => f(Node::Global(node)),
                Statement::Static(node) => f(Node::Static(node)),
                Statement::HaltCompiler(node) => f(Node::HaltCompiler(node)),
                Statement::Unset(node) => f(Node::Unset(node)),
                Statement::Noop(_) => {}
            },
            Node::ExpressionStatement(node) => {
                f(Node::Expression(node.expression));
                f(Node::Terminator(&node.terminator));
            }
            Node::BracedExpressionStringPart(node) => f(Node::Expression(node.expression)),
            Node::DocumentString(node) => {
                for part in node.parts.as_slice() {
                    f(Node::StringPart(part));
                }
            }
            Node::InterpolatedString(node) => {
                for part in node.parts.as_slice() {
                    f(Node::StringPart(part));
                }
            }
            Node::LiteralStringPart(_) => {}
            Node::ShellExecuteString(node) => {
                for part in node.parts.as_slice() {
                    f(Node::StringPart(part));
                }
            }
            Node::CompositeString(node) => f(match node {
                CompositeString::ShellExecute(node) => Node::ShellExecuteString(node),
                CompositeString::Interpolated(node) => Node::InterpolatedString(node),
                CompositeString::Document(node) => Node::DocumentString(node),
            }),
            Node::StringPart(node) => f(match node {
                StringPart::Literal(node) => Node::LiteralStringPart(node),
                StringPart::Expression(node) => Node::Expression(node),
                StringPart::BracedExpression(node) => Node::BracedExpressionStringPart(node),
            }),
            Node::ClosingTag(_) => {}
            Node::FullOpeningTag(_) => {}
            Node::OpeningTag(node) => match node {
                OpeningTag::Full(node) => f(Node::FullOpeningTag(node)),
                OpeningTag::Short(node) => f(Node::ShortOpeningTag(node)),
            },
            Node::ShortOpeningTag(_) => {}
            Node::Terminator(node) => match node {
                Terminator::Semicolon(_) => {}
                Terminator::ClosingTag(closing_tag) => f(Node::ClosingTag(closing_tag)),
                Terminator::TagPair(closing_tag, opening_tag) => {
                    f(Node::ClosingTag(closing_tag));
                    f(Node::OpeningTag(opening_tag));
                }
                Terminator::Missing(span) => f(Node::MissingTerminator(*span)),
            },
            Node::Throw(node) => {
                f(Node::Keyword(&node.throw));
                f(Node::Expression(node.exception));
            }
            Node::Hint(node) => match &node {
                Hint::Identifier(identifier) => f(Node::Identifier(identifier)),
                Hint::Parenthesized(parenthesized_hint) => {
                    f(Node::ParenthesizedHint(parenthesized_hint));
                }
                Hint::Nullable(nullable_hint) => f(Node::NullableHint(nullable_hint)),
                Hint::Union(union_hint) => f(Node::UnionHint(union_hint)),
                Hint::Intersection(intersection_hint) => f(Node::IntersectionHint(intersection_hint)),
                Hint::Null(keyword)
                | Hint::True(keyword)
                | Hint::False(keyword)
                | Hint::Array(keyword)
                | Hint::Callable(keyword)
                | Hint::Static(keyword)
                | Hint::Self_(keyword)
                | Hint::Parent(keyword) => f(Node::Keyword(keyword)),
                Hint::Void(local_identifier)
                | Hint::Never(local_identifier)
                | Hint::Float(local_identifier)
                | Hint::Bool(local_identifier)
                | Hint::Integer(local_identifier)
                | Hint::String(local_identifier)
                | Hint::Object(local_identifier)
                | Hint::Mixed(local_identifier)
                | Hint::Iterable(local_identifier) => f(Node::LocalIdentifier(local_identifier)),
            },
            Node::IntersectionHint(node) => {
                f(Node::Hint(node.left));
                f(Node::Hint(node.right));
            }
            Node::NullableHint(node) => f(Node::Hint(node.hint)),
            Node::ParenthesizedHint(node) => f(Node::Hint(node.hint)),
            Node::UnionHint(node) => {
                f(Node::Hint(node.left));
                f(Node::Hint(node.right));
            }
            Node::Unset(node) => {
                f(Node::Keyword(&node.unset));
                for e in node.values.iter() {
                    f(Node::Expression(e));
                }
                f(Node::Terminator(&node.terminator));
            }
            Node::DirectVariable(_) => {}
            Node::IndirectVariable(node) => f(Node::Expression(node.expression)),
            Node::NestedVariable(node) => {
                f(Node::Variable(node.variable));
            }
            Node::Variable(node) => match node {
                Variable::Direct(node) => f(Node::DirectVariable(node)),
                Variable::Indirect(node) => f(Node::IndirectVariable(node)),
                Variable::Nested(node) => f(Node::NestedVariable(node)),
            },
            Node::Pipe(pipe) => {
                f(Node::Expression(pipe.input));
                f(Node::Expression(pipe.callable));
            }
            Node::Error(_)
            | Node::MissingTerminator(_)
            | Node::ClassLikeMemberMissingSelector(_)
            | Node::ClassLikeConstantMissingSelector(_) => {}
        }
    }

    /// Returns all direct children as an owned `Vec`.
    ///
    /// This allocates on every call. Prefer [`Self::visit_children`] when you don't need a
    /// collected list — for example, when searching for a node or applying a transformation.
    #[inline]
    pub fn children(&self) -> Vec<Node<'ast, 'arena>> {
        let mut children = vec![];
        self.visit_children(|child| children.push(child));
        children
    }
}

impl HasSpan for Node<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::Program(node) => node.span(),
            Self::Access(node) => node.span(),
            Self::ConstantAccess(node) => node.span(),
            Self::ClassConstantAccess(node) => node.span(),
            Self::NullSafePropertyAccess(node) => node.span(),
            Self::PropertyAccess(node) => node.span(),
            Self::StaticPropertyAccess(node) => node.span(),
            Self::Argument(node) => node.span(),
            Self::ArgumentList(node) => node.span(),
            Self::PartialArgument(node) => node.span(),
            Self::PartialArgumentList(node) => node.span(),
            Self::NamedArgument(node) => node.span(),
            Self::NamedPlaceholderArgument(node) => node.span(),
            Self::PlaceholderArgument(node) => node.span(),
            Self::PositionalArgument(node) => node.span(),
            Self::VariadicPlaceholderArgument(node) => node.span(),
            Self::Array(node) => node.span(),
            Self::ArrayAccess(node) => node.span(),
            Self::ArrayAppend(node) => node.span(),
            Self::ArrayElement(node) => node.span(),
            Self::KeyValueArrayElement(node) => node.span(),
            Self::LegacyArray(node) => node.span(),
            Self::List(node) => node.span(),
            Self::MissingArrayElement(node) => node.span(),
            Self::ValueArrayElement(node) => node.span(),
            Self::VariadicArrayElement(node) => node.span(),
            Self::Attribute(node) => node.span(),
            Self::AttributeList(node) => node.span(),
            Self::Block(node) => node.span(),
            Self::Call(node) => node.span(),
            Self::FunctionCall(node) => node.span(),
            Self::MethodCall(node) => node.span(),
            Self::NullSafeMethodCall(node) => node.span(),
            Self::StaticMethodCall(node) => node.span(),
            Self::PartialApplication(node) => node.span(),
            Self::FunctionPartialApplication(node) => node.span(),
            Self::MethodPartialApplication(node) => node.span(),
            Self::StaticMethodPartialApplication(node) => node.span(),
            Self::ClassLikeConstant(node) => node.span(),
            Self::ClassLikeConstantItem(node) => node.span(),
            Self::EnumCase(node) => node.span(),
            Self::EnumCaseBackedItem(node) => node.span(),
            Self::EnumCaseItem(node) => node.span(),
            Self::EnumCaseUnitItem(node) => node.span(),
            Self::Extends(node) => node.span(),
            Self::Implements(node) => node.span(),
            Self::ClassLikeConstantSelector(node) => node.span(),
            Self::ClassLikeMember(node) => node.span(),
            Self::ClassLikeMemberExpressionSelector(node) => node.span(),
            Self::ClassLikeMemberSelector(node) => node.span(),
            Self::Method(node) => node.span(),
            Self::MethodAbstractBody(node) => node.span(),
            Self::MethodBody(node) => node.span(),
            Self::HookedProperty(node) => node.span(),
            Self::PlainProperty(node) => node.span(),
            Self::Property(node) => node.span(),
            Self::PropertyAbstractItem(node) => node.span(),
            Self::PropertyConcreteItem(node) => node.span(),
            Self::PropertyHook(node) => node.span(),
            Self::PropertyHookAbstractBody(node) => node.span(),
            Self::PropertyHookBody(node) => node.span(),
            Self::PropertyHookConcreteBody(node) => node.span(),
            Self::PropertyHookConcreteExpressionBody(node) => node.span(),
            Self::PropertyHookList(node) => node.span(),
            Self::PropertyItem(node) => node.span(),
            Self::TraitUse(node) => node.span(),
            Self::TraitUseAbsoluteMethodReference(node) => node.span(),
            Self::TraitUseAbstractSpecification(node) => node.span(),
            Self::TraitUseAdaptation(node) => node.span(),
            Self::TraitUseAliasAdaptation(node) => node.span(),
            Self::TraitUseConcreteSpecification(node) => node.span(),
            Self::TraitUseMethodReference(node) => node.span(),
            Self::TraitUsePrecedenceAdaptation(node) => node.span(),
            Self::TraitUseSpecification(node) => node.span(),
            Self::AnonymousClass(node) => node.span(),
            Self::Class(node) => node.span(),
            Self::Enum(node) => node.span(),
            Self::EnumBackingTypeHint(node) => node.span(),
            Self::Interface(node) => node.span(),
            Self::Trait(node) => node.span(),
            Self::Clone(node) => node.span(),
            Self::Constant(node) => node.span(),
            Self::ConstantItem(node) => node.span(),
            Self::Construct(node) => node.span(),
            Self::DieConstruct(node) => node.span(),
            Self::EmptyConstruct(node) => node.span(),
            Self::EvalConstruct(node) => node.span(),
            Self::ExitConstruct(node) => node.span(),
            Self::IncludeConstruct(node) => node.span(),
            Self::IncludeOnceConstruct(node) => node.span(),
            Self::IssetConstruct(node) => node.span(),
            Self::PrintConstruct(node) => node.span(),
            Self::RequireConstruct(node) => node.span(),
            Self::RequireOnceConstruct(node) => node.span(),
            Self::If(node) => node.span(),
            Self::IfBody(node) => node.span(),
            Self::IfColonDelimitedBody(node) => node.span(),
            Self::IfColonDelimitedBodyElseClause(node) => node.span(),
            Self::IfColonDelimitedBodyElseIfClause(node) => node.span(),
            Self::IfStatementBody(node) => node.span(),
            Self::IfStatementBodyElseClause(node) => node.span(),
            Self::IfStatementBodyElseIfClause(node) => node.span(),
            Self::Match(node) => node.span(),
            Self::MatchArm(node) => node.span(),
            Self::MatchDefaultArm(node) => node.span(),
            Self::MatchExpressionArm(node) => node.span(),
            Self::Switch(node) => node.span(),
            Self::SwitchBody(node) => node.span(),
            Self::SwitchBraceDelimitedBody(node) => node.span(),
            Self::SwitchCase(node) => node.span(),
            Self::SwitchCaseSeparator(node) => node.span(),
            Self::SwitchColonDelimitedBody(node) => node.span(),
            Self::SwitchDefaultCase(node) => node.span(),
            Self::SwitchExpressionCase(node) => node.span(),
            Self::Declare(node) => node.span(),
            Self::DeclareBody(node) => node.span(),
            Self::DeclareColonDelimitedBody(node) => node.span(),
            Self::DeclareItem(node) => node.span(),
            Self::Echo(node) => node.span(),
            Self::Expression(node) => node.span(),
            Self::Binary(node) => node.span(),
            Self::BinaryOperator(node) => node.span(),
            Self::UnaryPrefix(node) => node.span(),
            Self::UnaryPrefixOperator(node) => node.span(),
            Self::UnaryPostfix(node) => node.span(),
            Self::UnaryPostfixOperator(node) => node.span(),
            Self::Parenthesized(node) => node.span(),
            Self::ArrowFunction(node) => node.span(),
            Self::Closure(node) => node.span(),
            Self::ClosureUseClause(node) => node.span(),
            Self::ClosureUseClauseVariable(node) => node.span(),
            Self::Function(node) => node.span(),
            Self::FunctionLikeParameter(node) => node.span(),
            Self::FunctionLikeParameterDefaultValue(node) => node.span(),
            Self::FunctionLikeParameterList(node) => node.span(),
            Self::FunctionLikeReturnTypeHint(node) => node.span(),
            Self::Global(node) => node.span(),
            Self::Goto(node) => node.span(),
            Self::Label(node) => node.span(),
            Self::HaltCompiler(node) => node.span(),
            Self::FullyQualifiedIdentifier(node) => node.span(),
            Self::Identifier(node) => node.span(),
            Self::LocalIdentifier(node) => node.span(),
            Self::QualifiedIdentifier(node) => node.span(),
            Self::Inline(node) => node.span(),
            Self::Instantiation(node) => node.span(),
            Self::Keyword(node) => node.span(),
            Self::Literal(node) => node.span(),
            Self::LiteralFloat(node) => node.span(),
            Self::LiteralInteger(node) => node.span(),
            Self::LiteralString(node) => node.span(),
            Self::MagicConstant(node) => node.span(),
            Self::Modifier(node) => node.span(),
            Self::Namespace(node) => node.span(),
            Self::NamespaceBody(node) => node.span(),
            Self::NamespaceImplicitBody(node) => node.span(),
            Self::Assignment(node) => node.span(),
            Self::AssignmentOperator(node) => node.span(),
            Self::Conditional(node) => node.span(),
            Self::DoWhile(node) => node.span(),
            Self::Foreach(node) => node.span(),
            Self::ForeachBody(node) => node.span(),
            Self::ForeachColonDelimitedBody(node) => node.span(),
            Self::ForeachKeyValueTarget(node) => node.span(),
            Self::ForeachTarget(node) => node.span(),
            Self::ForeachValueTarget(node) => node.span(),
            Self::For(node) => node.span(),
            Self::ForBody(node) => node.span(),
            Self::ForColonDelimitedBody(node) => node.span(),
            Self::While(node) => node.span(),
            Self::WhileBody(node) => node.span(),
            Self::WhileColonDelimitedBody(node) => node.span(),
            Self::Break(node) => node.span(),
            Self::Continue(node) => node.span(),
            Self::Return(node) => node.span(),
            Self::Static(node) => node.span(),
            Self::StaticAbstractItem(node) => node.span(),
            Self::StaticConcreteItem(node) => node.span(),
            Self::StaticItem(node) => node.span(),
            Self::Try(node) => node.span(),
            Self::TryCatchClause(node) => node.span(),
            Self::TryFinallyClause(node) => node.span(),
            Self::MaybeTypedUseItem(node) => node.span(),
            Self::MixedUseItemList(node) => node.span(),
            Self::TypedUseItemList(node) => node.span(),
            Self::TypedUseItemSequence(node) => node.span(),
            Self::Use(node) => node.span(),
            Self::UseItem(node) => node.span(),
            Self::UseItemAlias(node) => node.span(),
            Self::UseItemSequence(node) => node.span(),
            Self::UseItems(node) => node.span(),
            Self::UseType(node) => node.span(),
            Self::Yield(node) => node.span(),
            Self::YieldFrom(node) => node.span(),
            Self::YieldPair(node) => node.span(),
            Self::YieldValue(node) => node.span(),
            Self::Statement(node) => node.span(),
            Self::ExpressionStatement(node) => node.span(),
            Self::BracedExpressionStringPart(node) => node.span(),
            Self::DocumentString(node) => node.span(),
            Self::InterpolatedString(node) => node.span(),
            Self::LiteralStringPart(node) => node.span(),
            Self::ShellExecuteString(node) => node.span(),
            Self::CompositeString(node) => node.span(),
            Self::StringPart(node) => node.span(),
            Self::ClosingTag(node) => node.span(),
            Self::EchoTag(node) => node.span(),
            Self::FullOpeningTag(node) => node.span(),
            Self::OpeningTag(node) => node.span(),
            Self::ShortOpeningTag(node) => node.span(),
            Self::Terminator(node) => node.span(),
            Self::Throw(node) => node.span(),
            Self::Hint(node) => node.span(),
            Self::IntersectionHint(node) => node.span(),
            Self::NullableHint(node) => node.span(),
            Self::ParenthesizedHint(node) => node.span(),
            Self::UnionHint(node) => node.span(),
            Self::Unset(node) => node.span(),
            Self::DirectVariable(node) => node.span(),
            Self::IndirectVariable(node) => node.span(),
            Self::NestedVariable(node) => node.span(),
            Self::Variable(node) => node.span(),
            Self::Pipe(node) => node.span(),
            Self::Error(span)
            | Self::MissingTerminator(span)
            | Self::ClassLikeMemberMissingSelector(span)
            | Self::ClassLikeConstantMissingSelector(span) => *span,
        }
    }
}
