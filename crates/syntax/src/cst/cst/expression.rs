use serde::Serialize;
use strum::Display;

use mago_php_version::PHPVersion;
use mago_php_version::feature::Feature;
use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::UnaryPrefixOperator;
use crate::cst::cst::access::Access;
use crate::cst::cst::access::ClassConstantAccess;
use crate::cst::cst::access::ConstantAccess;
use crate::cst::cst::access::NullSafePropertyAccess;
use crate::cst::cst::access::PropertyAccess;
use crate::cst::cst::argument::Argument;
use crate::cst::cst::array::Array;
use crate::cst::cst::array::ArrayAccess;
use crate::cst::cst::array::ArrayAppend;
use crate::cst::cst::array::ArrayElement;
use crate::cst::cst::array::LegacyArray;
use crate::cst::cst::array::List;
use crate::cst::cst::assignment::Assignment;
use crate::cst::cst::binary::Binary;
use crate::cst::cst::call::Call;
use crate::cst::cst::class_like::AnonymousClass;
use crate::cst::cst::class_like::member::ClassLikeConstantSelector;
use crate::cst::cst::class_like::member::ClassLikeMemberSelector;
use crate::cst::cst::clone::Clone;
use crate::cst::cst::conditional::Conditional;
use crate::cst::cst::construct::Construct;
use crate::cst::cst::control_flow::r#match::Match;
use crate::cst::cst::function_like::arrow_function::ArrowFunction;
use crate::cst::cst::function_like::closure::Closure;
use crate::cst::cst::identifier::Identifier;
use crate::cst::cst::instantiation::Instantiation;
use crate::cst::cst::keyword::Keyword;
use crate::cst::cst::literal::Literal;
use crate::cst::cst::magic_constant::MagicConstant;
use crate::cst::cst::partial_application::PartialApplication;
use crate::cst::cst::pipe::Pipe;
use crate::cst::cst::string::CompositeString;
use crate::cst::cst::string::StringPart;
use crate::cst::cst::throw::Throw;
use crate::cst::cst::unary::UnaryPostfix;
use crate::cst::cst::unary::UnaryPrefix;
use crate::cst::cst::variable::Variable;
use crate::cst::cst::r#yield::Yield;
use crate::cst::node::NodeKind;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Parenthesized<'arena> {
    pub left_parenthesis: Span,
    pub expression: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[non_exhaustive]
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
    PartialApplication(PartialApplication<'arena>),
    Access(Access<'arena>),
    Parent(Keyword<'arena>),
    Static(Keyword<'arena>),
    Self_(Keyword<'arena>),
    Instantiation(Instantiation<'arena>),
    MagicConstant(MagicConstant<'arena>),
    Pipe(Pipe<'arena>),
    Error(Span),
}

impl<'arena> Expression<'arena> {
    #[must_use]
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
            Self::Static(_) => false,
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
                Access::StaticProperty(_) => false,
            },
            Self::ArrayAccess(access) => {
                access.array.is_constant(version, initialization) && access.index.is_constant(version, initialization)
            }
            Self::Instantiation(instantiation)
                if initialization && version.is_supported(Feature::NewInInitializers) =>
            {
                instantiation.class.is_constant(version, initialization)
                    && instantiation.argument_list.as_ref().is_none_or(|arguments| {
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
            }
            Self::Conditional(conditional) => {
                conditional.condition.is_constant(version, initialization)
                    && conditional.then.as_ref().is_none_or(|e| e.is_constant(version, initialization))
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
            Self::PartialApplication(partial_application) => {
                // Only FCC (First-Class Callables) can be constant expressions
                // PFA with placeholders is not a constant expression
                if !partial_application.is_first_class_callable() {
                    return false;
                }

                if !version.is_supported(Feature::ClosureCreationInConstantExpressions) {
                    return false;
                }

                match partial_application {
                    PartialApplication::Function(function_pa) => {
                        function_pa.function.is_constant(version, initialization)
                    }
                    PartialApplication::Method(method_pa) => {
                        method_pa.object.is_constant(version, initialization)
                            && matches!(method_pa.method, ClassLikeMemberSelector::Identifier(_))
                    }
                    PartialApplication::StaticMethod(static_method_pa) => {
                        static_method_pa.class.is_constant(version, initialization)
                            && matches!(static_method_pa.method, ClassLikeMemberSelector::Identifier(_))
                    }
                }
            }
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub const fn unparenthesized(&self) -> &Expression<'arena> {
        if let Expression::Parenthesized(expression) = self { expression.expression } else { self }
    }

    #[inline]
    #[must_use]
    pub const fn is_assignment(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_assignment()
        } else {
            matches!(&self, Expression::Assignment(_))
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_call(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_call()
        } else {
            matches!(&self, Expression::Call(_))
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_variable(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_variable()
        } else {
            matches!(&self, Expression::Variable(_))
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_binary()
        } else {
            matches!(&self, Expression::Binary(_))
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_unary(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_unary()
        } else {
            matches!(&self, Expression::UnaryPrefix(_) | Expression::UnaryPostfix(_))
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_conditional(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_conditional()
        } else {
            matches!(&self, Expression::Conditional(_))
        }
    }

    #[inline]
    #[must_use]
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
    #[must_use]
    pub const fn is_reference(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_reference()
        } else {
            matches!(&self, Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), .. }))
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_true(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_true()
        } else {
            matches!(&self, Expression::Literal(Literal::True(_)))
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_false(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_false()
        } else {
            matches!(&self, Expression::Literal(Literal::False(_)))
        }
    }

    #[inline]
    #[must_use]
    pub const fn evaluates_to_boolean(&self) -> bool {
        match self {
            Expression::Parenthesized(expression) => expression.expression.evaluates_to_boolean(),
            Expression::Literal(Literal::True(_) | Literal::False(_)) => true,
            Expression::Binary(Binary { operator, .. })
                if operator.is_comparison() || operator.is_logical() || operator.is_instanceof() =>
            {
                true
            }
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_literal(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_literal()
        } else {
            matches!(&self, Expression::Literal(_))
        }
    }

    #[inline]
    #[must_use]
    pub fn is_string_literal(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_string_literal()
        } else {
            matches!(&self, Expression::Literal(Literal::String(_)))
        }
    }

    #[inline]
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub const fn is_throw(&self) -> bool {
        if let Expression::Parenthesized(expression) = self {
            expression.expression.is_throw()
        } else {
            matches!(&self, Expression::Throw(_))
        }
    }

    #[inline]
    #[must_use]
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
            Expression::PartialApplication(_) => NodeKind::PartialApplication,
            Expression::Access(_) => NodeKind::Access,
            Expression::Instantiation(_) => NodeKind::Instantiation,
            Expression::MagicConstant(_) => NodeKind::MagicConstant,
            Expression::Parent(_) => NodeKind::Keyword,
            Expression::Static(_) => NodeKind::Keyword,
            Expression::Self_(_) => NodeKind::Keyword,
            Expression::Pipe(_) => NodeKind::Pipe,
            Expression::Error(_) => NodeKind::Error,
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
            Expression::PartialApplication(expression) => expression.span(),
            Expression::Access(expression) => expression.span(),
            Expression::Parent(expression) => expression.span(),
            Expression::Static(expression) => expression.span(),
            Expression::Self_(expression) => expression.span(),
            Expression::Instantiation(expression) => expression.span(),
            Expression::MagicConstant(expression) => expression.span(),
            Expression::Pipe(expression) => expression.span(),
            Expression::Error(span) => *span,
        }
    }
}
