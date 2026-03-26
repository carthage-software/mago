use serde::Deserialize;
use serde::Serialize;

/// Represents the flags defined in a PHP `#[Attribute]` declaration,
/// specifying the targets where the attribute can be applied and whether it's repeatable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct AttributeFlags(u8);

impl AttributeFlags {
    /// Flag indicating the attribute can be applied to classes, interfaces, traits, and enums.
    /// Corresponds to `Attribute::TARGET_CLASS`.
    pub const TARGET_CLASS: AttributeFlags = AttributeFlags(1 << 0);

    /// Flag indicating the attribute can be applied to functions (including closures and arrow functions).
    /// Corresponds to `Attribute::TARGET_FUNCTION`.
    pub const TARGET_FUNCTION: AttributeFlags = AttributeFlags(1 << 1);

    /// Flag indicating the attribute can be applied to methods.
    /// Corresponds to `Attribute::TARGET_METHOD`.
    pub const TARGET_METHOD: AttributeFlags = AttributeFlags(1 << 2);

    /// Flag indicating the attribute can be applied to properties.
    /// Corresponds to `Attribute::TARGET_PROPERTY`.
    pub const TARGET_PROPERTY: AttributeFlags = AttributeFlags(1 << 3);

    /// Flag indicating the attribute can be applied to class constants.
    /// Corresponds to `Attribute::TARGET_CLASS_CONSTANT`.
    pub const TARGET_CLASS_CONSTANT: AttributeFlags = AttributeFlags(1 << 4);

    /// Flag indicating the attribute can be applied to function or method parameters.
    /// Corresponds to `Attribute::TARGET_PARAMETER`.
    pub const TARGET_PARAMETER: AttributeFlags = AttributeFlags(1 << 5);

    /// Flag indicating the attribute can be applied to global constants (defined with `const`).
    /// Corresponds to `Attribute::TARGET_CONSTANT`.
    pub const TARGET_CONSTANT: AttributeFlags = AttributeFlags(1 << 6);

    /// A combination of all `TARGET_*` flags, indicating the attribute can be applied anywhere.
    /// Corresponds to `Attribute::TARGET_ALL`.
    pub const TARGET_ALL: AttributeFlags =
        AttributeFlags((1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6));

    /// Flag indicating the attribute can be repeated on the same declaration.
    /// Corresponds to `Attribute::IS_REPEATABLE`.
    pub const IS_REPEATABLE: AttributeFlags = AttributeFlags(1 << 7);
}

impl AttributeFlags {
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        AttributeFlags(0)
    }

    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        AttributeFlags(bits)
    }

    #[inline]
    pub const fn insert(&mut self, other: AttributeFlags) {
        self.0 |= other.0;
    }

    #[inline]
    pub const fn set(&mut self, other: AttributeFlags, value: bool) {
        if value {
            self.insert(other);
        } else {
            self.0 &= !other.0;
        }
    }

    #[inline]
    pub const fn contains(self, other: AttributeFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn remove(&mut self, other: AttributeFlags) {
        self.0 &= !other.0;
    }

    #[inline]
    pub const fn intersects(self, other: AttributeFlags) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline]
    #[must_use]
    pub const fn union(&self, other: AttributeFlags) -> AttributeFlags {
        AttributeFlags(self.0 | other.0)
    }

    #[inline]
    #[must_use]
    pub const fn intersection(&self, other: AttributeFlags) -> AttributeFlags {
        AttributeFlags(self.0 & other.0)
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn all() -> Self {
        Self(Self::TARGET_ALL.0 | Self::IS_REPEATABLE.0)
    }
}

impl AttributeFlags {
    /// Checks if the `IS_REPEATABLE` flag is set, meaning the attribute
    /// can be declared multiple times on the same target.
    #[must_use]
    pub const fn is_repeatable(&self) -> bool {
        self.contains(Self::IS_REPEATABLE)
    }

    /// Checks if the `TARGET_CLASS` flag is set, indicating the attribute
    /// can be applied to classes, interfaces, traits, or enums.
    #[must_use]
    pub const fn targets_class(&self) -> bool {
        self.contains(Self::TARGET_CLASS)
    }

    /// Checks if the `TARGET_FUNCTION` flag is set, indicating the attribute
    /// can be applied to functions or closures.
    #[must_use]
    pub const fn targets_function(&self) -> bool {
        self.contains(Self::TARGET_FUNCTION)
    }

    /// Checks if the `TARGET_METHOD` flag is set, indicating the attribute
    /// can be applied to class or interface methods.
    #[must_use]
    pub const fn targets_method(&self) -> bool {
        self.contains(Self::TARGET_METHOD)
    }

    /// Checks if the `TARGET_PROPERTY` flag is set, indicating the attribute
    /// can be applied to class properties.
    #[must_use]
    pub const fn targets_property(&self) -> bool {
        self.contains(Self::TARGET_PROPERTY)
    }

    /// Checks if the `TARGET_CLASS_CONSTANT` flag is set, indicating the attribute
    /// can be applied to class constants.
    #[must_use]
    pub const fn targets_class_constant(&self) -> bool {
        self.contains(Self::TARGET_CLASS_CONSTANT)
    }

    /// Checks if the `TARGET_PARAMETER` flag is set, indicating the attribute
    /// can be applied to function or method parameters.
    #[must_use]
    pub const fn targets_parameter(&self) -> bool {
        self.contains(Self::TARGET_PARAMETER)
    }

    /// Checks if the `TARGET_CONSTANT` flag is set, indicating the attribute
    /// can be applied to global constants.
    #[must_use]
    pub const fn targets_constant(&self) -> bool {
        self.contains(Self::TARGET_CONSTANT)
    }

    /// Returns a list of human-readable strings for each target flag set.
    #[must_use]
    pub fn get_target_names(&self) -> Vec<&'static str> {
        let mut targets = Vec::with_capacity(7);

        if self.targets_class() {
            targets.push("classes");
        }

        if self.targets_function() {
            targets.push("functions");
        }

        if self.targets_method() {
            targets.push("methods");
        }

        if self.targets_property() {
            targets.push("properties");
        }

        if self.targets_class_constant() {
            targets.push("class constants");
        }

        if self.targets_parameter() {
            targets.push("parameters");
        }

        if self.targets_constant() {
            targets.push("global constants");
        }

        targets
    }
}

impl std::ops::BitOr for AttributeFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        AttributeFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for AttributeFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        AttributeFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for AttributeFlags {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        AttributeFlags(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for AttributeFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        AttributeFlags(!self.0)
    }
}

impl std::ops::BitOrAssign for AttributeFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAndAssign for AttributeFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitXorAssign for AttributeFlags {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
