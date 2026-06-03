use mago_span::HasSpan;
use mago_syntax::cst;

pub mod annotation;

use crate::ir::r#type::Type;
use crate::ir::r#type::TypeKind;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_type(&self, hint: &'arena cst::Hint<'arena>) -> &'arena Type<'arena> {
        self.arena.alloc(Type { span: hint.span(), kind: self.lower_type_kind(hint) })
    }

    fn lower_type_kind(&self, hint: &'arena cst::Hint<'arena>) -> TypeKind<'arena> {
        match hint {
            cst::Hint::Identifier(identifier) => {
                TypeKind::Named(self.lower_identifier(identifier, Some(NameResolutionKind::Default)))
            }
            cst::Hint::Parenthesized(parenthesized) => self.lower_type_kind(parenthesized.hint),
            cst::Hint::Nullable(nullable) => {
                let inner = self.lower_type_kind(nullable.hint);

                TypeKind::Union(self.arena.alloc_slice_copy(&[inner, TypeKind::Null]))
            }
            cst::Hint::Union(_) => {
                let mut members = bumpalo::collections::Vec::new_in(self.arena);
                self.collect_union(hint, &mut members);

                TypeKind::Union(members.into_bump_slice())
            }
            cst::Hint::Intersection(_) => {
                let mut members = bumpalo::collections::Vec::new_in(self.arena);
                self.collect_intersection(hint, &mut members);

                TypeKind::Intersection(members.into_bump_slice())
            }
            cst::Hint::Null(_) => TypeKind::Null,
            cst::Hint::True(_) => TypeKind::Bool(Some(true)),
            cst::Hint::False(_) => TypeKind::Bool(Some(false)),
            cst::Hint::Array(_) => TypeKind::Array,
            cst::Hint::Callable(_) => TypeKind::Callable,
            cst::Hint::Static(_) => TypeKind::Static,
            cst::Hint::Self_(_) => TypeKind::Self_,
            cst::Hint::Parent(_) => TypeKind::Parent,
            cst::Hint::Void(_) => TypeKind::Void,
            cst::Hint::Never(_) => TypeKind::Never,
            cst::Hint::Float(_) => TypeKind::Float,
            cst::Hint::Bool(_) => TypeKind::Bool(None),
            cst::Hint::Integer(_) => TypeKind::Integer,
            cst::Hint::String(_) => TypeKind::String,
            cst::Hint::Object(_) => TypeKind::Object,
            cst::Hint::Mixed(_) => TypeKind::Mixed,
            cst::Hint::Iterable(_) => TypeKind::Iterable,
        }
    }

    fn collect_union(
        &self,
        hint: &'arena cst::Hint<'arena>,
        members: &mut bumpalo::collections::Vec<'arena, TypeKind<'arena>>,
    ) {
        match hint {
            cst::Hint::Union(union) => {
                self.collect_union(union.left, members);
                self.collect_union(union.right, members);
            }
            cst::Hint::Parenthesized(parenthesized) => self.collect_union(parenthesized.hint, members),
            _ => members.push(self.lower_type_kind(hint)),
        }
    }

    fn collect_intersection(
        &self,
        hint: &'arena cst::Hint<'arena>,
        members: &mut bumpalo::collections::Vec<'arena, TypeKind<'arena>>,
    ) {
        match hint {
            cst::Hint::Intersection(intersection) => {
                self.collect_intersection(intersection.left, members);
                self.collect_intersection(intersection.right, members);
            }
            cst::Hint::Parenthesized(parenthesized) => self.collect_intersection(parenthesized.hint, members),
            _ => members.push(self.lower_type_kind(hint)),
        }
    }
}
