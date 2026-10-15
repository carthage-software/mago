//! Native keyword rules applied to a parsed hint, with the cursor leaf replaced.

use mago_names::{kind::NameKind, scope::NamespaceScope};
use mago_php_version::PHPVersion;
use mago_php_version::feature::Feature;
use mago_span::{HasSpan, Span};
use mago_syntax::cst::{Hint, Keyword, LocalIdentifier};

use super::native_type_context::Position;

pub(super) const KEYWORDS: &[&[u8]] = &[
    b"int",
    b"float",
    b"string",
    b"bool",
    b"array",
    b"object",
    b"callable",
    b"iterable",
    b"mixed",
    b"void",
    b"never",
    b"false",
    b"true",
    b"null",
    b"self",
    b"parent",
    b"static",
];

#[derive(Default)]
pub(super) struct RelativeScope {
    pub allows_self: bool,
    pub allows_parent: bool,
    pub arrow: bool,
}

pub(super) struct Rules<'a> {
    pub version: PHPVersion,
    pub position: Position,
    pub scope: RelativeScope,
    pub names: &'a NamespaceScope,
}

impl Rules<'_> {
    pub(super) fn accepts(&self, root: &Hint<'_>, marker: u32, name: &[u8]) -> bool {
        let Some(candidate) = keyword_hint(name) else { return false };
        if self.position == Position::Property && !self.version.is_supported(Feature::TypedProperties) {
            return false;
        }
        // Before PHP 8.2, false/null require another type: false|null and ?false
        // still count as standalone false for this version check.
        let has_other_type = has_other_type(root, marker, &candidate);
        self.valid(root, marker, &candidate, has_other_type, false)
    }

    fn valid(&self, hint: &Hint<'_>, marker: u32, candidate: &Hint<'_>, has_other_type: bool, in_union: bool) -> bool {
        let hint = replace(hint, marker, candidate);
        let supported = |feature| self.version.is_supported(feature);
        match hint {
            Hint::Nullable(nullable) => {
                let inner = replace(nullable.hint, marker, candidate);
                supported(Feature::NullableTypeHint)
                    && !inner.is_standalone()
                    && !inner.is_complex()
                    && !matches!(inner, Hint::Null(_))
                    && self.valid(nullable.hint, marker, candidate, has_other_type, false)
            }
            Hint::Union(union) => {
                let mut members = Vec::new();
                union_members(hint, marker, candidate, &mut members);
                supported(Feature::NativeUnionTypes)
                    && replace(union.left, marker, candidate).is_unionable()
                    && replace(union.right, marker, candidate).is_unionable()
                    && !members.iter().enumerate().any(|(index, left)| {
                        members[index + 1..].iter().any(|right| redundant(left, right, self.names))
                    })
                    && self.valid(union.left, marker, candidate, has_other_type, true)
                    && self.valid(union.right, marker, candidate, has_other_type, true)
            }
            Hint::Intersection(intersection) => {
                supported(Feature::PureIntersectionTypes)
                    && replace(intersection.left, marker, candidate).is_intersectable()
                    && replace(intersection.right, marker, candidate).is_intersectable()
                    && self.valid(intersection.left, marker, candidate, has_other_type, false)
                    && self.valid(intersection.right, marker, candidate, has_other_type, false)
            }
            Hint::Parenthesized(parenthesized) => {
                in_union
                    && supported(Feature::DisjunctiveNormalForm)
                    && matches!(parenthesized.hint, Hint::Intersection(_))
                    && self.valid(parenthesized.hint, marker, candidate, has_other_type, false)
            }
            Hint::Void(_) => self.position == Position::Return && !self.scope.arrow && supported(Feature::VoidTypeHint),
            Hint::Never(_) => {
                self.position == Position::Return
                    && supported(Feature::NeverTypeHint)
                    && (!self.scope.arrow || supported(Feature::NeverReturnTypeInArrowFunction))
            }
            Hint::Callable(_) => !matches!(self.position, Position::Property | Position::Constant),
            Hint::Static(_) => {
                self.position == Position::Return && self.scope.allows_self && supported(Feature::StaticReturnTypeHint)
            }
            Hint::Self_(_) => self.scope.allows_self,
            Hint::Parent(_) => self.scope.allows_parent,
            Hint::Object(_) => self.version >= PHPVersion::PHP72,
            Hint::Integer(_) | Hint::Float(_) | Hint::String(_) | Hint::Bool(_) => self.version >= PHPVersion::PHP70,
            Hint::Mixed(_) => supported(Feature::MixedTypeHint),
            Hint::Iterable(_) => supported(Feature::IterableTypeHint),
            Hint::True(_) => supported(Feature::TrueTypeHint),
            Hint::False(_) => {
                supported(if has_other_type { Feature::FalseCompoundTypeHint } else { Feature::FalseTypeHint })
            }
            Hint::Null(_) => {
                supported(if has_other_type { Feature::NullCompoundTypeHint } else { Feature::NullTypeHint })
            }
            _ => true,
        }
    }
}

