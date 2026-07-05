use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::IR;
use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::error::Error;
use crate::ir::error::annotation::AnnotationError;
use crate::ir::expression::Access;
use crate::ir::expression::ArrayElement;
use crate::ir::expression::ArrayLike;
use crate::ir::expression::Assignment;
use crate::ir::expression::Binary;
use crate::ir::expression::Call;
use crate::ir::expression::Callee;
use crate::ir::expression::CompositeString;
use crate::ir::expression::CompositeStringPart;
use crate::ir::expression::Conditional;
use crate::ir::expression::Expression;
use crate::ir::expression::Instantiation;
use crate::ir::expression::Match;
use crate::ir::expression::MatchArm;
use crate::ir::expression::PartialApplication;
use crate::ir::expression::UnaryPostfix;
use crate::ir::expression::UnaryPrefix;
use crate::ir::expression::Yield;
use crate::ir::expression::operator::AssignmentOperator;
use crate::ir::expression::operator::BinaryOperator;
use crate::ir::expression::operator::UnaryPostfixOperator;
use crate::ir::expression::operator::UnaryPrefixOperator;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::identifier::Identifier;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::annotation::alias::ImportedTypeAliasAnnotation;
use crate::ir::item::annotation::alias::TypeAliasAnnotation;
use crate::ir::item::annotation::effect::AssertAnnotation;
use crate::ir::item::annotation::effect::AssertAnnotationPattern;
use crate::ir::item::annotation::effect::AssertAnnotationTarget;
use crate::ir::item::annotation::effect::SelfOutAnnotation;
use crate::ir::item::annotation::effect::ThrowsAnnotation;
use crate::ir::item::annotation::generics::InheritedTypeParameterAnnotation;
use crate::ir::item::annotation::generics::TypeParameterAnnotation;
use crate::ir::item::annotation::generics::WhereConstraintAnnotation;
use crate::ir::item::annotation::inheritance::ExtendsAnnotation;
use crate::ir::item::annotation::inheritance::ImplementsAnnotation;
use crate::ir::item::annotation::inheritance::MixinAnnotation;
use crate::ir::item::annotation::inheritance::RequireExtendsAnnotation;
use crate::ir::item::annotation::inheritance::RequireImplementsAnnotation;
use crate::ir::item::annotation::inheritance::SealedAnnotation;
use crate::ir::item::annotation::inheritance::UseAnnotation;
use crate::ir::item::annotation::member::MethodAnnotation;
use crate::ir::item::annotation::member::PropertyAnnotation;
use crate::ir::item::annotation::parameter::ParameterAnnotation;
use crate::ir::item::annotation::parameter::ParameterOutAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::expression::anonymous_class::AnonymousClass;
use crate::ir::item::expression::arrow_function::ArrowFunction;
use crate::ir::item::expression::closure::Closure;
use crate::ir::item::expression::closure::ClosureUseClauseVariable;
use crate::ir::item::inheritance::Extends;
use crate::ir::item::inheritance::Implements;
use crate::ir::item::member::MemberItem;
use crate::ir::item::member::constant::ClassLikeConstant;
use crate::ir::item::member::enum_case::EnumCase;
use crate::ir::item::member::hook::Hook;
use crate::ir::item::member::hook::HookBody;
use crate::ir::item::member::method::Method;
use crate::ir::item::member::property::HookedProperty;
use crate::ir::item::member::property::Property;
use crate::ir::item::member::trait_use::TraitUse;
use crate::ir::item::member::trait_use::TraitUseAdaptation;
use crate::ir::item::member::trait_use::TraitUseAliasAdaptation;
use crate::ir::item::member::trait_use::TraitUsePrecedenceAdaptation;
use crate::ir::item::modifier::Modifier;
use crate::ir::item::modifier::Visibility;
use crate::ir::item::parameter::Parameter;
use crate::ir::item::statement::ItemStatement;
use crate::ir::item::statement::class::Class;
use crate::ir::item::statement::constant::Constant;
use crate::ir::item::statement::r#enum::Enum;
use crate::ir::item::statement::r#enum::EnumBackingType;
use crate::ir::item::statement::function::Function;
use crate::ir::item::statement::interface::Interface;
use crate::ir::item::statement::r#trait::Trait;
use crate::ir::literal::Literal;
use crate::ir::literal::LiteralFloat;
use crate::ir::literal::LiteralInteger;
use crate::ir::literal::LiteralString;
use crate::ir::name::Name;
use crate::ir::statement::Block;
use crate::ir::statement::Declare;
use crate::ir::statement::DeclareItem;
use crate::ir::statement::DoWhile;
use crate::ir::statement::ElseClause;
use crate::ir::statement::For;
use crate::ir::statement::Foreach;
use crate::ir::statement::GlobalItem;
use crate::ir::statement::If;
use crate::ir::statement::Namespace;
use crate::ir::statement::Statement;
use crate::ir::statement::StaticItem;
use crate::ir::statement::Switch;
use crate::ir::statement::SwitchCase;
use crate::ir::statement::Tag;
use crate::ir::statement::Terminator;
use crate::ir::statement::Try;
use crate::ir::statement::TryCatchClause;
use crate::ir::statement::UseItem;
use crate::ir::statement::While;
use crate::ir::statement::annotation::VariableBindingAnnotation;
use crate::ir::r#type::Type;
use crate::ir::r#type::annotation::CallableTypeAnnotation;
use crate::ir::r#type::annotation::CallableTypeAnnotationParameter;
use crate::ir::r#type::annotation::ConditionalTypeAnnotation;
use crate::ir::r#type::annotation::GenericParameterTypeAnnotation;
use crate::ir::r#type::annotation::NamedTypeAnnotation;
use crate::ir::r#type::annotation::ObjectShapeTypeAnnotation;
use crate::ir::r#type::annotation::ShapeTypeAnnotation;
use crate::ir::r#type::annotation::ShapeTypeAnnotationAdditionalFields;
use crate::ir::r#type::annotation::ShapeTypeAnnotationField;
use crate::ir::r#type::annotation::StringTypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use crate::ir::variable::Variable;
use crate::ir::variable::annotation::VariableAnnotation;

