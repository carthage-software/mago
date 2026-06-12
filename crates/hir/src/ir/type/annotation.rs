use ordered_float::OrderedFloat;
#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::name::Name;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TypeAnnotation<'arena> {
    pub span: Span,
    pub kind: TypeAnnotationKind<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TypeAnnotationKind<'arena> {
    Named(NamedTypeAnnotation<'arena>),
    GenericParameter(GenericParameterTypeAnnotation<'arena>),
    Union(&'arena [TypeAnnotation<'arena>]),
    Intersection(&'arena [TypeAnnotation<'arena>]),
    Array(bool, &'arena TypeAnnotation<'arena>, &'arena TypeAnnotation<'arena>),
    List(bool, &'arena TypeAnnotation<'arena>),
    Iterable(&'arena TypeAnnotation<'arena>, &'arena TypeAnnotation<'arena>),
    ClassLikeString(&'arena TypeAnnotation<'arena>),
    ClassString(&'arena TypeAnnotation<'arena>),
    InterfaceString(&'arena TypeAnnotation<'arena>),
    EnumString(&'arena TypeAnnotation<'arena>),
    TraitString(&'arena TypeAnnotation<'arena>),
    Mixed(bool),
    Null,
    Void,
    Never,
    Resource(Option<bool>),
    Bool(Option<bool>),
    Float(Option<FloatLiteral>),
    Int(Option<IntLiteral>),
    String(StringTypeAnnotation<'arena>),
    StringableObject,
    Object,
    ObjectShape(ObjectShapeTypeAnnotation<'arena>),
    Numeric,
    ArrayKey,
    Scalar,
    MemberReference(Identifier<'arena>, MemberReferenceSelector<'arena>),
    AliasReference(ReferenceKind<'arena>, Name<'arena>),
    Shape(ShapeTypeAnnotation<'arena>),
    Callable(CallableTypeAnnotation<'arena>),
    Variable(DirectVariable<'arena>),
    ThisVariable,
    Conditional(ConditionalTypeAnnotation<'arena>),
    KeyOf(&'arena TypeAnnotation<'arena>),
    ValueOf(&'arena TypeAnnotation<'arena>),
    IntMask(&'arena [TypeAnnotation<'arena>]),
    IntMaskOf(&'arena TypeAnnotation<'arena>),
    New(&'arena TypeAnnotation<'arena>),
    TemplateType(&'arena [TypeAnnotation<'arena>]),
    IndexAccess(&'arena TypeAnnotation<'arena>, &'arena TypeAnnotation<'arena>),
    Negated(&'arena TypeAnnotation<'arena>),
    Posited(&'arena TypeAnnotation<'arena>),
    IntRange(Option<i64>, Option<i64>),
    PropertiesOf(PropertiesOfFilter, &'arena TypeAnnotation<'arena>),
    Slice(&'arena TypeAnnotation<'arena>),
    Wildcard,
    GlobalSelector(GlobalSelector<'arena>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ReferenceKind<'arena> {
    Identifier(Identifier<'arena>),
    Self_(Identifier<'arena>),
    Static(Identifier<'arena>),
    Parent(Identifier<'arena>),
}

impl<'arena> ReferenceKind<'arena> {
    #[must_use]
    pub fn identifier(&self) -> Identifier<'arena> {
        match self {
            ReferenceKind::Identifier(identifier)
            | ReferenceKind::Self_(identifier)
            | ReferenceKind::Static(identifier)
            | ReferenceKind::Parent(identifier) => *identifier,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct NamedTypeAnnotation<'arena> {
    pub span: Span,
    pub kind: ReferenceKind<'arena>,
    pub type_arguments: Option<Delimited<'arena, TypeAnnotation<'arena>>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct GenericParameterTypeAnnotation<'arena> {
    pub span: Span,
    pub name: Name<'arena>,
    pub defining_entity: TypeParameterDefiningEntity<'arena>,
    pub bound: Option<&'arena TypeAnnotation<'arena>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum MemberReferenceSelector<'arena> {
    Wildcard,
    Exact(Name<'arena>),
    StartsWith(Name<'arena>),
    EndsWith(Name<'arena>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum GlobalSelector<'arena> {
    StartsWith(Identifier<'arena>),
    EndsWith(Identifier<'arena>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum IntLiteral {
    Specific(i64),
    Unspecified,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum FloatLiteral {
    Specific(OrderedFloat<f64>),
    Unspecified,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum StringCasing {
    Uppercase,
    Lowercase,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum StringLiteral<'arena> {
    Specific(&'arena [u8]),
    Unspecified,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct StringTypeAnnotation<'arena> {
    pub span: Span,
    pub casing: Option<StringCasing>,
    pub literal: Option<StringLiteral<'arena>>,
    pub non_empty: bool,
    pub numeric: bool,
    pub truthy: bool,
    pub callable: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum PropertiesOfFilter {
    All,
    Public,
    Protected,
    Private,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum CallableTypeKind {
    Callable,
    PureCallable,
    Closure,
    PureClosure,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct CallableTypeAnnotationParameter<'arena> {
    pub span: Span,
    pub r#type: Option<&'arena TypeAnnotation<'arena>>,
    pub variadic: bool,
    pub by_reference: bool,
    pub variable: Option<DirectVariable<'arena>>,
    pub has_default: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct CallableTypeAnnotation<'arena> {
    pub span: Span,
    pub kind: CallableTypeKind,
    pub parameters: Option<Delimited<'arena, CallableTypeAnnotationParameter<'arena>>>,
    pub r#return: Option<&'arena TypeAnnotation<'arena>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ConditionalTypeAnnotation<'arena> {
    pub span: Span,
    pub target: &'arena TypeAnnotation<'arena>,
    pub subject: &'arena TypeAnnotation<'arena>,
    pub is_negated: bool,
    pub then: &'arena TypeAnnotation<'arena>,
    pub r#else: &'arena TypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
pub enum ShapeTypeAnnotationKey<'arena> {
    String(&'arena [u8]),
    Integer(i64),
    ClassLikeConstant(Identifier<'arena>, Name<'arena>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ShapeTypeAnnotationField<'arena> {
    pub span: Span,
    pub key: ShapeTypeAnnotationKey<'arena>,
    pub optional: bool,
    pub value: TypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ShapeTypeAnnotation<'arena> {
    pub span: Span,
    pub fields: Delimited<'arena, ShapeTypeAnnotationField<'arena>>,
    pub additional_fields: Option<ShapeTypeAnnotationAdditionalFields<'arena>>,
    pub is_list: bool,
    pub non_empty: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ShapeTypeAnnotationAdditionalFields<'arena> {
    pub span: Span,
    pub key_type: &'arena TypeAnnotation<'arena>,
    pub value_type: &'arena TypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ObjectShapeTypeAnnotation<'arena> {
    pub span: Span,
    pub fields: Delimited<'arena, ShapeTypeAnnotationField<'arena>>,
    pub sealed: bool,
}

impl HasSpan for TypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for NamedTypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for GenericParameterTypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for StringTypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for CallableTypeAnnotationParameter<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for CallableTypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ConditionalTypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ShapeTypeAnnotationField<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ShapeTypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ShapeTypeAnnotationAdditionalFields<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ObjectShapeTypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for TypeAnnotation<'_> {
    type Output<'arena> = TypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        TypeAnnotation { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl CopyInto for TypeAnnotationKind<'_> {
    type Output<'arena> = TypeAnnotationKind<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            TypeAnnotationKind::Named(named) => TypeAnnotationKind::Named(named.copy_into(arena)),
            TypeAnnotationKind::GenericParameter(parameter) => {
                TypeAnnotationKind::GenericParameter(parameter.copy_into(arena))
            }
            TypeAnnotationKind::Union(kinds) => TypeAnnotationKind::Union(copy_slice_into(kinds, arena)),
            TypeAnnotationKind::Intersection(kinds) => TypeAnnotationKind::Intersection(copy_slice_into(kinds, arena)),
            TypeAnnotationKind::Array(non_empty, key, value) => {
                TypeAnnotationKind::Array(non_empty, copy_ref_into(key, arena), copy_ref_into(value, arena))
            }
            TypeAnnotationKind::List(non_empty, value) => {
                TypeAnnotationKind::List(non_empty, copy_ref_into(value, arena))
            }
            TypeAnnotationKind::Iterable(key, value) => {
                TypeAnnotationKind::Iterable(copy_ref_into(key, arena), copy_ref_into(value, arena))
            }
            TypeAnnotationKind::ClassLikeString(kind) => {
                TypeAnnotationKind::ClassLikeString(copy_ref_into(kind, arena))
            }
            TypeAnnotationKind::ClassString(kind) => TypeAnnotationKind::ClassString(copy_ref_into(kind, arena)),
            TypeAnnotationKind::InterfaceString(kind) => {
                TypeAnnotationKind::InterfaceString(copy_ref_into(kind, arena))
            }
            TypeAnnotationKind::EnumString(kind) => TypeAnnotationKind::EnumString(copy_ref_into(kind, arena)),
            TypeAnnotationKind::TraitString(kind) => TypeAnnotationKind::TraitString(copy_ref_into(kind, arena)),
            TypeAnnotationKind::Mixed(non_empty) => TypeAnnotationKind::Mixed(non_empty),
            TypeAnnotationKind::Null => TypeAnnotationKind::Null,
            TypeAnnotationKind::Void => TypeAnnotationKind::Void,
            TypeAnnotationKind::Never => TypeAnnotationKind::Never,
            TypeAnnotationKind::Resource(value) => TypeAnnotationKind::Resource(value),
            TypeAnnotationKind::Bool(value) => TypeAnnotationKind::Bool(value),
            TypeAnnotationKind::Float(value) => TypeAnnotationKind::Float(value),
            TypeAnnotationKind::Int(value) => TypeAnnotationKind::Int(value),
            TypeAnnotationKind::String(string) => TypeAnnotationKind::String(string.copy_into(arena)),
            TypeAnnotationKind::StringableObject => TypeAnnotationKind::StringableObject,
            TypeAnnotationKind::Object => TypeAnnotationKind::Object,
            TypeAnnotationKind::ObjectShape(shape) => TypeAnnotationKind::ObjectShape(shape.copy_into(arena)),
            TypeAnnotationKind::Numeric => TypeAnnotationKind::Numeric,
            TypeAnnotationKind::ArrayKey => TypeAnnotationKind::ArrayKey,
            TypeAnnotationKind::Scalar => TypeAnnotationKind::Scalar,
            TypeAnnotationKind::MemberReference(identifier, selector) => {
                TypeAnnotationKind::MemberReference(identifier.copy_into(arena), selector.copy_into(arena))
            }
            TypeAnnotationKind::AliasReference(kind, name) => {
                TypeAnnotationKind::AliasReference(kind.copy_into(arena), name.copy_into(arena))
            }
            TypeAnnotationKind::Shape(shape) => TypeAnnotationKind::Shape(shape.copy_into(arena)),
            TypeAnnotationKind::Callable(callable) => TypeAnnotationKind::Callable(callable.copy_into(arena)),
            TypeAnnotationKind::Variable(variable) => TypeAnnotationKind::Variable(variable.copy_into(arena)),
            TypeAnnotationKind::ThisVariable => TypeAnnotationKind::ThisVariable,
            TypeAnnotationKind::Conditional(conditional) => {
                TypeAnnotationKind::Conditional(conditional.copy_into(arena))
            }
            TypeAnnotationKind::KeyOf(kind) => TypeAnnotationKind::KeyOf(copy_ref_into(kind, arena)),
            TypeAnnotationKind::ValueOf(kind) => TypeAnnotationKind::ValueOf(copy_ref_into(kind, arena)),
            TypeAnnotationKind::IntMask(kinds) => TypeAnnotationKind::IntMask(copy_slice_into(kinds, arena)),
            TypeAnnotationKind::IntMaskOf(kind) => TypeAnnotationKind::IntMaskOf(copy_ref_into(kind, arena)),
            TypeAnnotationKind::New(kind) => TypeAnnotationKind::New(copy_ref_into(kind, arena)),
            TypeAnnotationKind::TemplateType(kinds) => TypeAnnotationKind::TemplateType(copy_slice_into(kinds, arena)),
            TypeAnnotationKind::IndexAccess(target, index) => {
                TypeAnnotationKind::IndexAccess(copy_ref_into(target, arena), copy_ref_into(index, arena))
            }
            TypeAnnotationKind::Negated(kind) => TypeAnnotationKind::Negated(copy_ref_into(kind, arena)),
            TypeAnnotationKind::Posited(kind) => TypeAnnotationKind::Posited(copy_ref_into(kind, arena)),
            TypeAnnotationKind::IntRange(minimum, maximum) => TypeAnnotationKind::IntRange(minimum, maximum),
            TypeAnnotationKind::PropertiesOf(filter, kind) => {
                TypeAnnotationKind::PropertiesOf(filter, copy_ref_into(kind, arena))
            }
            TypeAnnotationKind::Slice(kind) => TypeAnnotationKind::Slice(copy_ref_into(kind, arena)),
            TypeAnnotationKind::Wildcard => TypeAnnotationKind::Wildcard,
            TypeAnnotationKind::GlobalSelector(selector) => {
                TypeAnnotationKind::GlobalSelector(selector.copy_into(arena))
            }
        }
    }
}

impl CopyInto for ReferenceKind<'_> {
    type Output<'arena> = ReferenceKind<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            ReferenceKind::Identifier(identifier) => ReferenceKind::Identifier(identifier.copy_into(arena)),
            ReferenceKind::Self_(identifier) => ReferenceKind::Self_(identifier.copy_into(arena)),
            ReferenceKind::Static(identifier) => ReferenceKind::Static(identifier.copy_into(arena)),
            ReferenceKind::Parent(identifier) => ReferenceKind::Parent(identifier.copy_into(arena)),
        }
    }
}

impl CopyInto for NamedTypeAnnotation<'_> {
    type Output<'arena> = NamedTypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        NamedTypeAnnotation {
            span: self.span,
            kind: self.kind.copy_into(arena),
            type_arguments: self.type_arguments.map(|type_arguments| type_arguments.copy_into(arena)),
        }
    }
}

impl CopyInto for GenericParameterTypeAnnotation<'_> {
    type Output<'arena> = GenericParameterTypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        GenericParameterTypeAnnotation {
            span: self.span,
            name: self.name.copy_into(arena),
            defining_entity: self.defining_entity.copy_into(arena),
            bound: self.bound.map(|bound| copy_ref_into(bound, arena)),
        }
    }
}

impl CopyInto for MemberReferenceSelector<'_> {
    type Output<'arena> = MemberReferenceSelector<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            MemberReferenceSelector::Wildcard => MemberReferenceSelector::Wildcard,
            MemberReferenceSelector::Exact(name) => MemberReferenceSelector::Exact(name.copy_into(arena)),
            MemberReferenceSelector::StartsWith(name) => MemberReferenceSelector::StartsWith(name.copy_into(arena)),
            MemberReferenceSelector::EndsWith(name) => MemberReferenceSelector::EndsWith(name.copy_into(arena)),
        }
    }
}

impl CopyInto for GlobalSelector<'_> {
    type Output<'arena> = GlobalSelector<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            GlobalSelector::StartsWith(identifier) => GlobalSelector::StartsWith(identifier.copy_into(arena)),
            GlobalSelector::EndsWith(identifier) => GlobalSelector::EndsWith(identifier.copy_into(arena)),
        }
    }
}

impl CopyInto for StringLiteral<'_> {
    type Output<'arena> = StringLiteral<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            StringLiteral::Specific(value) => StringLiteral::Specific(arena.alloc_slice_copy(value)),
            StringLiteral::Unspecified => StringLiteral::Unspecified,
        }
    }
}

impl CopyInto for StringTypeAnnotation<'_> {
    type Output<'arena> = StringTypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        StringTypeAnnotation {
            span: self.span,
            casing: self.casing,
            literal: self.literal.map(|literal| literal.copy_into(arena)),
            non_empty: self.non_empty,
            numeric: self.numeric,
            truthy: self.truthy,
            callable: self.callable,
        }
    }
}

impl CopyInto for CallableTypeAnnotationParameter<'_> {
    type Output<'arena> = CallableTypeAnnotationParameter<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        CallableTypeAnnotationParameter {
            span: self.span,
            r#type: self.r#type.map(|r#type| copy_ref_into(r#type, arena)),
            variadic: self.variadic,
            by_reference: self.by_reference,
            variable: self.variable.map(|variable| variable.copy_into(arena)),
            has_default: self.has_default,
        }
    }
}

