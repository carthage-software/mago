use serde::Serialize;

use crate::ir::argument::Argument;
use crate::ir::attribute::Attribute;
use crate::ir::effect::annotation::AssertsAnnotation;
use crate::ir::effect::annotation::ThrowsAnnotation;
use crate::ir::expression::Expression;
use crate::ir::generics::annotation::TypeParameterAnnotation;
use crate::ir::inheritance::ExtendsOne;
use crate::ir::inheritance::Implements;
use crate::ir::inheritance::annotation::ExtendsAnnotation;
use crate::ir::inheritance::annotation::ImplementsAnnotation;
use crate::ir::inheritance::annotation::MixinAnnotation;
use crate::ir::member::ClassLikeConstant;
use crate::ir::member::HookedProperty;
use crate::ir::member::Method;
use crate::ir::member::Property;
use crate::ir::member::TraitUse;
use crate::ir::parameter::Parameter;
use crate::ir::statement::Statement;
use crate::ir::r#type::Type;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DefinitionExpression<'arena, S, D, E> {
    pub meta: D,
    pub kind: DefinitionExpressionKind<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum DefinitionExpressionKind<'arena, S, D, E> {
    AnonymousClass(&'arena AnonymousClass<'arena, S, D, E>),
    ArrowFunction(&'arena ArrowFunction<'arena, S, D, E>),
    Closure(&'arena Closure<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArrowFunction<'arena, S, D, E> {
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub is_static: bool,
    pub type_parameter_annotations: &'arena [TypeParameterAnnotation<'arena>],
    pub parameters: &'arena [Parameter<'arena, S, D, E>],
    pub return_by_reference: bool,
    pub return_type: Option<&'arena Type<'arena>>,
    pub return_type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub throws: &'arena [ThrowsAnnotation<'arena>],
    pub asserts: &'arena [AssertsAnnotation<'arena>],
    pub asserts_if_true: &'arena [AssertsAnnotation<'arena>],
    pub asserts_if_false: &'arena [AssertsAnnotation<'arena>],
    pub expression: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClosureUseClauseVariable<'arena> {
    pub is_by_reference: bool,
    pub variable: DirectVariable<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Closure<'arena, S, D, E> {
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub is_static: bool,
    pub type_parameter_annotations: &'arena [TypeParameterAnnotation<'arena>],
    pub parameters: &'arena [Parameter<'arena, S, D, E>],
    pub return_by_reference: bool,
    pub return_type: Option<&'arena Type<'arena>>,
    pub return_type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub throws: &'arena [ThrowsAnnotation<'arena>],
    pub asserts: &'arena [AssertsAnnotation<'arena>],
    pub asserts_if_true: &'arena [AssertsAnnotation<'arena>],
    pub asserts_if_false: &'arena [AssertsAnnotation<'arena>],
    pub use_variables: &'arena [ClosureUseClauseVariable<'arena>],
    pub body: &'arena Statement<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AnonymousClass<'arena, S, D, E> {
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub arguments: &'arena [Argument<'arena, S, D, E>],
    pub extends: Option<&'arena ExtendsOne<'arena>>,
    pub extends_annotations: &'arena [ExtendsAnnotation<'arena>],
    pub implements: Option<&'arena Implements<'arena>>,
    pub implements_annotations: &'arena [ImplementsAnnotation<'arena>],
    pub mixin_annotations: &'arena [MixinAnnotation<'arena>],
    pub trait_uses: &'arena [TraitUse<'arena>],
    pub constants: &'arena [ClassLikeConstant<'arena, S, D, E>],
    pub properties: &'arena [Property<'arena, S, D, E>],
    pub hooked_properties: &'arena [HookedProperty<'arena, S, D, E>],
    pub methods: &'arena [Method<'arena, S, D, E>],
}
