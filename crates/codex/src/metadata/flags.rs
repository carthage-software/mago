use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetadataFlags(u64);

impl MetadataFlags {
    pub const ABSTRACT: MetadataFlags = MetadataFlags(1 << 0);
    pub const FINAL: MetadataFlags = MetadataFlags(1 << 1);
    pub const READONLY: MetadataFlags = MetadataFlags(1 << 3);
    pub const DEPRECATED: MetadataFlags = MetadataFlags(1 << 4);
    pub const ENUM_INTERFACE: MetadataFlags = MetadataFlags(1 << 5);
    pub const POPULATED: MetadataFlags = MetadataFlags(1 << 6);
    pub const INTERNAL: MetadataFlags = MetadataFlags(1 << 7);
    pub const CONSISTENT_CONSTRUCTOR: MetadataFlags = MetadataFlags(1 << 11);
    pub const CONSISTENT_TEMPLATES: MetadataFlags = MetadataFlags(1 << 12);
    pub const UNCHECKED: MetadataFlags = MetadataFlags(1 << 13);
    pub const USER_DEFINED: MetadataFlags = MetadataFlags(1 << 14);
    pub const BUILTIN: MetadataFlags = MetadataFlags(1 << 15);
    pub const HAS_YIELD: MetadataFlags = MetadataFlags(1 << 16);
    pub const MUST_USE: MetadataFlags = MetadataFlags(1 << 17);
    pub const HAS_THROW: MetadataFlags = MetadataFlags(1 << 18);
    pub const PURE: MetadataFlags = MetadataFlags(1 << 19);
    pub const IGNORE_NULLABLE_RETURN: MetadataFlags = MetadataFlags(1 << 20);
    pub const IGNORE_FALSABLE_RETURN: MetadataFlags = MetadataFlags(1 << 21);
    pub const INHERITS_DOCS: MetadataFlags = MetadataFlags(1 << 22);
    pub const NO_NAMED_ARGUMENTS: MetadataFlags = MetadataFlags(1 << 23);
    pub const BACKED_ENUM_CASE: MetadataFlags = MetadataFlags(1 << 24);
    pub const UNIT_ENUM_CASE: MetadataFlags = MetadataFlags(1 << 25);
    pub const BY_REFERENCE: MetadataFlags = MetadataFlags(1 << 26);
    pub const VARIADIC: MetadataFlags = MetadataFlags(1 << 27);
    pub const PROMOTED_PROPERTY: MetadataFlags = MetadataFlags(1 << 28);
    pub const HAS_DEFAULT: MetadataFlags = MetadataFlags(1 << 29);
    pub const VIRTUAL_PROPERTY: MetadataFlags = MetadataFlags(1 << 30);
    pub const ASYMMETRIC_PROPERTY: MetadataFlags = MetadataFlags(1 << 31);
    pub const STATIC: MetadataFlags = MetadataFlags(1 << 32);
    pub const WRITEONLY: MetadataFlags = MetadataFlags(1 << 33);
    pub const MAGIC_PROPERTY: MetadataFlags = MetadataFlags(1 << 34);
    pub const MAGIC_METHOD: MetadataFlags = MetadataFlags(1 << 35);
    pub const API: MetadataFlags = MetadataFlags(1 << 36);
}

impl MetadataFlags {
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        MetadataFlags(0)
    }

    #[inline]
    pub const fn insert(&mut self, other: MetadataFlags) {
        self.0 |= other.0;
    }

    #[inline]
    pub const fn set(&mut self, other: MetadataFlags, value: bool) {
        if value {
            self.insert(other);
        } else {
            self.0 &= !other.0;
        }
    }

    #[inline]
    pub const fn contains(self, other: MetadataFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn remove(&mut self, other: MetadataFlags) {
        self.0 &= !other.0;
    }

    #[inline]
    pub const fn intersects(self, other: MetadataFlags) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline]
    #[must_use]
    pub const fn union(&self, other: MetadataFlags) -> MetadataFlags {
        MetadataFlags(self.0 | other.0)
    }

    #[inline]
    #[must_use]
    pub const fn intersection(&self, other: MetadataFlags) -> MetadataFlags {
        MetadataFlags(self.0 & other.0)
    }
}

impl MetadataFlags {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(self) -> bool {
        self.contains(Self::DEPRECATED)
    }

    #[inline]
    #[must_use]
    pub const fn is_abstract(self) -> bool {
        self.contains(Self::ABSTRACT)
    }

    #[inline]
    #[must_use]
    pub const fn is_final(self) -> bool {
        self.contains(Self::FINAL)
    }