fn replace<'a, 'b>(hint: &'a Hint<'b>, marker: u32, candidate: &'a Hint<'b>) -> &'a Hint<'b> {
    if hint.start_offset() == marker && matches!(hint, Hint::Identifier(_)) { candidate } else { hint }
}

fn has_other_type(hint: &Hint<'_>, marker: u32, candidate: &Hint<'_>) -> bool {
    match replace(hint, marker, candidate) {
        Hint::Nullable(hint) => has_other_type(hint.hint, marker, candidate),
        Hint::Parenthesized(hint) => has_other_type(hint.hint, marker, candidate),
        Hint::Union(hint) => {
            has_other_type(hint.left, marker, candidate) || has_other_type(hint.right, marker, candidate)
        }
        Hint::Intersection(hint) => {
            has_other_type(hint.left, marker, candidate) || has_other_type(hint.right, marker, candidate)
        }
        Hint::Null(_) | Hint::False(_) => false,
        _ => true,
    }
}

fn union_members<'a, 'b>(hint: &'a Hint<'b>, marker: u32, candidate: &'a Hint<'b>, out: &mut Vec<&'a Hint<'b>>) {
    match replace(hint, marker, candidate) {
        Hint::Union(hint) => {
            union_members(hint.left, marker, candidate, out);
            union_members(hint.right, marker, candidate, out);
        }
        hint => out.push(hint),
    }
}

fn redundant(left: &Hint<'_>, right: &Hint<'_>, names: &NamespaceScope) -> bool {
    if std::mem::discriminant(left) == std::mem::discriminant(right)
        && !matches!(left, Hint::Identifier(_) | Hint::Parenthesized(_))
    {
        return true;
    }
    for (left, right) in [(left, right), (right, left)] {
        if matches!(left, Hint::Bool(_)) && matches!(right, Hint::True(_) | Hint::False(_))
            || matches!(left, Hint::True(_)) && matches!(right, Hint::False(_))
            || matches!(left, Hint::Object(_))
                && matches!(
                    right,
                    Hint::Identifier(_) | Hint::Self_(_) | Hint::Parent(_) | Hint::Static(_) | Hint::Parenthesized(_)
                )
            || matches!(left, Hint::Iterable(_)) && matches!(right, Hint::Array(_))
        {
            return true;
        }
        if matches!(left, Hint::Iterable(_))
            && let Hint::Identifier(identifier) = right
            && names.resolve_str(NameKind::Default, identifier.value()).0.eq_ignore_ascii_case(b"Traversable")
        {
            return true;
        }
    }
    false
}

fn keyword_hint(name: &[u8]) -> Option<Hint<'_>> {
    let identifier = LocalIdentifier { span: Span::zero(), value: name };
    let keyword = Keyword { span: Span::zero(), value: name };
    Some(match name {
        b"int" => Hint::Integer(identifier),
        b"float" => Hint::Float(identifier),
        b"string" => Hint::String(identifier),
        b"bool" => Hint::Bool(identifier),
        b"object" => Hint::Object(identifier),
        b"mixed" => Hint::Mixed(identifier),
        b"iterable" => Hint::Iterable(identifier),
        b"void" => Hint::Void(identifier),
        b"never" => Hint::Never(identifier),
        b"array" => Hint::Array(keyword),
        b"callable" => Hint::Callable(keyword),
        b"false" => Hint::False(keyword),
        b"true" => Hint::True(keyword),
        b"null" => Hint::Null(keyword),
        b"static" => Hint::Static(keyword),
        b"self" => Hint::Self_(keyword),
        b"parent" => Hint::Parent(keyword),
        _ => return None,
    })
}
