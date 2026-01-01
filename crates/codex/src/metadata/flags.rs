use serde::Deserialize;
use serde::Serialize;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct MetadataFlags: u64 {
        const ABSTRACT                  = 1 << 0;
        const FINAL                     = 1 << 1;
        const READONLY                  = 1 << 3;
        const DEPRECATED                = 1 << 4;
        const ENUM_INTERFACE            = 1 << 5;
        const POPULATED                 = 1 << 6;
        const INTERNAL                  = 1 << 7;
        const CONSISTENT_CONSTRUCTOR    = 1 << 11;
        const CONSISTENT_TEMPLATES      = 1 << 12;
        const UNCHECKED                 = 1 << 13;
        const USER_DEFINED              = 1 << 14;
        const BUILTIN                   = 1 << 15;
        const HAS_YIELD                 = 1 << 16;
        const MUST_USE                  = 1 << 17;
        const HAS_THROW                 = 1 << 18;
        const PURE                      = 1 << 19;
        const IGNORE_NULLABLE_RETURN    = 1 << 20;
        const IGNORE_FALSABLE_RETURN    = 1 << 21;
        const INHERITS_DOCS             = 1 << 22;
        const NO_NAMED_ARGUMENTS        = 1 << 23;
        const BACKED_ENUM_CASE          = 1 << 24;
        const UNIT_ENUM_CASE            = 1 << 25;
        const BY_REFERENCE              = 1 << 26;
        const VARIADIC                  = 1 << 27;
        const PROMOTED_PROPERTY         = 1 << 28;
        const HAS_DEFAULT               = 1 << 29;
        const VIRTUAL_PROPERTY          = 1 << 30;
        const ASYMMETRIC_PROPERTY       = 1 << 31;
        const STATIC                    = 1 << 32;
        const WRITEONLY                 = 1 << 33;
        const MAGIC_PROPERTY            = 1 << 34;
        const MAGIC_METHOD              = 1 << 35;
    }
}

// example: helper methods for extra readability
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
}
