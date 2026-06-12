#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U32Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::error::annotation::AnnotationError;
use crate::ir::item::annotation::alias::ImportedTypeAliasAnnotation;
use crate::ir::item::annotation::alias::TypeAliasAnnotation;
use crate::ir::item::annotation::effect::AssertAnnotation;
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
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use crate::ir::variable::annotation::VariableAnnotation;

pub mod alias;
pub mod effect;
pub mod generics;
pub mod inheritance;
pub mod member;
pub mod parameter;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u32)]
pub enum ItemAnnotationTag {
    Abstract = 1 << 0,
    Deprecated = 1 << 1,
    Example = 1 << 2,
    Final = 1 << 3,
    Internal = 1 << 4,
    Api = 1 << 5,
    Experimental = 1 << 6,
    Readonly = 1 << 7,
    Immutable = 1 << 8,
    InheritDoc = 1 << 9,
    Pure = 1 << 10,
    Impure = 1 << 11,
    MutationFree = 1 << 12,
    ExternalMutationFree = 1 << 13,
    SuspendsFiber = 1 << 14,
    IgnoreNullableReturnType = 1 << 15,
    IgnoreFalsableReturnType = 1 << 16,
    NoNamedArguments = 1 << 17,
    MustUse = 1 << 18,
    ConsistentConstructor = 1 << 19,
    ConsistentTypeParameterAnnotations = 1 << 20,
    SealProperties = 1 << 21,
    NoSealProperties = 1 << 22,
    SealMethods = 1 << 23,
    NoSealMethods = 1 << 24,
    EnumInterface = 1 << 25,
    NotDeprecated = 1 << 26,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ItemAnnotation<'arena, I, S, E> {
    pub span: Span,
    pub type_aliases: &'arena [TypeAliasAnnotation<'arena>],
    pub imported_type_aliases: &'arena [ImportedTypeAliasAnnotation<'arena>],
    pub type_parameters: &'arena [TypeParameterAnnotation<'arena>],
    pub inherited_type_parameters: &'arena [InheritedTypeParameterAnnotation<'arena>],
    pub extends: &'arena [ExtendsAnnotation<'arena>],
    pub require_extends: &'arena [RequireExtendsAnnotation<'arena>],
    pub implements: &'arena [ImplementsAnnotation<'arena>],
    pub require_implements: &'arena [RequireImplementsAnnotation<'arena>],
    pub uses: &'arena [UseAnnotation<'arena>],
    pub sealings: &'arena [SealedAnnotation<'arena>],
    pub mixins: &'arena [MixinAnnotation<'arena>],
    pub methods: &'arena [MethodAnnotation<'arena, I, S, E>],
    pub properties: &'arena [PropertyAnnotation<'arena>],
    pub parameters: &'arena [ParameterAnnotation<'arena, I, S, E>],
    pub parameter_outs: &'arena [ParameterOutAnnotation<'arena>],
    pub where_constraints: &'arena [WhereConstraintAnnotation<'arena>],
    pub return_type: &'arena [TypeAnnotation<'arena>],
    pub throws: &'arena [ThrowsAnnotation<'arena>],
    pub asserts: &'arena [AssertAnnotation<'arena>],
    pub asserts_if_true: &'arena [AssertAnnotation<'arena>],
    pub asserts_if_false: &'arena [AssertAnnotation<'arena>],
    pub self_out: &'arena [SelfOutAnnotation<'arena>],
    pub pure_unless_callable_impure: &'arena [DirectVariable<'arena>],
    pub var: &'arena [VariableAnnotation<'arena>],
    pub tags: U32Flags<ItemAnnotationTag>,
    pub errors: &'arena [AnnotationError],
}

impl<I, S, E> HasSpan for ItemAnnotation<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl From<ItemAnnotationTag> for u32 {
    fn from(tag: ItemAnnotationTag) -> Self {
        tag as u32
    }
}
