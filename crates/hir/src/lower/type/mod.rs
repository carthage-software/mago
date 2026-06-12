use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst;

pub mod annotation;

use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::ir::r#type::Type;
use crate::ir::r#type::TypeKind;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_type(&mut self, hint: &'scratch cst::Hint<'scratch>) -> &'arena Type<'arena> {
        self.arena.alloc(Type { span: hint.span(), kind: self.lower_type_kind(hint) })
    }

    fn enclosing_class_or_static(&self, span: Span) -> Identifier<'arena> {
        self.type_resolution.enclosing_class().unwrap_or(Identifier {
            span,
            value: b"static",
            kind: IdentifierKind::Local,
        })
    }

    fn lower_type_kind(&mut self, hint: &'scratch cst::Hint<'scratch>) -> TypeKind<'arena> {
        match hint {
            cst::Hint::Identifier(identifier) => {
                TypeKind::Named(self.lower_identifier(identifier, Some(NameResolutionKind::Default)))
            }
            cst::Hint::Parenthesized(parenthesized) => self.lower_type_kind(parenthesized.hint),
            cst::Hint::Nullable(nullable) => {
                let inner = Type { span: nullable.hint.span(), kind: self.lower_type_kind(nullable.hint) };
                let null = Type { span: nullable.question_mark, kind: TypeKind::Null };

                TypeKind::Union(self.arena.alloc_slice_copy(&[inner, null]))
            }
            cst::Hint::Union(_) => {
                let mut members = mago_allocator::vec::Vec::new_in(self.arena);
                self.collect_union(hint, &mut members);

                TypeKind::Union(members.leak())
            }
            cst::Hint::Intersection(_) => {
                let mut members = mago_allocator::vec::Vec::new_in(self.arena);
                self.collect_intersection(hint, &mut members);

                TypeKind::Intersection(members.leak())
            }
            cst::Hint::Null(_) => TypeKind::Null,
            cst::Hint::True(_) => TypeKind::Bool(Some(true)),
            cst::Hint::False(_) => TypeKind::Bool(Some(false)),
            cst::Hint::Array(_) => TypeKind::Array,
            cst::Hint::Callable(_) => TypeKind::Callable,
            cst::Hint::Static(keyword) => TypeKind::Static(self.enclosing_class_or_static(keyword.span())),
            cst::Hint::Self_(keyword) => TypeKind::Self_(self.enclosing_class_or_static(keyword.span())),
            cst::Hint::Parent(keyword) => {
                TypeKind::Parent(Identifier { span: keyword.span(), value: b"parent", kind: IdentifierKind::Local })
            }
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
        &mut self,
        hint: &'scratch cst::Hint<'scratch>,
        members: &mut mago_allocator::vec::Vec<'arena, Type<'arena>, A>,
    ) {
        match hint {
            cst::Hint::Union(union) => {
                self.collect_union(union.left, members);
                self.collect_union(union.right, members);
            }
            cst::Hint::Parenthesized(parenthesized) => self.collect_union(parenthesized.hint, members),
            _ => members.push(Type { span: hint.span(), kind: self.lower_type_kind(hint) }),
        }
    }

    fn collect_intersection(
        &mut self,
        hint: &'scratch cst::Hint<'scratch>,
        members: &mut mago_allocator::vec::Vec<'arena, Type<'arena>, A>,
    ) {
        match hint {
            cst::Hint::Intersection(intersection) => {
                self.collect_intersection(intersection.left, members);
                self.collect_intersection(intersection.right, members);
            }
            cst::Hint::Parenthesized(parenthesized) => self.collect_intersection(parenthesized.hint, members),
            _ => members.push(Type { span: hint.span(), kind: self.lower_type_kind(hint) }),
        }
    }
}
