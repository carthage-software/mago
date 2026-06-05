use serde::Serialize;

use mago_php_version::PHPVersionRange;
use mago_span::Span;

use crate::ir::attribute::Attribute;
use crate::ir::attribute::AttributeTarget;
use crate::ir::effect::annotation::AssertAnnotation;
use crate::ir::effect::annotation::ThrowsAnnotation;
use crate::ir::expression::Expression;
use crate::ir::flags::Flags;
use crate::ir::generics::annotation::TypeParameterAnnotation;
use crate::ir::generics::annotation::WhereConstraintAnnotation;
use crate::ir::identifier::Identifier;
use crate::ir::inheritance::ExtendsOne;
use crate::ir::inheritance::ExtendsOneOrMore;
use crate::ir::inheritance::Implements;
use crate::ir::inheritance::annotation::ExtendsAnnotation;
use crate::ir::inheritance::annotation::ImplementsAnnotation;
use crate::ir::inheritance::annotation::MixinAnnotation;
use crate::ir::inheritance::annotation::RequireExtendsAnnotation;
use crate::ir::inheritance::annotation::RequireImplementsAnnotation;
use crate::ir::inheritance::annotation::SealedAnnotation;
use crate::ir::member::ClassLikeConstant;
use crate::ir::member::EnumCase;
use crate::ir::member::HookedProperty;
use crate::ir::member::Method;
use crate::ir::member::Property;
use crate::ir::member::TraitUse;
use crate::ir::member::annotation::ImportedTypeAliasAnnotation;
use crate::ir::member::annotation::MethodAnnotation;
use crate::ir::member::annotation::PropertyAnnotation;
use crate::ir::member::annotation::TypeAliasAnnotation;
use crate::ir::modifier::Modifier;
use crate::ir::parameter::Parameter;
use crate::ir::statement::Statement;
use crate::ir::r#type::Type;
use crate::ir::r#type::annotation::TypeAnnotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DefinitionStatement<'arena, S, D, E> {
    pub meta: D,
    pub kind: DefinitionStatementKind<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum DefinitionStatementKind<'arena, S, D, E> {
    Class(&'arena Class<'arena, S, D, E>),
    Interface(&'arena Interface<'arena, S, D, E>),
    Trait(&'arena Trait<'arena, S, D, E>),
    Enum(&'arena Enum<'arena, S, D, E>),
    Constant(&'arena Constant<'arena, S, D, E>),
    Function(&'arena Function<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum EnumFlags {
    Internal = 1 << 2,
    API = 1 << 3,
    Experimental = 1 << 4,
    SealedMethods = 1 << 5,
    UnsealedMethods = 1 << 6,
    Deprecated = 1 << 7,
    HasDocblock = 1 << 8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EnumBackingType<'arena> {
    pub span: Span,
    pub r#type: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Enum<'arena, S, D, E> {
    pub flags: Flags<EnumFlags>,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub attribute_target: Option<Flags<AttributeTarget>>,
    pub name: Identifier<'arena>,
    pub backing_type: Option<EnumBackingType<'arena>>,
    pub type_alias_annotations: &'arena [TypeAliasAnnotation<'arena>],
    pub imported_type_alias_annotations: &'arena [ImportedTypeAliasAnnotation<'arena>],
    pub implements: Option<&'arena Implements<'arena>>,
    pub implements_annotations: &'arena [ImplementsAnnotation<'arena>],
    pub require_extends_annotations: &'arena [RequireExtendsAnnotation<'arena>],
    pub require_implements_annotations: &'arena [RequireImplementsAnnotation<'arena>],
    pub mixin_annotations: &'arena [MixinAnnotation<'arena>],
    pub trait_uses: &'arena [TraitUse<'arena>],
    pub constants: &'arena [ClassLikeConstant<'arena, S, D, E>],
    pub enum_cases: &'arena [EnumCase<'arena, S, D, E>],
    pub methods: &'arena [Method<'arena, S, D, E>],
    pub method_annotations: &'arena [MethodAnnotation<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum TraitFlags {
    Internal = 1 << 2,
    API = 1 << 3,
    Experimental = 1 << 4,
    SealedProperties = 1 << 5,
    UnsealedProperties = 1 << 6,
    SealedMethods = 1 << 7,
    UnsealedMethods = 1 << 8,
    Deprecated = 1 << 9,
    HasDocblock = 1 << 10,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Trait<'arena, S, D, E> {
    pub flags: Flags<TraitFlags>,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub attribute_target: Option<Flags<AttributeTarget>>,
    pub name: Identifier<'arena>,
    pub type_parameter_annotations: &'arena [TypeParameterAnnotation<'arena>],
    pub type_alias_annotations: &'arena [TypeAliasAnnotation<'arena>],
    pub imported_type_alias_annotations: &'arena [ImportedTypeAliasAnnotation<'arena>],
    pub require_extends_annotations: &'arena [RequireExtendsAnnotation<'arena>],
    pub require_implements_annotations: &'arena [RequireImplementsAnnotation<'arena>],
    pub sealed_annotation: Option<&'arena SealedAnnotation<'arena>>,
    pub mixin_annotations: &'arena [MixinAnnotation<'arena>],
    pub trait_uses: &'arena [TraitUse<'arena>],
    pub constants: &'arena [ClassLikeConstant<'arena, S, D, E>],
    pub properties: &'arena [Property<'arena, S, D, E>],
    pub hooked_properties: &'arena [HookedProperty<'arena, S, D, E>],
    pub property_annotations: &'arena [PropertyAnnotation<'arena>],
    pub methods: &'arena [Method<'arena, S, D, E>],
    pub method_annotations: &'arena [MethodAnnotation<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum InterfaceFlags {
    Internal = 1 << 2,
    API = 1 << 3,
    Experimental = 1 << 4,
    EnumInterface = 1 << 5,
    ConsistentConstructor = 1 << 6,
    ConsistentTypeParameterAnnotations = 1 << 7,
    SealedProperties = 1 << 8,
    UnsealedProperties = 1 << 9,
    SealedMethods = 1 << 10,
    UnsealedMethods = 1 << 11,
    Deprecated = 1 << 12,
    HasDocblock = 1 << 13,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Interface<'arena, S, D, E> {
    pub flags: Flags<InterfaceFlags>,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub attribute_target: Option<Flags<AttributeTarget>>,
    pub name: Identifier<'arena>,
    pub type_parameter_annotations: &'arena [TypeParameterAnnotation<'arena>],
    pub type_alias_annotations: &'arena [TypeAliasAnnotation<'arena>],
    pub imported_type_alias_annotations: &'arena [ImportedTypeAliasAnnotation<'arena>],
    pub extends: Option<&'arena ExtendsOneOrMore<'arena>>,
    pub extends_annotations: &'arena [ExtendsAnnotation<'arena>],
    pub require_extends_annotations: &'arena [RequireExtendsAnnotation<'arena>],
    pub require_implements_annotations: &'arena [RequireImplementsAnnotation<'arena>],
    pub sealed_annotation: Option<&'arena SealedAnnotation<'arena>>,
    pub mixin_annotations: &'arena [MixinAnnotation<'arena>],
    pub constants: &'arena [ClassLikeConstant<'arena, S, D, E>],
    pub hooked_properties: &'arena [HookedProperty<'arena, S, D, E>],
    pub methods: &'arena [Method<'arena, S, D, E>],
    pub method_annotations: &'arena [MethodAnnotation<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum ClassFlags {
    Internal = 1 << 2,
    API = 1 << 3,
    Experimental = 1 << 4,
    ConsistentConstructor = 1 << 5,
    ConsistentTypeParameterAnnotations = 1 << 6,
    SealedProperties = 1 << 7,
    UnsealedProperties = 1 << 8,
    SealedMethods = 1 << 9,
    UnsealedMethods = 1 << 10,
    Deprecated = 1 << 11,
    HasDocblock = 1 << 12,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Class<'arena, S, D, E> {
    pub flags: Flags<ClassFlags>,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub attribute_target: Option<Flags<AttributeTarget>>,
    pub name: Identifier<'arena>,
    pub type_parameter_annotations: &'arena [TypeParameterAnnotation<'arena>],
    pub modifiers: &'arena [Modifier],
    pub type_alias_annotations: &'arena [TypeAliasAnnotation<'arena>],
    pub imported_type_alias_annotations: &'arena [ImportedTypeAliasAnnotation<'arena>],
    pub extends: Option<&'arena ExtendsOne<'arena>>,
    pub extends_annotations: &'arena [ExtendsAnnotation<'arena>],
    pub implements: Option<&'arena Implements<'arena>>,
    pub implements_annotations: &'arena [ImplementsAnnotation<'arena>],
    pub require_extends_annotations: &'arena [RequireExtendsAnnotation<'arena>],
    pub require_implements_annotations: &'arena [RequireImplementsAnnotation<'arena>],
    pub sealed_annotation: Option<&'arena SealedAnnotation<'arena>>,
    pub mixin_annotations: &'arena [MixinAnnotation<'arena>],
    pub trait_uses: &'arena [TraitUse<'arena>],
    pub constants: &'arena [ClassLikeConstant<'arena, S, D, E>],
    pub properties: &'arena [Property<'arena, S, D, E>],
    pub hooked_properties: &'arena [HookedProperty<'arena, S, D, E>],
    pub property_annotations: &'arena [PropertyAnnotation<'arena>],
    pub methods: &'arena [Method<'arena, S, D, E>],
    pub method_annotations: &'arena [MethodAnnotation<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum ConstantFlags {
    Internal = 1 << 2,
    API = 1 << 3,
    Experimental = 1 << 4,
    Deprecated = 1 << 5,
    HasDocblock = 1 << 6,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Constant<'arena, S, D, E> {
    pub flags: Flags<ConstantFlags>,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub items: &'arena [ConstantItem<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConstantItem<'arena, S, D, E> {
    pub name: Identifier<'arena>,
    pub value: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum FunctionFlags {
    Deprecated = 1 << 0,
    Experimental = 1 << 2,
    Internal = 1 << 3,
    API = 1 << 4,
    Pure = 1 << 5,
    SuspendsFiber = 1 << 6,
    IgnoreNullableReturnType = 1 << 7,
    IgnoreFalsableReturnType = 1 << 8,
    NoNamedArguments = 1 << 9,
    MustUse = 1 << 10,
    HasDocblock = 1 << 11,
    MutationFree = 1 << 12,
    ExternalMutationFree = 1 << 13,
    AssertionsInferred = 1 << 14,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Function<'arena, S, D, E> {
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub flags: Flags<FunctionFlags>,
    pub name: Identifier<'arena>,
    pub type_parameter_annotations: &'arena [TypeParameterAnnotation<'arena>],
    pub parameters: &'arena [Parameter<'arena, S, D, E>],
    pub where_constraint_annotations: &'arena [WhereConstraintAnnotation<'arena>],
    pub return_by_reference: bool,
    pub return_type: Option<&'arena Type<'arena>>,
    pub return_type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub throws_annotations: &'arena [ThrowsAnnotation<'arena>],
    pub assert_annotations: &'arena [AssertAnnotation<'arena>],
    pub assert_if_true_annotations: &'arena [AssertAnnotation<'arena>],
    pub assert_if_false_annotations: &'arena [AssertAnnotation<'arena>],
    pub body: &'arena Statement<'arena, S, D, E>,
}

impl From<EnumFlags> for u32 {
    fn from(value: EnumFlags) -> Self {
        value as u32
    }
}

impl From<TraitFlags> for u32 {
    fn from(value: TraitFlags) -> Self {
        value as u32
    }
}

impl From<InterfaceFlags> for u32 {
    fn from(value: InterfaceFlags) -> Self {
        value as u32
    }
}

impl From<ClassFlags> for u32 {
    fn from(value: ClassFlags) -> Self {
        value as u32
    }
}

impl From<ConstantFlags> for u32 {
    fn from(value: ConstantFlags) -> Self {
        value as u32
    }
}

impl From<FunctionFlags> for u32 {
    fn from(value: FunctionFlags) -> Self {
        value as u32
    }
}