impl CopyInto for CallableTypeAnnotation<'_> {
    type Output<'arena> = CallableTypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        CallableTypeAnnotation {
            span: self.span,
            kind: self.kind,
            parameters: self.parameters.map(|parameters| parameters.copy_into(arena)),
            r#return: self.r#return.map(|r#return| copy_ref_into(r#return, arena)),
        }
    }
}

impl CopyInto for ConditionalTypeAnnotation<'_> {
    type Output<'arena> = ConditionalTypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ConditionalTypeAnnotation {
            span: self.span,
            target: copy_ref_into(self.target, arena),
            subject: copy_ref_into(self.subject, arena),
            is_negated: self.is_negated,
            then: copy_ref_into(self.then, arena),
            r#else: copy_ref_into(self.r#else, arena),
        }
    }
}

impl CopyInto for ShapeTypeAnnotationKey<'_> {
    type Output<'arena> = ShapeTypeAnnotationKey<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            ShapeTypeAnnotationKey::String(value) => ShapeTypeAnnotationKey::String(arena.alloc_slice_copy(value)),
            ShapeTypeAnnotationKey::Integer(value) => ShapeTypeAnnotationKey::Integer(value),
            ShapeTypeAnnotationKey::ClassLikeConstant(identifier, name) => {
                ShapeTypeAnnotationKey::ClassLikeConstant(identifier.copy_into(arena), name.copy_into(arena))
            }
        }
    }
}

