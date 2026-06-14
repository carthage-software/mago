#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;
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

impl CopyInto for ItemAnnotationTag {
    type Output<'arena> = ItemAnnotationTag;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl<I, S, E> CopyInto for ItemAnnotation<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ItemAnnotation<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ItemAnnotation {
            span: self.span,
            type_aliases: copy_slice_into(self.type_aliases, arena),
            imported_type_aliases: copy_slice_into(self.imported_type_aliases, arena),
            type_parameters: copy_slice_into(self.type_parameters, arena),
            inherited_type_parameters: copy_slice_into(self.inherited_type_parameters, arena),
            extends: copy_slice_into(self.extends, arena),
            require_extends: copy_slice_into(self.require_extends, arena),
            implements: copy_slice_into(self.implements, arena),
            require_implements: copy_slice_into(self.require_implements, arena),
            uses: copy_slice_into(self.uses, arena),
            sealings: copy_slice_into(self.sealings, arena),
            mixins: copy_slice_into(self.mixins, arena),
            methods: copy_slice_into(self.methods, arena),
            properties: copy_slice_into(self.properties, arena),
            parameters: copy_slice_into(self.parameters, arena),
            parameter_outs: copy_slice_into(self.parameter_outs, arena),
            where_constraints: copy_slice_into(self.where_constraints, arena),
            return_type: copy_slice_into(self.return_type, arena),
            throws: copy_slice_into(self.throws, arena),
            asserts: copy_slice_into(self.asserts, arena),
            asserts_if_true: copy_slice_into(self.asserts_if_true, arena),
            asserts_if_false: copy_slice_into(self.asserts_if_false, arena),
            self_out: copy_slice_into(self.self_out, arena),
            pure_unless_callable_impure: copy_slice_into(self.pure_unless_callable_impure, arena),
            var: copy_slice_into(self.var, arena),
            tags: self.tags,
            errors: arena.alloc_slice_copy(self.errors),
        }
    }
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