mod visit;

/// The discriminant of a [`Node`]: one variant per node type, independent of
/// the IR's type parameters so it can key a rule registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeKind {
    Access,
    AnnotationError,
    AnonymousClass,
    Argument,
    ArrayElement,
    ArrayLike,
    ArrowFunction,
    AssertAnnotation,
    AssertAnnotationPattern,
    AssertAnnotationTarget,
    Assignment,
    AssignmentOperator,
    Attribute,
    Binary,
    BinaryOperator,
    Block,
    Call,
    CallableTypeAnnotation,
    CallableTypeAnnotationParameter,
    Callee,
    Class,
    ClassLikeConstant,
    Closure,
    ClosureUseClauseVariable,
    Conditional,
    ConditionalTypeAnnotation,
    Constant,
    ConstantSelector,
    Declare,
    DeclareItem,
    DirectVariable,
    DoWhile,
    Enum,
    EnumBackingType,
    EnumCase,
    Error,
    Expression,
    Extends,
    ExtendsAnnotation,
    For,
    Foreach,
    Function,
    GenericParameterTypeAnnotation,
    GlobalItem,
    Hook,
    HookBody,
    HookedProperty,
    Identifier,
    If,
    ElseClause,
    Implements,
    ImplementsAnnotation,
    ImportedTypeAliasAnnotation,
    InheritedTypeParameterAnnotation,
    Instantiation,
    Interface,
    Ir,
    ItemAnnotation,
    ItemStatement,
    CompositeString,
    CompositeStringPart,
    Literal,
    LiteralFloat,
    LiteralInteger,
    LiteralString,
    Match,
    MatchArm,
    MemberItem,
    MemberSelector,
    Method,
    MethodAnnotation,
    MixinAnnotation,
    Modifier,
    Name,
    NamedTypeAnnotation,
    Namespace,
    ObjectShapeTypeAnnotation,
    Parameter,
    ParameterAnnotation,
    ParameterOutAnnotation,
    PartialApplication,
    PartialArgument,
    Property,
    PropertyAnnotation,
    RequireExtendsAnnotation,
    RequireImplementsAnnotation,
    SealedAnnotation,
    SelfOutAnnotation,
    ShapeTypeAnnotation,
    ShapeTypeAnnotationAdditionalFields,
    ShapeTypeAnnotationField,
    Statement,
    Tag,
    Terminator,
    StaticItem,
    StringTypeAnnotation,
    Switch,
    SwitchCase,
    ThrowsAnnotation,
    Trait,
    TraitUse,
    TraitUseAdaptation,
    TraitUseAliasAdaptation,
    TraitUsePrecedenceAdaptation,
    Try,
    TryCatchClause,
    Type,
    TypeAliasAnnotation,
    TypeAnnotation,
    TypeParameterAnnotation,
    UnaryPostfix,
    UnaryPostfixOperator,
    UnaryPrefix,
    UnaryPrefixOperator,
    UseAnnotation,
    UseItem,
    Variable,
    VariableAnnotation,
    VariableBindingAnnotation,
    Visibility,
    WhereConstraintAnnotation,
    While,
    Yield,
}

