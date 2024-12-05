use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::access::Access;
use crate::ast::access::ClassConstantAccess;
use crate::ast::access::NullSafePropertyAccess;
use crate::ast::access::PropertyAccess;
use crate::ast::argument::Argument;
use crate::ast::array::Array;
use crate::ast::array::ArrayAccess;
use crate::ast::array::ArrayAppend;
use crate::ast::array::ArrayElement;
use crate::ast::array::LegacyArray;
use crate::ast::array::List;
use crate::ast::call::Call;
use crate::ast::class_like::member::ClassLikeConstantSelector;
use crate::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::class_like::AnonymousClass;
use crate::ast::clone::Clone;
use crate::ast::closure_creation::ClosureCreation;
use crate::ast::construct::Construct;
use crate::ast::control_flow::r#match::Match;
use crate::ast::function_like::arrow_function::ArrowFunction;
use crate::ast::function_like::closure::Closure;
use crate::ast::identifier::Identifier;
use crate::ast::instantiation::Instantiation;
use crate::ast::keyword::Keyword;
use crate::ast::literal::Literal;
use crate::ast::magic_constant::MagicConstant;
use crate::ast::operation::arithmetic::ArithmeticOperation;
use crate::ast::operation::arithmetic::ArithmeticPrefixOperator;
use crate::ast::operation::assignment::AssignmentOperation;
use crate::ast::operation::bitwise::BitwiseOperation;
use crate::ast::operation::cast::CastOperation;
use crate::ast::operation::coalesce::CoalesceOperation;
use crate::ast::operation::comparison::ComparisonOperation;
use crate::ast::operation::concat::ConcatOperation;
use crate::ast::operation::instanceof::InstanceofOperation;
use crate::ast::operation::logical::LogicalOperation;
use crate::ast::operation::ternary::TernaryOperation;
use crate::ast::r#yield::Yield;
use crate::ast::string::CompositeString;
use crate::ast::string::StringPart;
use crate::ast::throw::Throw;
use crate::ast::variable::Variable;
use crate::node::NodeKind;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Parenthesized {
    pub left_parenthesis: Span,
    pub expression: Expression,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Referenced {
    pub ampersand: Span,
    pub expression: Expression,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Suppressed {
    pub at: Span,
    pub expression: Expression,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Expression {
    Parenthesized(Box<Parenthesized>),
    Referenced(Box<Referenced>),
    Suppressed(Box<Suppressed>),
    Literal(Literal),
    CompositeString(Box<CompositeString>),
    ArithmeticOperation(Box<ArithmeticOperation>),
    AssignmentOperation(Box<AssignmentOperation>),
    BitwiseOperation(Box<BitwiseOperation>),
    ComparisonOperation(Box<ComparisonOperation>),
    LogicalOperation(Box<LogicalOperation>),
    CastOperation(Box<CastOperation>),
    TernaryOperation(Box<TernaryOperation>),
    CoalesceOperation(Box<CoalesceOperation>),
    ConcatOperation(Box<ConcatOperation>),
    InstanceofOperation(Box<InstanceofOperation>),
    Array(Box<Array>),
    LegacyArray(Box<LegacyArray>),
    List(Box<List>),
    ArrayAccess(Box<ArrayAccess>),
    ArrayAppend(Box<ArrayAppend>),
    AnonymousClass(Box<AnonymousClass>),
    Closure(Box<Closure>),
    ArrowFunction(Box<ArrowFunction>),
    Variable(Variable),
    Identifier(Identifier),
    Match(Box<Match>),
    Yield(Box<Yield>),
    Construct(Box<Construct>),
    Throw(Box<Throw>),
    Clone(Box<Clone>),
    Call(Call),
    Access(Box<Access>),
    ClosureCreation(Box<ClosureCreation>),
    Parent(Keyword),
    Static(Keyword),
    Self_(Keyword),
    Instantiation(Box<Instantiation>),
    MagicConstant(MagicConstant),
}

impl Expression {
    pub fn is_constant(&self, initilization: bool) -> bool {
        match &self {
            Self::Literal(_) => true,
            Self::Identifier(_) => true,
            Self::MagicConstant(_) => true,
            Self::Self_(_) => true,
            Self::Parent(_) => true,
            Self::Static(_) => true,
            Self::Parenthesized(expression) => expression.expression.is_constant(initilization),
            Self::ArithmeticOperation(expression) => match expression.as_ref() {
                ArithmeticOperation::Infix(arithmetic_infix_operation) => {
                    arithmetic_infix_operation.lhs.is_constant(initilization)
                        && arithmetic_infix_operation.rhs.is_constant(initilization)
                }
                ArithmeticOperation::Prefix(arithmetic_prefix_operation) => {
                    match arithmetic_prefix_operation.operator {
                        ArithmeticPrefixOperator::Plus(_) | ArithmeticPrefixOperator::Minus(_) => {
                            arithmetic_prefix_operation.value.is_constant(initilization)
                        }
                        _ => false,
                    }
                }
                ArithmeticOperation::Postfix(_) => false,
            },
            Self::Access(access) => match access.as_ref() {
                Access::ClassConstant(ClassConstantAccess { class, constant, .. }) => {
                    matches!(constant, ClassLikeConstantSelector::Identifier(_)) && class.is_constant(initilization)
                }
                Access::Property(PropertyAccess { object, property, .. }) => {
                    matches!(property, ClassLikeMemberSelector::Identifier(_)) && object.is_constant(initilization)
                }
                Access::NullSafeProperty(NullSafePropertyAccess { object, property, .. }) => {
                    matches!(property, ClassLikeMemberSelector::Identifier(_)) && object.is_constant(initilization)
                }
                _ => false,
            },
            Self::ArrayAccess(access) => {
                access.array.is_constant(initilization) && access.index.is_constant(initilization)
            }
            Self::BitwiseOperation(expression) => match expression.as_ref() {
                BitwiseOperation::Prefix(bitwise_prefix_operation) => {
                    bitwise_prefix_operation.value.is_constant(initilization)
                }
                BitwiseOperation::Infix(bitwise_infix_operation) => {
                    bitwise_infix_operation.lhs.is_constant(initilization)
                        && bitwise_infix_operation.rhs.is_constant(initilization)
                }
            },
            Self::Instantiation(instantiation) if initilization => {
                instantiation.class.is_constant(initilization)
                    && instantiation
                        .arguments
                        .as_ref()
                        .map(|arguments| {
                            arguments.arguments.iter().all(|argument| match &argument {
                                Argument::Positional(positional_argument) => {
                                    positional_argument.ellipsis.is_none()
                                        && positional_argument.value.is_constant(initilization)
                                }
                                Argument::Named(named_argument) => {
                                    named_argument.ellipsis.is_none() && named_argument.value.is_constant(initilization)
                                }
                            })
                        })
                        .unwrap_or(true)
            }
            Self::CoalesceOperation(operation) => {
                operation.lhs.is_constant(initilization) && operation.rhs.is_constant(initilization)
            }
            Self::ComparisonOperation(operation) => {
                operation.lhs.is_constant(initilization) && operation.rhs.is_constant(initilization)
            }
            Self::LogicalOperation(operation) => match operation.as_ref() {
                LogicalOperation::Prefix(logical_prefix_operation) => {
                    logical_prefix_operation.value.is_constant(initilization)
                }
                LogicalOperation::Infix(logical_infix_operation) => {
                    logical_infix_operation.lhs.is_constant(initilization)
                        && logical_infix_operation.rhs.is_constant(initilization)
                }
            },
            Self::ConcatOperation(operation) => {
                operation.lhs.is_constant(initilization) && operation.rhs.is_constant(initilization)
            }
            Self::TernaryOperation(operation) => match operation.as_ref() {
                TernaryOperation::Conditional(conditional_ternary_operation) => {
                    conditional_ternary_operation.condition.is_constant(initilization)
                        && conditional_ternary_operation
                            .then
                            .as_ref()
                            .map(|e| e.is_constant(initilization))
                            .unwrap_or(true)
                        && conditional_ternary_operation.r#else.is_constant(initilization)
                }
                TernaryOperation::Elvis(elvis_ternary_operation) => {
                    elvis_ternary_operation.condition.is_constant(initilization)
                        && elvis_ternary_operation.r#else.is_constant(initilization)
                }
            },
            Self::Array(array) => array.elements.inner.iter().all(|element| match &element {
                ArrayElement::KeyValue(key_value_array_element) => {
                    key_value_array_element.key.is_constant(initilization)
                        && key_value_array_element.value.is_constant(initilization)
                }
                ArrayElement::Value(value_array_element) => value_array_element.value.is_constant(initilization),
                ArrayElement::Variadic(variadic_array_element) => {
                    variadic_array_element.value.is_constant(initilization)
                }
                ArrayElement::Missing(_) => false,
            }),
            Self::LegacyArray(array) => array.elements.inner.iter().all(|element| match &element {
                ArrayElement::KeyValue(key_value_array_element) => {
                    key_value_array_element.key.is_constant(initilization)
                        && key_value_array_element.value.is_constant(initilization)
                }
                ArrayElement::Value(value_array_element) => value_array_element.value.is_constant(initilization),
                ArrayElement::Variadic(variadic_array_element) => {
                    variadic_array_element.value.is_constant(initilization)
                }
                ArrayElement::Missing(_) => false,
            }),
            Self::CompositeString(string) => match string.as_ref() {
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
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match &self {
            Expression::ArithmeticOperation(arithmetic) => {
                matches!(arithmetic.as_ref(), ArithmeticOperation::Infix(_))
            }
            Expression::BitwiseOperation(bitwise) => {
                matches!(bitwise.as_ref(), BitwiseOperation::Infix(_))
            }
            Expression::LogicalOperation(logical) => {
                matches!(logical.as_ref(), LogicalOperation::Infix(_))
            }
            Expression::ComparisonOperation(_) => true,
            Expression::ConcatOperation(_) => true,
            Expression::CoalesceOperation(_) => true,
            _ => false,
        }
    }

    pub fn is_unary(&self) -> bool {
        match &self {
            Expression::Suppressed(_) => true,
            Expression::Referenced(_) => true,
            Expression::ArithmeticOperation(arithmetic) => {
                matches!(arithmetic.as_ref(), ArithmeticOperation::Prefix(_))
            }
            Expression::BitwiseOperation(bitwise) => {
                matches!(bitwise.as_ref(), BitwiseOperation::Prefix(_))
            }
            Expression::LogicalOperation(logical) => {
                matches!(logical.as_ref(), LogicalOperation::Prefix(_))
            }
            _ => false,
        }
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, Expression::Literal(_))
    }

    pub fn is_string_literal(&self) -> bool {
        match &self {
            Expression::Literal(literal) => match literal {
                Literal::String(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn node_kind(&self) -> NodeKind {
        match &self {
            Expression::Parenthesized(_) => NodeKind::Parenthesized,
            Expression::Referenced(_) => NodeKind::Referenced,
            Expression::Suppressed(_) => NodeKind::Suppressed,
            Expression::Literal(_) => NodeKind::Literal,
            Expression::CompositeString(_) => NodeKind::CompositeString,
            Expression::ArithmeticOperation(_) => NodeKind::ArithmeticOperation,
            Expression::AssignmentOperation(_) => NodeKind::AssignmentOperation,
            Expression::BitwiseOperation(_) => NodeKind::BitwiseOperation,
            Expression::ComparisonOperation(_) => NodeKind::ComparisonOperation,
            Expression::LogicalOperation(_) => NodeKind::LogicalOperation,
            Expression::CastOperation(_) => NodeKind::CastOperation,
            Expression::TernaryOperation(_) => NodeKind::TernaryOperation,
            Expression::CoalesceOperation(_) => NodeKind::CoalesceOperation,
            Expression::ConcatOperation(_) => NodeKind::ConcatOperation,
            Expression::InstanceofOperation(_) => NodeKind::InstanceofOperation,
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
        }
    }
}

impl HasSpan for Parenthesized {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}

impl HasSpan for Referenced {
    fn span(&self) -> Span {
        self.ampersand.join(self.expression.span())
    }
}

impl HasSpan for Suppressed {
    fn span(&self) -> Span {
        self.at.join(self.expression.span())
    }
}

impl HasSpan for Expression {
    fn span(&self) -> Span {
        match &self {
            Expression::Parenthesized(expression) => expression.span(),
            Expression::Referenced(expression) => expression.span(),
            Expression::Suppressed(expression) => expression.span(),
            Expression::Literal(expression) => expression.span(),
            Expression::CompositeString(expression) => expression.span(),
            Expression::ArithmeticOperation(expression) => expression.span(),
            Expression::AssignmentOperation(expression) => expression.span(),
            Expression::BitwiseOperation(expression) => expression.span(),
            Expression::ComparisonOperation(expression) => expression.span(),
            Expression::LogicalOperation(expression) => expression.span(),
            Expression::CastOperation(expression) => expression.span(),
            Expression::TernaryOperation(expression) => expression.span(),
            Expression::CoalesceOperation(expression) => expression.span(),
            Expression::ConcatOperation(expression) => expression.span(),
            Expression::InstanceofOperation(expression) => expression.span(),
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
        }
    }
}
