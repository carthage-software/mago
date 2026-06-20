//! Shape-level sugar on [`TypeBuilder`]: constructors that encode a
//! convention (literal-string refinement derivation, sealed-list rest type,
//! the "mixed" callable signature) rather than just interning a payload.

use std::num::NonZeroU32;

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::symbol::class_like::ClassLikeKind;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::object::enumeration::EnumAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known;

fn is_numeric_string(value: &[u8]) -> bool {
    core::str::from_utf8(value).is_ok_and(|text| text.parse::<i64>().is_ok() || text.parse::<f64>().is_ok())
}

impl<'arena, S, A> TypeBuilder<'_, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// Intern a string literal atom. Refinement properties (`is_numeric`,
    /// `is_truthy`, `is_non_empty`) and casing are derived from the value:
    /// `"hello"` is non-empty, truthy, and lowercase; `""` is none of
    /// those; `"123"` is numeric and truthy.
    #[must_use]
    pub fn string_literal(&mut self, value: &[u8]) -> Atom<'arena> {
        let is_numeric = is_numeric_string(value);
        let is_non_empty = is_numeric || !value.is_empty();
        let is_truthy = is_non_empty && value != b"0";
        let has_lower = value.iter().any(u8::is_ascii_lowercase);
        let has_upper = value.iter().any(u8::is_ascii_uppercase);
        let casing = if has_lower && !has_upper {
            StringCasing::Lowercase
        } else if has_upper && !has_lower {
            StringCasing::Uppercase
        } else {
            StringCasing::Unspecified
        };

        let mut flags = U8Flags::empty();
        flags.set_value(StringRefinementFlag::Numeric, is_numeric);
        flags.set_value(StringRefinementFlag::Truthy, is_truthy);
        flags.set_value(StringRefinementFlag::NonEmpty, is_non_empty);

        let literal = StringLiteral::Value(self.intern(value));

        self.string(StringAtom { literal, casing, flags })
    }

    /// Intern a named object atom with no type arguments and default flags.
    #[must_use]
    pub fn object_named(&mut self, name: &[u8]) -> Atom<'arena> {
        let name = self.intern_class_like_path(name);

        self.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() })
    }

    /// Intern an enum atom ("any case of enum `name`").
    #[must_use]
    pub fn enum_any(&mut self, name: &[u8]) -> Atom<'arena> {
        let name = self.intern_class_like_path(name);

        self.enumeration(EnumAtom { name, case: None })
    }

    /// Intern an enum-case atom (`name::case`).
    #[must_use]
    pub fn enum_case(&mut self, name: &[u8], case: &[u8]) -> Atom<'arena> {
        let name = self.intern_class_like_path(name);
        let case = self.intern(case);

        self.enumeration(EnumAtom { name, case: Some(case) })
    }

    /// Intern a literal class-string atom (`class-string` with a concrete
    /// name).
    #[must_use]
    pub fn class_string_literal(&mut self, name: &[u8]) -> Atom<'arena> {
        let value = self.intern_class_like_path(name);

        self.class_like_string(ClassLikeStringAtom {
            kind: ClassLikeKind::Class,
            specifier: ClassLikeStringSpecifier::Literal { value },
        })
    }

    /// Intern a `list<element>` (or `non-empty-list<element>`) atom with no
    /// fixed-position elements.
    #[must_use]
    pub fn list_of(&mut self, element_type: Type<'arena>, non_empty: bool) -> Atom<'arena> {
        let mut flags = U8Flags::empty();
        flags.set_value(ListFlag::NonEmpty, non_empty);

        self.list(ListAtom { element_type, known_elements: None, known_count: None, flags })
    }

    /// Intern a sealed list atom (`list{0: T0, 1: T1, …}`) with the given
    /// known entries and no rest element type.
    #[must_use]
    pub fn sealed_list(&mut self, elements: &[KnownElement<'arena>], non_empty: bool) -> Atom<'arena> {
        let known_count = NonZeroU32::new(elements.len() as u32);
        let known_elements = Some(self.known_elements(elements));
        let mut flags = U8Flags::empty();
        flags.set_value(ListFlag::NonEmpty, non_empty);

        self.list(ListAtom { element_type: well_known::TYPE_NEVER, known_elements, known_count, flags })
    }

    /// Intern an unsealed keyed-array atom (`array<K, V>` /
    /// `non-empty-array<K, V>`) with no known fixed entries.
    #[must_use]
    pub fn keyed_unsealed(
        &mut self,
        key_type: Type<'arena>,
        value_type: Type<'arena>,
        non_empty: bool,
    ) -> Atom<'arena> {
        let mut flags = U8Flags::empty();
        flags.set_value(ArrayFlag::NonEmpty, non_empty);

        self.array(ArrayAtom { key_param: Some(key_type), value_param: Some(value_type), known_items: None, flags })
    }

    /// Intern a sealed keyed-array atom (`array{a: int, b: string, …}`)
    /// with the given known entries and no rest type.
    #[must_use]
    pub fn keyed_sealed(&mut self, items: &[KnownItem<'arena>], non_empty: bool) -> Atom<'arena> {
        let known_items = Some(self.known_items(items));
        let mut flags = U8Flags::empty();
        flags.set_value(ArrayFlag::NonEmpty, non_empty);

        self.array(ArrayAtom { key_param: None, value_param: None, known_items, flags })
    }

    /// Intern a `callable(...)` with a "mixed" signature: parameters
    /// unspecified, return type `mixed`, no `throws`.
    #[must_use]
    pub fn callable_mixed(&mut self) -> Atom<'arena> {
        let signature = self.signature(Signature {
            parameters: None,
            return_type: well_known::TYPE_MIXED,
            throws: None,
            flags: U8Flags::empty(),
        });

        Atom::Callable(CallableAtom::Signature(signature))
    }

    /// Intern a `Closure(...)` with the same "mixed" signature as
    /// [`callable_mixed`](Self::callable_mixed) but tagged as a closure.
    #[must_use]
    pub fn closure_mixed(&mut self) -> Atom<'arena> {
        let signature = self.signature(Signature {
            parameters: None,
            return_type: well_known::TYPE_MIXED,
            throws: None,
            flags: U8Flags::empty(),
        });

        Atom::Callable(CallableAtom::Closure(signature))
    }
}
