use serde::Deserialize;
use serde::Serialize;

bitflags::bitflags! {
    /// Represents the flags defined in a PHP `#[Attribute]` declaration,
    /// specifying the targets where the attribute can be applied and whether it's repeatable.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AttributeFlags: u8 {
        /// Flag indicating the attribute can be applied to classes, interfaces, traits, and enums.
        /// Corresponds to `Attribute::TARGET_CLASS`.
        const TARGET_CLASS          = 1 << 0; //  1

        /// Flag indicating the attribute can be applied to functions (including closures and arrow functions).
        /// Corresponds to `Attribute::TARGET_FUNCTION`.
        const TARGET_FUNCTION       = 1 << 1; //  2

        /// Flag indicating the attribute can be applied to methods.
        /// Corresponds to `Attribute::TARGET_METHOD`.
        const TARGET_METHOD         = 1 << 2; //  4

        /// Flag indicating the attribute can be applied to properties.
        /// Corresponds to `Attribute::TARGET_PROPERTY`.
        const TARGET_PROPERTY       = 1 << 3; //  8

        /// Flag indicating the attribute can be applied to class constants.
        /// Corresponds to `Attribute::TARGET_CLASS_CONSTANT`.
        const TARGET_CLASS_CONSTANT = 1 << 4; // 16

        /// Flag indicating the attribute can be applied to function or method parameters.
        /// Corresponds to `Attribute::TARGET_PARAMETER`.
        const TARGET_PARAMETER      = 1 << 5; // 32

        /// Flag indicating the attribute can be applied to global constants (defined with `const`).
        /// Corresponds to `Attribute::TARGET_CONSTANT`.
        const TARGET_CONSTANT       = 1 << 6; // 64

        /// A combination of all `TARGET_*` flags, indicating the attribute can be applied anywhere.
        /// Corresponds to `Attribute::TARGET_ALL`.
        const TARGET_ALL = Self::TARGET_CLASS.bits()
            | Self::TARGET_FUNCTION.bits()
            | Self::TARGET_METHOD.bits()
            | Self::TARGET_PROPERTY.bits()
            | Self::TARGET_CLASS_CONSTANT.bits()
            | Self::TARGET_PARAMETER.bits()
            | Self::TARGET_CONSTANT.bits(); // 127

        /// Flag indicating the attribute can be repeated on the same declaration.
        /// Corresponds to `Attribute::IS_REPEATABLE`.
        const IS_REPEATABLE         = 1 << 7; // 128
    }
}

impl AttributeFlags {
    /// Checks if the `IS_REPEATABLE` flag is set, meaning the attribute
    /// can be declared multiple times on the same target.
    pub const fn is_repeatable(&self) -> bool {
        self.contains(Self::IS_REPEATABLE)
    }

    /// Checks if the `TARGET_CLASS` flag is set, indicating the attribute
    /// can be applied to classes, interfaces, traits, or enums.
    pub const fn targets_class(&self) -> bool {
        self.contains(Self::TARGET_CLASS)
    }

    /// Checks if the `TARGET_FUNCTION` flag is set, indicating the attribute
    /// can be applied to functions or closures.
    pub const fn targets_function(&self) -> bool {
        self.contains(Self::TARGET_FUNCTION)
    }

    /// Checks if the `TARGET_METHOD` flag is set, indicating the attribute
    /// can be applied to class or interface methods.
    pub const fn targets_method(&self) -> bool {
        self.contains(Self::TARGET_METHOD)
    }

    /// Checks if the `TARGET_PROPERTY` flag is set, indicating the attribute
    /// can be applied to class properties.
    pub const fn targets_property(&self) -> bool {
        self.contains(Self::TARGET_PROPERTY)
    }

    /// Checks if the `TARGET_CLASS_CONSTANT` flag is set, indicating the attribute
    /// can be applied to class constants.
    pub const fn targets_class_constant(&self) -> bool {
        self.contains(Self::TARGET_CLASS_CONSTANT)
    }

    /// Checks if the `TARGET_PARAMETER` flag is set, indicating the attribute
    /// can be applied to function or method parameters.
    pub const fn targets_parameter(&self) -> bool {
        self.contains(Self::TARGET_PARAMETER)
    }

    /// Checks if the `TARGET_CONSTANT` flag is set, indicating the attribute
    /// can be applied to global constants.
    pub const fn targets_constant(&self) -> bool {
        self.contains(Self::TARGET_CONSTANT)
    }

    /// Returns a list of human-readable strings for each target flag set.
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
