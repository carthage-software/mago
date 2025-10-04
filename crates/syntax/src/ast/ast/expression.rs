use serde::Serialize;
use strum::Display;

use mago_php_version::PHPVersion;
use mago_php_version::feature::Feature;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::UnaryPrefixOperator;
use crate::ast::ast::access::Access;
use crate::ast::ast::access::ClassConstantAccess;
use crate::ast::ast::access::ConstantAccess;
use crate::ast::ast::access::NullSafePropertyAccess;
use crate::ast::ast::access::PropertyAccess;
use crate::ast::ast::argument::Argument;
use crate::ast::ast::array::Array;
use crate::ast::ast::array::ArrayAccess;
use crate::ast::ast::array::ArrayAppend;
use crate::ast::ast::array::ArrayElement;
use crate::ast::ast::array::LegacyArray;
use crate::ast::ast::array::List;
use crate::ast::ast::assignment::Assignment;
use crate::ast::ast::binary::Binary;
use crate::ast::ast::call::Call;
use crate::ast::ast::class_like::AnonymousClass;
use crate::ast::ast::class_like::member::ClassLikeConstantSelector;
use crate::ast::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::ast::clone::Clone;
use crate::ast::ast::closure_creation::ClosureCreation;
use crate::ast::ast::conditional::Conditional;
use crate::ast::ast::construct::Construct;
use crate::ast::ast::control_flow::r#match::Match;
use crate::ast::ast::function_like::arrow_function::ArrowFunction;
use crate::ast::ast::function_like::closure::Closure;
use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::instantiation::Instantiation;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::literal::Literal;
use crate::ast::ast::magic_constant::MagicConstant;
use crate::ast::ast::pipe::Pipe;
use crate::ast::ast::string::CompositeString;
use crate::ast::ast::string::StringPart;
use crate::ast::ast::throw::Throw;
use crate::ast::ast::unary::UnaryPostfix;
use crate::ast::ast::unary::UnaryPrefix;
use crate::ast::ast::variable::Variable;
use crate::ast::ast::r#yield::Yield;
use crate::ast::node::NodeKind;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Parenthesized<'arena> {
    pub left_parenthesis: Span,
    pub expression: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Expression<'arena> {
    Binary(Binary<'arena>),
    UnaryPrefix(UnaryPrefix<'arena>),
    UnaryPostfix(UnaryPostfix<'arena>),
    Parenthesized(Parenthesized<'arena>),
    Literal(Literal<'arena>),
    CompositeString(CompositeString<'arena>),
    Assignment(Assignment<'arena>),
    Conditional(Conditional<'arena>),
    Array(Array<'arena>),
    LegacyArray(LegacyArray<'arena>),
    List(List<'arena>),
    ArrayAccess(ArrayAccess<'arena>),
    ArrayAppend(ArrayAppend<'arena>),
    AnonymousClass(AnonymousClass<'arena>),
    Closure(Closure<'arena>),
    ArrowFunction(ArrowFunction<'arena>),
    Variable(Variable<'arena>),
    ConstantAccess(ConstantAccess<'arena>),
    Identifier(Identifier<'arena>),
    Match(Match<'arena>),
    Yield(Yield<'arena>),
    Construct(Construct<'arena>),
    Throw(Throw<'arena>),
    Clone(Clone<'arena>),
    Call(Call<'arena>),
    Access(Access<'arena>),
    ClosureCreation(ClosureCreation<'arena>),
    Parent(Keyword<'arena>),
    Static(Keyword<'arena>),
    Self_(Keyword<'arena>),
    Instantiation(Instantiation<'arena>),
    MagicConstant(MagicConstant<'arena>),
    Pipe(Pipe<'arena>),
}

impl<'arena> Expression<'arena> {
    pub fn is_constant(&self, version: &PHPVersion, initialization: bool) -> bool {
        match &self {
            Self::Binary(operation) => {
                operation.operator.is_constant()
                    && operation.lhs.is_constant(version, initialization)
                    && operation.rhs.is_constant(version, initialization)
            }
            Self::UnaryPrefix(operation) => {
                operation.operator.is_constant() && operation.operand.is_constant(version, initialization)
            }
            Self::UnaryPostfix(operation) => {
                operation.operator.is_constant() && operation.operand.is_constant(version, initialization)
            }
            Self::Literal(_) => true,
            Self::Identifier(_) => true,
            Self::MagicConstant(_) => true,
            Self::ConstantAccess(_) => true,
            Self::Self_(_) => true,
            Self::Parent(_) => true,
            Self::Static(_) => true,
            Self::Parenthesized(expression) => expression.expression.is_constant(version, initialization),
            Self::Access(access) => match access {
                Access::ClassConstant(ClassConstantAccess { class, constant, .. }) => {
                    matches!(constant, ClassLikeConstantSelector::Identifier(_))
                        && class.is_constant(version, initialization)
                }
                Access::Property(PropertyAccess { object, property, .. }) => {
                    matches!(property, ClassLikeMemberSelector::Identifier(_))
                        && object.is_constant(version, initialization)
                }
                Access::NullSafeProperty(NullSafePropertyAccess { object, property, .. }) => {
                    matches!(property, ClassLikeMemberSelector::Identifier(_))
                        && object.is_constant(version, initialization)
                }
                _ => false,
            },
            Self::ArrayAccess(access) => {
                access.array.is_constant(version, initialization) && access.index.is_constant(version, initialization)
            }
            Self::Instantiation(instantiation)
                if initialization && version.is_supported(Feature::NewInInitializers) =>
            {
                instantiation.class.is_constant(version, initialization)
                    && instantiation
                        .argument_list
                        .as_ref()
                        .map(|arguments| {
                            arguments.arguments.iter().all(|argument| match &argument {
                                Argument::Positional(positional_argument) => {
                                    positional_argument.ellipsis.is_none()
                                        && positional_argument.value.is_constant(version, initialization)
                                }
                                Argument::Named(named_argument) => {
                                    named_argument.value.is_constant(version, initialization)
                                }
                            })
                        })
                        .unwrap_or(true)
            }
            Self::Conditional(conditional) => {
                conditional.condition.is_constant(version, initialization)
                    && conditional.then.as_ref().map(|e| e.is_constant(version, initialization)).unwrap_or(true)
                    && conditional.r#else.is_constant(version, initialization)
            }
            Self::Array(array) => array.elements.nodes.iter().all(|element| match &element {
                ArrayElement::KeyValue(key_value_array_element) => {
                    key_value_array_element.key.is_constant(version, initialization)
                        && key_value_array_element.value.is_constant(version, initialization)
                }
                ArrayElement::Value(value_array_element) => {
                    value_array_element.value.is_constant(version, initialization)
                }
                ArrayElement::Variadic(variadic_array_element) => {
                    variadic_array_element.value.is_constant(version, initialization)
                }
                ArrayElement::Missing(_) => false,
            }),
            Self::LegacyArray(array) => array.elements.nodes.iter().all(|element| match &element {
                ArrayElement::KeyValue(key_value_array_element) => {
                    key_value_array_element.key.is_constant(version, initialization)
                        && key_value_array_element.value.is_constant(version, initialization)
                }
                ArrayElement::Value(value_array_element) => {
                    value_array_element.value.is_constant(version, initialization)
                }
                ArrayElement::Variadic(variadic_array_element) => {
                    variadic_array_element.value.is_constant(version, initialization)
                }
                ArrayElement::Missing(_) => false,
            }),
            Self::CompositeString(string) => match string {
                CompositeString::Interpolated(interpolated_string) => {
                    interpolated_string.parts.iter().all(|part| match part {
                        StringPart::Literal(_) => true,
                        StringPart::Expression(_) => false,
                        StringPart::BracedExpression(_) => false,
                    })
                }
                CompositeString::Document(document_string) => document_string.parts.iter().all(|part| match part {
                    StringPart::Literal(_) => true,
                    StringPart::Expression(_) => false,
                    StringPart::BracedExpression(_) => false,
                }),
                CompositeString::ShellExecute(_) => false,
            },
            Self::Closure(closure) => {
                closure.r#static.is_some() && version.is_supported(Feature::ClosureInConstantExpressions)
            }
            Self::ClosureCreation(closure_creation) => {
                if !version.is_supported(Feature::ClosureCreationInConstantExpressions) {
                    return false;
                }

                match closure_creation {
                    ClosureCreation::Function(function_closure_creation) => {
                        function_closure_creation.function.is_constant(version, initialization)
                    }
                    ClosureCreation::Method(method_closure_creation) => {
                        method_closure_creation.object.is_constant(version, initialization)
                            && matches!(method_closure_creation.method, ClassLikeMemberSelector::Identifier(_))
                    }
                    ClosureCreation::StaticMethod(static_method_closure_creation) => {
                        static_method_closure_creation.class.is_constant(version, initialization)
                            && matches!(static_method_closure_creation.method, ClassLikeMemberSelector::Identifier(_))
                    }
                }
            }
            _ => false,
        }
    }

    #[inline]
    pub const fn unparenthesized(&self) -> &Expression<'arena> {
        if let Expression::Parenthesized(expression) = self { expression.expression } else { self }
    }

    #[inline]
    pub const fn is_assignment(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_assignment()
        } else {
            matches!(&self, Expression::Assignment(_))
        }
    }

    #[inline]
    pub const fn is_call(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_call()
        } else {
            matches!(&self, Expression::Call(_))
        }
    }

    #[inline]
    pub const fn is_variable(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_variable()
        } else {
            matches!(&self, Expression::Variable(_))
        }
    }

    #[inline]
    pub const fn is_binary(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_binary()
        } else {
            matches!(&self, Expression::Binary(_))
        }
    }

    #[inline]
    pub const fn is_unary(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_unary()
        } else {
            matches!(&self, Expression::UnaryPrefix(_) | Expression::UnaryPostfix(_))
        }
    }

    #[inline]
    pub const fn is_conditional(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_conditional()
        } else {
            matches!(&self, Expression::Conditional(_))
        }
    }

    #[inline]
    pub const fn is_unary_or_binary_or_conditional(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_unary_or_binary_or_conditional()
        } else {
            matches!(
                &self,
                Expression::UnaryPrefix(_)
                    | Expression::UnaryPostfix(_)
                    | Expression::Binary(_)
                    | Expression::Conditional(_)
            )
        }
    }

    #[inline]
    pub const fn is_reference(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_reference()
        } else {
            matches!(&self, Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), .. }))
        }
    }

    #[inline]
    pub const fn is_true(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_true()
        } else {
            matches!(&self, Expression::Literal(Literal::True(_)))
        }
    }

    #[inline]
    pub const fn is_false(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_false()
        } else {
            matches!(&self, Expression::Literal(Literal::False(_)))
        }
    }

    #[inline]
    pub const fn evaluates_to_boolean(&self) -> bool {
        match self {
            Expression::Parenthesized(expression) => expression.expression.evaluates_to_boolean(),
            Expression::Literal(Literal::True(_)) | Expression::Literal(Literal::False(_)) => true,
            Expression::Binary(Binary { operator, .. })
                if operator.is_comparison() || operator.is_logical() || operator.is_instanceof() =>
            {
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn is_literal(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_literal()
        } else {
            matches!(&self, Expression::Literal(_))
        }
    }

    #[inline]
    pub fn is_string_literal(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_string_literal()
        } else {
            matches!(&self, Expression::Literal(Literal::String(_)))
        }
    }

    #[inline]
    pub fn is_referenceable(&self, include_calls: bool) -> bool {
        match self {
            Expression::Variable(_) => true,
            Expression::ArrayAccess(array_access) => array_access.array.is_referenceable(include_calls),
            Expression::Access(Access::Property(_) | Access::StaticProperty(_)) => true,
            Expression::Pipe(_) if include_calls => true,
            Expression::Call(call) if include_calls && !call.is_null_safe() => true,
            _ => false,
        }
    }

    #[inline]
    pub fn get_array_like_elements(&self) -> Option<&[ArrayElement<'arena>]> {
        match self {
            Expression::Parenthesized(expression) => expression.expression.get_array_like_elements(),
            Expression::Array(array) => Some(array.elements.as_slice()),
            Expression::LegacyArray(array) => Some(array.elements.as_slice()),
            Expression::List(list) => Some(list.elements.as_slice()),
            _ => None,
        }
    }

    #[inline]
    pub const fn node_kind(&self) -> NodeKind {
        match &self {
            Expression::Binary(_) => NodeKind::Binary,
            Expression::ConstantAccess(_) => NodeKind::ConstantAccess,
            Expression::UnaryPrefix(_) => NodeKind::UnaryPrefix,
            Expression::UnaryPostfix(_) => NodeKind::UnaryPostfix,
            Expression::Parenthesized(_) => NodeKind::Parenthesized,
            Expression::Literal(_) => NodeKind::Literal,
            Expression::CompositeString(_) => NodeKind::CompositeString,
            Expression::Assignment(_) => NodeKind::Assignment,
            Expression::Conditional(_) => NodeKind::Conditional,
            Expression::Array(_) => NodeKind::Array,
            Expression::LegacyArray(_) => NodeKind::LegacyArray,
            Expression::List(_) => NodeKind::List,
            Expression::ArrayAccess(_) => NodeKind::ArrayAccess,
            Expression::ArrayAppend(_) => NodeKind::ArrayAppend,
            Expression::AnonymousClass(_) => NodeKind::AnonymousClass,
            Expression::Closure(_) => NodeKind::Closure,
            Expression::ArrowFunction(_) => NodeKind::ArrowFunction,
            Expression::Variable(_) => NodeKind::Variable,
            Expression::Identifier(_) => NodeKind::Identifier,
            Expression::Match(_) => NodeKind::Match,
            Expression::Yield(_) => NodeKind::Yield,
            Expression::Construct(_) => NodeKind::Construct,
            Expression::Throw(_) => NodeKind::Throw,
            Expression::Clone(_) => NodeKind::Clone,
            Expression::Call(_) => NodeKind::Call,
            Expression::Access(_) => NodeKind::Access,
            Expression::ClosureCreation(_) => NodeKind::ClosureCreation,
            Expression::Instantiation(_) => NodeKind::Instantiation,
            Expression::MagicConstant(_) => NodeKind::MagicConstant,
            Expression::Parent(_) => NodeKind::Keyword,
            Expression::Static(_) => NodeKind::Keyword,
            Expression::Self_(_) => NodeKind::Keyword,
            Expression::Pipe(_) => NodeKind::Pipe,
        }
    }
}

impl HasSpan for Parenthesized<'_> {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}

impl HasSpan for Expression<'_> {
    fn span(&self) -> Span {
        match &self {
            Expression::Binary(expression) => expression.span(),
            Expression::ConstantAccess(expression) => expression.span(),
            Expression::UnaryPrefix(expression) => expression.span(),
            Expression::UnaryPostfix(expression) => expression.span(),
            Expression::Parenthesized(expression) => expression.span(),
            Expression::Literal(expression) => expression.span(),
            Expression::CompositeString(expression) => expression.span(),
            Expression::Assignment(expression) => expression.span(),
            Expression::Conditional(expression) => expression.span(),
            Expression::Array(expression) => expression.span(),
            Expression::LegacyArray(expression) => expression.span(),
            Expression::List(expression) => expression.span(),
            Expression::ArrayAccess(expression) => expression.span(),
            Expression::ArrayAppend(expression) => expression.span(),
            Expression::AnonymousClass(expression) => expression.span(),
            Expression::Closure(expression) => expression.span(),
            Expression::ArrowFunction(expression) => expression.span(),
            Expression::Variable(expression) => expression.span(),
            Expression::Identifier(expression) => expression.span(),
            Expression::Match(expression) => expression.span(),
            Expression::Yield(expression) => expression.span(),
            Expression::Construct(expression) => expression.span(),
            Expression::Throw(expression) => expression.span(),
            Expression::Clone(expression) => expression.span(),
            Expression::Call(expression) => expression.span(),
            Expression::Access(expression) => expression.span(),
            Expression::ClosureCreation(expression) => expression.span(),
            Expression::Parent(expression) => expression.span(),
            Expression::Static(expression) => expression.span(),
            Expression::Self_(expression) => expression.span(),
            Expression::Instantiation(expression) => expression.span(),
            Expression::MagicConstant(expression) => expression.span(),
            Expression::Pipe(expression) => expression.span(),
        }
    }
}