impl CopyInto for ShapeTypeAnnotationField<'_> {
    type Output<'arena> = ShapeTypeAnnotationField<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ShapeTypeAnnotationField {
            span: self.span,
            key: self.key.copy_into(arena),
            optional: self.optional,
            value: self.value.copy_into(arena),
        }
    }
}

impl CopyInto for ShapeTypeAnnotation<'_> {
    type Output<'arena> = ShapeTypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ShapeTypeAnnotation {
            span: self.span,
            fields: self.fields.copy_into(arena),
            additional_fields: self.additional_fields.map(|additional_fields| additional_fields.copy_into(arena)),
            is_list: self.is_list,
            non_empty: self.non_empty,
        }
    }
}

impl CopyInto for ShapeTypeAnnotationAdditionalFields<'_> {
    type Output<'arena> = ShapeTypeAnnotationAdditionalFields<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ShapeTypeAnnotationAdditionalFields {
            span: self.span,
            key_type: copy_ref_into(self.key_type, arena),
            value_type: copy_ref_into(self.value_type, arena),
        }
    }
}

impl CopyInto for ObjectShapeTypeAnnotation<'_> {
    type Output<'arena> = ObjectShapeTypeAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ObjectShapeTypeAnnotation { span: self.span, fields: self.fields.copy_into(arena), sealed: self.sealed }
    }
}