    #[inline]
    #[must_use]
    pub const fn has_yield(self) -> bool {
        self.contains(Self::HAS_YIELD)
    }

    #[inline]
    #[must_use]
    pub const fn must_use(self) -> bool {
        self.contains(Self::MUST_USE)
    }

    #[inline]
    #[must_use]
    pub const fn is_pure(self) -> bool {
        self.contains(Self::PURE)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_constructor(self) -> bool {
        self.contains(Self::CONSISTENT_CONSTRUCTOR)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_templates(self) -> bool {
        self.contains(Self::CONSISTENT_TEMPLATES)
    }

    #[inline]
    #[must_use]
    pub const fn is_user_defined(self) -> bool {
        self.contains(Self::USER_DEFINED)
    }

    #[inline]
    #[must_use]
    pub const fn is_built_in(self) -> bool {
        self.contains(Self::BUILTIN)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(self) -> bool {
        self.contains(Self::INTERNAL)
    }

    #[inline]
    #[must_use]
    pub const fn is_populated(self) -> bool {
        self.contains(Self::POPULATED)
    }

    #[inline]
    #[must_use]
    pub const fn is_readonly(self) -> bool {
        self.contains(Self::READONLY)
    }

    #[inline]
    #[must_use]
    pub const fn is_writeonly(self) -> bool {
        self.contains(Self::WRITEONLY)
    }

    #[inline]
    #[must_use]
    pub const fn is_enum_interface(self) -> bool {
        self.contains(Self::ENUM_INTERFACE)
    }

    #[inline]
    #[must_use]
    pub const fn is_unchecked(self) -> bool {
        self.contains(Self::UNCHECKED)
    }

    #[inline]
    #[must_use]
    pub const fn ignore_nullable_return(self) -> bool {
        self.contains(Self::IGNORE_NULLABLE_RETURN)
    }

    #[inline]
    #[must_use]
    pub const fn ignore_falsable_return(self) -> bool {
        self.contains(Self::IGNORE_FALSABLE_RETURN)
    }

    #[inline]
    #[must_use]
    pub const fn inherits_docs(self) -> bool {
        self.contains(Self::INHERITS_DOCS)
    }

    #[inline]
    #[must_use]
    pub const fn forbids_named_arguments(self) -> bool {
        self.contains(Self::NO_NAMED_ARGUMENTS)
    }

    #[inline]
    #[must_use]
    pub const fn has_throw(self) -> bool {
        self.contains(Self::HAS_THROW)
    }

    #[inline]
    #[must_use]
    pub const fn is_backed_enum_case(self) -> bool {
        self.contains(Self::BACKED_ENUM_CASE)
    }

    #[inline]
    #[must_use]
    pub const fn is_unit_enum_case(self) -> bool {
        self.contains(Self::UNIT_ENUM_CASE)
    }

    #[inline]
    #[must_use]
    pub const fn is_by_reference(self) -> bool {
        self.contains(Self::BY_REFERENCE)
    }

    #[inline]
    #[must_use]
    pub const fn is_variadic(self) -> bool {
        self.contains(Self::VARIADIC)
    }

    #[inline]
    #[must_use]
    pub const fn is_promoted_property(self) -> bool {
        self.contains(Self::PROMOTED_PROPERTY)
    }

    #[inline]
    #[must_use]
    pub const fn has_default(self) -> bool {
        self.contains(Self::HAS_DEFAULT)
    }

    #[inline]
    #[must_use]
    pub const fn is_virtual_property(self) -> bool {
        self.contains(Self::VIRTUAL_PROPERTY)
    }

    #[inline]
    #[must_use]
    pub const fn is_magic_property(self) -> bool {
        self.contains(Self::MAGIC_PROPERTY)
    }

    #[inline]
    #[must_use]
    pub const fn is_magic_method(self) -> bool {
        self.contains(Self::MAGIC_METHOD)
    }

    #[inline]
    #[must_use]
    pub const fn is_asymmetric_property(self) -> bool {
        self.contains(Self::ASYMMETRIC_PROPERTY)
    }

    #[inline]
    #[must_use]
    pub const fn is_static(self) -> bool {
        self.contains(Self::STATIC)
    }

    #[inline]
    #[must_use]
    pub const fn is_public_api(self) -> bool {
        self.contains(Self::API)
    }
}

impl std::ops::BitOr for MetadataFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        MetadataFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for MetadataFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        MetadataFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for MetadataFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        MetadataFlags(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for MetadataFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        MetadataFlags(!self.0)
    }
}

impl std::ops::BitOrAssign for MetadataFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAndAssign for MetadataFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitXorAssign for MetadataFlags {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
