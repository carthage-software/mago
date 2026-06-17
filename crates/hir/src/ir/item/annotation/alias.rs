#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::identifier::Identifier;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TypeAliasAnnotation<'arena> {
    pub span: Span,
    pub name: Name<'arena>,
    pub r#type: &'arena TypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ImportedTypeAliasAnnotation<'arena> {
    pub span: Span,
    pub name: Name<'arena>,
    pub from: Identifier<'arena>,
    pub r#as: Option<Name<'arena>>,
}

impl CopyInto for TypeAliasAnnotation<'_> {
    type Output<'arena> = TypeAliasAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        TypeAliasAnnotation {
            span: self.span,
            name: self.name.copy_into(arena),
            r#type: copy_ref_into(self.r#type, arena),
        }
    }
}

impl CopyInto for ImportedTypeAliasAnnotation<'_> {
    type Output<'arena> = ImportedTypeAliasAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ImportedTypeAliasAnnotation {
            span: self.span,
            name: self.name.copy_into(arena),
            from: self.from.copy_into(arena),
            r#as: self.r#as.map(|r#as| r#as.copy_into(arena)),
        }
    }
}

impl HasSpan for TypeAliasAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ImportedTypeAliasAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