/// A borrowed handle to any span-bearing node in the [`IR`] tree.
///
/// Generic over the IR's parameters (`I` symbols, `S` statement meta, `E`
/// expression meta) so every consumer can use its own instantiation; the type
/// checker walks `Node<'_, '_, SymbolId, Flow, Type>` and reads `E` off each
/// expression. `'ir` is the borrow of the tree, `'arena` where the nodes live.
#[derive(Debug, Clone, Copy)]
pub enum Node<'ir, 'arena, I, S, E> {
    Access(&'ir Access<'arena, I, S, E>),
    AnnotationError(&'ir AnnotationError),
    AnonymousClass(&'ir AnonymousClass<'arena, I, S, E>),
    Argument(&'ir Argument<'arena, I, S, E>),
    ArrayElement(&'ir ArrayElement<'arena, I, S, E>),
    ArrayLike(&'ir ArrayLike<'arena, I, S, E>),
    ArrowFunction(&'ir ArrowFunction<'arena, I, S, E>),
    AssertAnnotation(&'ir AssertAnnotation<'arena>),
    AssertAnnotationPattern(&'ir AssertAnnotationPattern<'arena>),
    AssertAnnotationTarget(&'ir AssertAnnotationTarget<'arena>),
    Assignment(&'ir Assignment<'arena, I, S, E>),
    AssignmentOperator(&'ir AssignmentOperator),
    Attribute(&'ir Attribute<'arena, I, S, E>),
    Binary(&'ir Binary<'arena, I, S, E>),
    BinaryOperator(&'ir BinaryOperator),
    Block(&'ir Block<'arena, I, S, E>),
    Call(&'ir Call<'arena, I, S, E>),
    CallableTypeAnnotation(&'ir CallableTypeAnnotation<'arena>),
    CallableTypeAnnotationParameter(&'ir CallableTypeAnnotationParameter<'arena>),
    Callee(&'ir Callee<'arena, I, S, E>),
    Class(&'ir Class<'arena, I, S, E>),
    ClassLikeConstant(&'ir ClassLikeConstant<'arena, I, S, E>),
    Closure(&'ir Closure<'arena, I, S, E>),
    ClosureUseClauseVariable(&'ir ClosureUseClauseVariable<'arena>),
    Conditional(&'ir Conditional<'arena, I, S, E>),
    ConditionalTypeAnnotation(&'ir ConditionalTypeAnnotation<'arena>),
    Constant(&'ir Constant<'arena, I, S, E>),
    ConstantSelector(&'ir ConstantSelector<'arena, I, S, E>),
    Declare(&'ir Declare<'arena, I, S, E>),
    DeclareItem(&'ir DeclareItem<'arena, I, S, E>),
    DirectVariable(&'ir DirectVariable<'arena>),
    DoWhile(&'ir DoWhile<'arena, I, S, E>),
    Enum(&'ir Enum<'arena, I, S, E>),
    EnumBackingType(&'ir EnumBackingType<'arena>),
    EnumCase(&'ir EnumCase<'arena, I, S, E>),
    Error(&'ir Error),
    Expression(&'ir Expression<'arena, I, S, E>),
    Extends(&'ir Extends<'arena>),
    ExtendsAnnotation(&'ir ExtendsAnnotation<'arena>),
    For(&'ir For<'arena, I, S, E>),
    Foreach(&'ir Foreach<'arena, I, S, E>),
    Function(&'ir Function<'arena, I, S, E>),
    GenericParameterTypeAnnotation(&'ir GenericParameterTypeAnnotation<'arena>),
    GlobalItem(&'ir GlobalItem<'arena, I, S, E>),
    Hook(&'ir Hook<'arena, I, S, E>),
    HookBody(&'ir HookBody<'arena, I, S, E>),
    HookedProperty(&'ir HookedProperty<'arena, I, S, E>),
    Identifier(&'ir Identifier<'arena>),
    If(&'ir If<'arena, I, S, E>),
    ElseClause(&'ir ElseClause<'arena, I, S, E>),
    Implements(&'ir Implements<'arena>),
    ImplementsAnnotation(&'ir ImplementsAnnotation<'arena>),
    ImportedTypeAliasAnnotation(&'ir ImportedTypeAliasAnnotation<'arena>),
    InheritedTypeParameterAnnotation(&'ir InheritedTypeParameterAnnotation<'arena>),
    Instantiation(&'ir Instantiation<'arena, I, S, E>),
    Interface(&'ir Interface<'arena, I, S, E>),
    Ir(&'ir IR<'arena, I, S, E>),
    ItemAnnotation(&'ir ItemAnnotation<'arena, I, S, E>),
    ItemStatement(&'ir ItemStatement<'arena, I, S, E>),
    Literal(&'ir Literal<'arena>),
    CompositeString(&'ir CompositeString<'arena, I, S, E>),
    CompositeStringPart(&'ir CompositeStringPart<'arena, I, S, E>),
    LiteralFloat(&'ir LiteralFloat<'arena>),
    LiteralInteger(&'ir LiteralInteger<'arena>),
    LiteralString(&'ir LiteralString<'arena>),
    Match(&'ir Match<'arena, I, S, E>),
    MatchArm(&'ir MatchArm<'arena, I, S, E>),
    MemberItem(&'ir MemberItem<'arena, I, S, E>),
    MemberSelector(&'ir MemberSelector<'arena, I, S, E>),
    Method(&'ir Method<'arena, I, S, E>),
    MethodAnnotation(&'ir MethodAnnotation<'arena, I, S, E>),
    MixinAnnotation(&'ir MixinAnnotation<'arena>),
    Modifier(&'ir Modifier),
    Name(&'ir Name<'arena>),
    NamedTypeAnnotation(&'ir NamedTypeAnnotation<'arena>),
    Namespace(&'ir Namespace<'arena, I, S, E>),
    ObjectShapeTypeAnnotation(&'ir ObjectShapeTypeAnnotation<'arena>),
    Parameter(&'ir Parameter<'arena, I, S, E>),
    ParameterAnnotation(&'ir ParameterAnnotation<'arena, I, S, E>),
    ParameterOutAnnotation(&'ir ParameterOutAnnotation<'arena>),
    PartialApplication(&'ir PartialApplication<'arena, I, S, E>),
    PartialArgument(&'ir PartialArgument<'arena, I, S, E>),
    Property(&'ir Property<'arena, I, S, E>),
    PropertyAnnotation(&'ir PropertyAnnotation<'arena>),
    RequireExtendsAnnotation(&'ir RequireExtendsAnnotation<'arena>),
    RequireImplementsAnnotation(&'ir RequireImplementsAnnotation<'arena>),
    SealedAnnotation(&'ir SealedAnnotation<'arena>),
    SelfOutAnnotation(&'ir SelfOutAnnotation<'arena>),
    ShapeTypeAnnotation(&'ir ShapeTypeAnnotation<'arena>),
    ShapeTypeAnnotationAdditionalFields(&'ir ShapeTypeAnnotationAdditionalFields<'arena>),
    ShapeTypeAnnotationField(&'ir ShapeTypeAnnotationField<'arena>),
    Statement(&'ir Statement<'arena, I, S, E>),
    Tag(&'ir Tag),
    Terminator(&'ir Terminator),
    StaticItem(&'ir StaticItem<'arena, I, S, E>),
    StringTypeAnnotation(&'ir StringTypeAnnotation<'arena>),
    Switch(&'ir Switch<'arena, I, S, E>),
    SwitchCase(&'ir SwitchCase<'arena, I, S, E>),
    ThrowsAnnotation(&'ir ThrowsAnnotation<'arena>),
    Trait(&'ir Trait<'arena, I, S, E>),
    TraitUse(&'ir TraitUse<'arena, I, S, E>),
    TraitUseAdaptation(&'ir TraitUseAdaptation<'arena>),
    TraitUseAliasAdaptation(&'ir TraitUseAliasAdaptation<'arena>),
    TraitUsePrecedenceAdaptation(&'ir TraitUsePrecedenceAdaptation<'arena>),
    Try(&'ir Try<'arena, I, S, E>),
    TryCatchClause(&'ir TryCatchClause<'arena, I, S, E>),
    Type(&'ir Type<'arena>),
    TypeAliasAnnotation(&'ir TypeAliasAnnotation<'arena>),
    TypeAnnotation(&'ir TypeAnnotation<'arena>),
    TypeParameterAnnotation(&'ir TypeParameterAnnotation<'arena>),
    UnaryPostfix(&'ir UnaryPostfix<'arena, I, S, E>),
    UnaryPostfixOperator(&'ir UnaryPostfixOperator),
    UnaryPrefix(&'ir UnaryPrefix<'arena, I, S, E>),
    UnaryPrefixOperator(&'ir UnaryPrefixOperator),
    UseAnnotation(&'ir UseAnnotation<'arena>),
    UseItem(&'ir UseItem<'arena>),
    Variable(&'ir Variable<'arena, I, S, E>),
    VariableAnnotation(&'ir VariableAnnotation<'arena>),
    VariableBindingAnnotation(&'ir VariableBindingAnnotation<'arena>),
    Visibility(&'ir Visibility),
    WhereConstraintAnnotation(&'ir WhereConstraintAnnotation<'arena>),
    While(&'ir While<'arena, I, S, E>),
    Yield(&'ir Yield<'arena, I, S, E>),
}

impl<'ir, 'arena, I, S, E> Node<'ir, 'arena, I, S, E> {
    /// The discriminant of this node.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        match self {
            Self::Access(_) => NodeKind::Access,
            Self::AnnotationError(_) => NodeKind::AnnotationError,
            Self::AnonymousClass(_) => NodeKind::AnonymousClass,
            Self::Argument(_) => NodeKind::Argument,
            Self::ArrayElement(_) => NodeKind::ArrayElement,
            Self::ArrayLike(_) => NodeKind::ArrayLike,
            Self::ArrowFunction(_) => NodeKind::ArrowFunction,
            Self::AssertAnnotation(_) => NodeKind::AssertAnnotation,
            Self::AssertAnnotationPattern(_) => NodeKind::AssertAnnotationPattern,
            Self::AssertAnnotationTarget(_) => NodeKind::AssertAnnotationTarget,
            Self::Assignment(_) => NodeKind::Assignment,
            Self::AssignmentOperator(_) => NodeKind::AssignmentOperator,
            Self::Attribute(_) => NodeKind::Attribute,
            Self::Binary(_) => NodeKind::Binary,
            Self::BinaryOperator(_) => NodeKind::BinaryOperator,
            Self::Block(_) => NodeKind::Block,
            Self::Call(_) => NodeKind::Call,
            Self::CallableTypeAnnotation(_) => NodeKind::CallableTypeAnnotation,
            Self::CallableTypeAnnotationParameter(_) => NodeKind::CallableTypeAnnotationParameter,
            Self::Callee(_) => NodeKind::Callee,
            Self::Class(_) => NodeKind::Class,
            Self::ClassLikeConstant(_) => NodeKind::ClassLikeConstant,
            Self::Closure(_) => NodeKind::Closure,
            Self::ClosureUseClauseVariable(_) => NodeKind::ClosureUseClauseVariable,
            Self::Conditional(_) => NodeKind::Conditional,
            Self::ConditionalTypeAnnotation(_) => NodeKind::ConditionalTypeAnnotation,
            Self::Constant(_) => NodeKind::Constant,
            Self::ConstantSelector(_) => NodeKind::ConstantSelector,
            Self::Declare(_) => NodeKind::Declare,
            Self::DeclareItem(_) => NodeKind::DeclareItem,
            Self::DirectVariable(_) => NodeKind::DirectVariable,
            Self::DoWhile(_) => NodeKind::DoWhile,
            Self::Enum(_) => NodeKind::Enum,
            Self::EnumBackingType(_) => NodeKind::EnumBackingType,
            Self::EnumCase(_) => NodeKind::EnumCase,
            Self::Error(_) => NodeKind::Error,
            Self::Expression(_) => NodeKind::Expression,
            Self::Extends(_) => NodeKind::Extends,
            Self::ExtendsAnnotation(_) => NodeKind::ExtendsAnnotation,
            Self::For(_) => NodeKind::For,
            Self::Foreach(_) => NodeKind::Foreach,
            Self::Function(_) => NodeKind::Function,
            Self::GenericParameterTypeAnnotation(_) => NodeKind::GenericParameterTypeAnnotation,
            Self::GlobalItem(_) => NodeKind::GlobalItem,
            Self::Hook(_) => NodeKind::Hook,
            Self::HookBody(_) => NodeKind::HookBody,
            Self::HookedProperty(_) => NodeKind::HookedProperty,
            Self::Identifier(_) => NodeKind::Identifier,
            Self::If(_) => NodeKind::If,
            Self::ElseClause(_) => NodeKind::ElseClause,
            Self::Implements(_) => NodeKind::Implements,
            Self::ImplementsAnnotation(_) => NodeKind::ImplementsAnnotation,
            Self::ImportedTypeAliasAnnotation(_) => NodeKind::ImportedTypeAliasAnnotation,
            Self::InheritedTypeParameterAnnotation(_) => NodeKind::InheritedTypeParameterAnnotation,
            Self::Instantiation(_) => NodeKind::Instantiation,
            Self::Interface(_) => NodeKind::Interface,
            Self::Ir(_) => NodeKind::Ir,
            Self::ItemAnnotation(_) => NodeKind::ItemAnnotation,
            Self::ItemStatement(_) => NodeKind::ItemStatement,
            Self::CompositeString(_) => NodeKind::CompositeString,
            Self::CompositeStringPart(_) => NodeKind::CompositeStringPart,
            Self::Literal(_) => NodeKind::Literal,
            Self::LiteralFloat(_) => NodeKind::LiteralFloat,
            Self::LiteralInteger(_) => NodeKind::LiteralInteger,
            Self::LiteralString(_) => NodeKind::LiteralString,
            Self::Match(_) => NodeKind::Match,
            Self::MatchArm(_) => NodeKind::MatchArm,
            Self::MemberItem(_) => NodeKind::MemberItem,
            Self::MemberSelector(_) => NodeKind::MemberSelector,
            Self::Method(_) => NodeKind::Method,
            Self::MethodAnnotation(_) => NodeKind::MethodAnnotation,
            Self::MixinAnnotation(_) => NodeKind::MixinAnnotation,
            Self::Modifier(_) => NodeKind::Modifier,
            Self::Name(_) => NodeKind::Name,
            Self::NamedTypeAnnotation(_) => NodeKind::NamedTypeAnnotation,
            Self::Namespace(_) => NodeKind::Namespace,
            Self::ObjectShapeTypeAnnotation(_) => NodeKind::ObjectShapeTypeAnnotation,
            Self::Parameter(_) => NodeKind::Parameter,
            Self::ParameterAnnotation(_) => NodeKind::ParameterAnnotation,
            Self::ParameterOutAnnotation(_) => NodeKind::ParameterOutAnnotation,
            Self::PartialApplication(_) => NodeKind::PartialApplication,
            Self::PartialArgument(_) => NodeKind::PartialArgument,
            Self::Property(_) => NodeKind::Property,
            Self::PropertyAnnotation(_) => NodeKind::PropertyAnnotation,
            Self::RequireExtendsAnnotation(_) => NodeKind::RequireExtendsAnnotation,
            Self::RequireImplementsAnnotation(_) => NodeKind::RequireImplementsAnnotation,
            Self::SealedAnnotation(_) => NodeKind::SealedAnnotation,
            Self::SelfOutAnnotation(_) => NodeKind::SelfOutAnnotation,
            Self::ShapeTypeAnnotation(_) => NodeKind::ShapeTypeAnnotation,
            Self::ShapeTypeAnnotationAdditionalFields(_) => NodeKind::ShapeTypeAnnotationAdditionalFields,
            Self::ShapeTypeAnnotationField(_) => NodeKind::ShapeTypeAnnotationField,
            Self::Statement(_) => NodeKind::Statement,
            Self::Tag(_) => NodeKind::Tag,
            Self::Terminator(_) => NodeKind::Terminator,
            Self::StaticItem(_) => NodeKind::StaticItem,
            Self::StringTypeAnnotation(_) => NodeKind::StringTypeAnnotation,
            Self::Switch(_) => NodeKind::Switch,
            Self::SwitchCase(_) => NodeKind::SwitchCase,
            Self::ThrowsAnnotation(_) => NodeKind::ThrowsAnnotation,
            Self::Trait(_) => NodeKind::Trait,
            Self::TraitUse(_) => NodeKind::TraitUse,
            Self::TraitUseAdaptation(_) => NodeKind::TraitUseAdaptation,
            Self::TraitUseAliasAdaptation(_) => NodeKind::TraitUseAliasAdaptation,
            Self::TraitUsePrecedenceAdaptation(_) => NodeKind::TraitUsePrecedenceAdaptation,
            Self::Try(_) => NodeKind::Try,
            Self::TryCatchClause(_) => NodeKind::TryCatchClause,
            Self::Type(_) => NodeKind::Type,
            Self::TypeAliasAnnotation(_) => NodeKind::TypeAliasAnnotation,
            Self::TypeAnnotation(_) => NodeKind::TypeAnnotation,
            Self::TypeParameterAnnotation(_) => NodeKind::TypeParameterAnnotation,
            Self::UnaryPostfix(_) => NodeKind::UnaryPostfix,
            Self::UnaryPostfixOperator(_) => NodeKind::UnaryPostfixOperator,
            Self::UnaryPrefix(_) => NodeKind::UnaryPrefix,
            Self::UnaryPrefixOperator(_) => NodeKind::UnaryPrefixOperator,
            Self::UseAnnotation(_) => NodeKind::UseAnnotation,
            Self::UseItem(_) => NodeKind::UseItem,
            Self::Variable(_) => NodeKind::Variable,
            Self::VariableAnnotation(_) => NodeKind::VariableAnnotation,
            Self::VariableBindingAnnotation(_) => NodeKind::VariableBindingAnnotation,
            Self::Visibility(_) => NodeKind::Visibility,
            Self::WhereConstraintAnnotation(_) => NodeKind::WhereConstraintAnnotation,
            Self::While(_) => NodeKind::While,
            Self::Yield(_) => NodeKind::Yield,
        }
    }

    /// This node's immediate sub-nodes, in source order.
    #[must_use]
    pub fn children(&self) -> std::vec::Vec<Node<'ir, 'arena, I, S, E>> {
        let mut children = std::vec::Vec::new();
        self.visit_children(|child| children.push(child));
        children
    }
}

impl<I, S, E> HasSpan for Node<'_, '_, I, S, E> {
    fn span(&self) -> Span {
        match self {
            Self::Access(node) => node.span(),
            Self::AnnotationError(node) => node.span(),
            Self::AnonymousClass(node) => node.span(),
            Self::Argument(node) => node.span(),
            Self::ArrayElement(node) => node.span(),
            Self::ArrayLike(node) => node.span(),
            Self::ArrowFunction(node) => node.span(),
            Self::AssertAnnotation(node) => node.span(),
            Self::AssertAnnotationPattern(node) => node.span(),
            Self::AssertAnnotationTarget(node) => node.span(),
            Self::Assignment(node) => node.span(),
            Self::AssignmentOperator(node) => node.span(),
            Self::Attribute(node) => node.span(),
            Self::Binary(node) => node.span(),
            Self::BinaryOperator(node) => node.span(),
            Self::Block(node) => node.span(),
            Self::Call(node) => node.span(),
            Self::CallableTypeAnnotation(node) => node.span(),
            Self::CallableTypeAnnotationParameter(node) => node.span(),
            Self::Callee(node) => node.span(),
            Self::Class(node) => node.span(),
            Self::ClassLikeConstant(node) => node.span(),
            Self::Closure(node) => node.span(),
            Self::ClosureUseClauseVariable(node) => node.span(),
            Self::Conditional(node) => node.span(),
            Self::ConditionalTypeAnnotation(node) => node.span(),
            Self::Constant(node) => node.span(),
            Self::ConstantSelector(node) => node.span(),
            Self::Declare(node) => node.span(),
            Self::DeclareItem(node) => node.span(),
            Self::DirectVariable(node) => node.span(),
            Self::DoWhile(node) => node.span(),
            Self::Enum(node) => node.span(),
            Self::EnumBackingType(node) => node.span(),
            Self::EnumCase(node) => node.span(),
            Self::Error(node) => node.span(),
            Self::Expression(node) => node.span(),
            Self::Extends(node) => node.span(),
            Self::ExtendsAnnotation(node) => node.span(),
            Self::For(node) => node.span(),
            Self::Foreach(node) => node.span(),
            Self::Function(node) => node.span(),
            Self::GenericParameterTypeAnnotation(node) => node.span(),
            Self::GlobalItem(node) => node.span(),
            Self::Hook(node) => node.span(),
            Self::HookBody(node) => node.span(),
            Self::HookedProperty(node) => node.span(),
            Self::Identifier(node) => node.span(),
            Self::If(node) => node.span(),
            Self::ElseClause(node) => node.span(),
            Self::Implements(node) => node.span(),
            Self::ImplementsAnnotation(node) => node.span(),
            Self::ImportedTypeAliasAnnotation(node) => node.span(),
            Self::InheritedTypeParameterAnnotation(node) => node.span(),
            Self::Instantiation(node) => node.span(),
            Self::Interface(node) => node.span(),
            Self::Ir(node) => node.span(),
            Self::ItemAnnotation(node) => node.span(),
            Self::ItemStatement(node) => node.span(),
            Self::CompositeString(node) => node.span(),
            Self::CompositeStringPart(node) => node.span(),
            Self::Literal(node) => node.span(),
            Self::LiteralFloat(node) => node.span(),
            Self::LiteralInteger(node) => node.span(),
            Self::LiteralString(node) => node.span(),
            Self::Match(node) => node.span(),
            Self::MatchArm(node) => node.span(),
            Self::MemberItem(node) => node.span(),
            Self::MemberSelector(node) => node.span(),
            Self::Method(node) => node.span(),
            Self::MethodAnnotation(node) => node.span(),
            Self::MixinAnnotation(node) => node.span(),
            Self::Modifier(node) => node.span(),
            Self::Name(node) => node.span(),
            Self::NamedTypeAnnotation(node) => node.span(),
            Self::Namespace(node) => node.span(),
            Self::ObjectShapeTypeAnnotation(node) => node.span(),
            Self::Parameter(node) => node.span(),
            Self::ParameterAnnotation(node) => node.span(),
            Self::ParameterOutAnnotation(node) => node.span(),
            Self::PartialApplication(node) => node.span(),
            Self::PartialArgument(node) => node.span(),
            Self::Property(node) => node.span(),
            Self::PropertyAnnotation(node) => node.span(),
            Self::RequireExtendsAnnotation(node) => node.span(),
            Self::RequireImplementsAnnotation(node) => node.span(),
            Self::SealedAnnotation(node) => node.span(),
            Self::SelfOutAnnotation(node) => node.span(),
            Self::ShapeTypeAnnotation(node) => node.span(),
            Self::ShapeTypeAnnotationAdditionalFields(node) => node.span(),
            Self::ShapeTypeAnnotationField(node) => node.span(),
            Self::Statement(node) => node.span(),
            Self::Tag(node) => node.span(),
            Self::Terminator(node) => node.span(),
            Self::StaticItem(node) => node.span(),
            Self::StringTypeAnnotation(node) => node.span(),
            Self::Switch(node) => node.span(),
            Self::SwitchCase(node) => node.span(),
            Self::ThrowsAnnotation(node) => node.span(),
            Self::Trait(node) => node.span(),
            Self::TraitUse(node) => node.span(),
            Self::TraitUseAdaptation(node) => node.span(),
            Self::TraitUseAliasAdaptation(node) => node.span(),
            Self::TraitUsePrecedenceAdaptation(node) => node.span(),
            Self::Try(node) => node.span(),
            Self::TryCatchClause(node) => node.span(),
            Self::Type(node) => node.span(),
            Self::TypeAliasAnnotation(node) => node.span(),
            Self::TypeAnnotation(node) => node.span(),
            Self::TypeParameterAnnotation(node) => node.span(),
            Self::UnaryPostfix(node) => node.span(),
            Self::UnaryPostfixOperator(node) => node.span(),
            Self::UnaryPrefix(node) => node.span(),
            Self::UnaryPrefixOperator(node) => node.span(),
            Self::UseAnnotation(node) => node.span(),
            Self::UseItem(node) => node.span(),
            Self::Variable(node) => node.span(),
            Self::VariableAnnotation(node) => node.span(),
            Self::VariableBindingAnnotation(node) => node.span(),
            Self::Visibility(node) => node.span(),
            Self::WhereConstraintAnnotation(node) => node.span(),
            Self::While(node) => node.span(),
            Self::Yield(node) => node.span(),
        }
    }
}
