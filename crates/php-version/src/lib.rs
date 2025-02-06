use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::error::ParsingError;
use crate::feature::Feature;

pub mod error;
pub mod feature;

/// Represents a PHP version in `(major, minor, patch)` format,
/// packed internally into a single `u32` for easy comparison.
///
/// # Examples
///
/// ```
/// use mago_php_version::PHPVersion;
///
/// let version = PHPVersion::new(8, 4, 0);
/// assert_eq!(version.major(), 8);
/// assert_eq!(version.minor(), 4);
/// assert_eq!(version.patch(), 0);
/// assert_eq!(version.to_version_id(), 0x08_04_00);
/// assert_eq!(version.to_string(), "8.4.0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PHPVersion(u32);

impl PHPVersion {
    /// The PHP 7.0 version.
    pub const PHP70: PHPVersion = PHPVersion::new(7, 0, 0);

    /// The PHP 7.1 version.
    pub const PHP71: PHPVersion = PHPVersion::new(7, 1, 0);

    /// The PHP 7.2 version.
    pub const PHP72: PHPVersion = PHPVersion::new(7, 2, 0);

    /// The PHP 7.3 version.
    pub const PHP73: PHPVersion = PHPVersion::new(7, 3, 0);

    /// The PHP 7.4 version.
    pub const PHP74: PHPVersion = PHPVersion::new(7, 4, 0);

    /// The PHP 8.0 version.
    pub const PHP80: PHPVersion = PHPVersion::new(8, 0, 0);

    /// The PHP 8.1 version.
    pub const PHP81: PHPVersion = PHPVersion::new(8, 1, 0);

    /// The PHP 8.2 version.
    pub const PHP82: PHPVersion = PHPVersion::new(8, 2, 0);

    /// The PHP 8.3 version.
    pub const PHP83: PHPVersion = PHPVersion::new(8, 3, 0);

    /// The PHP 8.4 version.
    pub const PHP84: PHPVersion = PHPVersion::new(8, 4, 0);

    /// Creates a new `PHPVersion` from the provided `major`, `minor`, and `patch` values.
    ///
    /// The internal representation packs these three components into a single `u32`
    /// for efficient comparisons.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    ///
    /// let version = PHPVersion::new(8, 1, 3);
    /// assert_eq!(version.major(), 8);
    /// assert_eq!(version.minor(), 1);
    /// assert_eq!(version.patch(), 3);
    /// ```
    #[inline]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self((major << 16) | (minor << 8) | patch)
    }

    /// Creates a `PHPVersion` directly from a raw version ID (e.g. `80400` for `8.4.0`).
    ///
    /// This can be useful if you already have the numeric form. The higher bits represent
    /// the major version, the next bits represent minor, and the lowest bits represent patch.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    ///
    /// // "8.4.0" => 0x080400 in hex, which is 525312 in decimal
    /// let version = PHPVersion::from_version_id(0x080400);
    /// assert_eq!(version.to_string(), "8.4.0");
    /// ```
    #[inline]
    pub const fn from_version_id(version_id: u32) -> Self {
        Self(version_id)
    }

    /// Returns the **major** component of the PHP version.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    ///
    /// let version = PHPVersion::new(8, 2, 0);
    /// assert_eq!(version.major(), 8);
    /// ```
    #[inline]
    pub const fn major(&self) -> u32 {
        self.0 >> 16
    }

    /// Returns the **minor** component of the PHP version.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    ///
    /// let version = PHPVersion::new(8, 2, 0);
    /// assert_eq!(version.minor(), 2);
    /// ```
    #[inline]
    pub const fn minor(&self) -> u32 {
        (self.0 >> 8) & 0xff
    }

    /// Returns the **patch** component of the PHP version.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    ///
    /// let version = PHPVersion::new(8, 1, 13);
    /// assert_eq!(version.patch(), 13);
    /// ```
    #[inline]
    pub const fn patch(&self) -> u32 {
        self.0 & 0xff
    }

    /// Determines if this version is **at least** `major.minor.patch`.
    ///
    /// Returns `true` if `self >= (major.minor.patch)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    ///
    /// let version = PHPVersion::new(8, 0, 0);
    /// assert!(version.is_at_least(8, 0, 0));
    /// assert!(version.is_at_least(7, 4, 30)); // 8.0.0 is newer than 7.4.30
    /// assert!(!version.is_at_least(8, 1, 0));
    /// ```
    pub const fn is_at_least(&self, major: u32, minor: u32, patch: u32) -> bool {
        self.0 >= ((major << 16) | (minor << 8) | patch)
    }

    /// Checks if a given [`Feature`] is supported by this PHP version.
    ///
    /// The logic is based on version thresholds (e.g. `>= 8.0.0` or `< 8.0.0`).
    /// Each `Feature` variant corresponds to a behavior introduced, removed, or changed
    /// at a particular version boundary.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    /// use mago_php_version::feature::Feature;
    ///
    /// let version = PHPVersion::new(7, 4, 0);
    /// assert!(version.is_supported(Feature::NullCoalesceAssign));
    /// assert!(!version.is_supported(Feature::NamedArguments));
    /// ```
    pub const fn is_supported(&self, feature: Feature) -> bool {
        match feature {
            Feature::NullableTypeHint
            | Feature::IterableTypeHint
            | Feature::VoidTypeHint
            | Feature::ClassLikeConstantVisibilityModifiers => self.0 >= 0x07_01_00,
            Feature::TrailingCommaInListSyntax
            | Feature::ParameterTypeWidening
            | Feature::AllUnicodeScalarCodePointsInMbSubstituteCharacter => self.0 >= 0x07_02_00,
            Feature::ListReferenceAssignment | Feature::TrailingCommaInFunctionCalls => self.0 >= 0x07_03_00,
            Feature::NullCoalesceAssign
            | Feature::ParameterContravariance
            | Feature::ReturnCovariance
            | Feature::PregUnmatchedAsNull
            | Feature::ArrowFunctions
            | Feature::NumericLiteralSeparator
            | Feature::TypedProperties => self.0 >= 0x070400,
            Feature::NonCapturingCatches
            | Feature::NativeUnionTypes
            | Feature::LessOverridenParametersWithVariadic
            | Feature::ThrowExpression
            | Feature::ClassConstantOnExpression
            | Feature::PromotedProperties
            | Feature::NamedArguments
            | Feature::ThrowsTypeErrorForInternalFunctions
            | Feature::ThrowsValueErrorForInternalFunctions
            | Feature::HHPrintfSpecifier
            | Feature::StricterRoundFunctions
            | Feature::ThrowsOnInvalidMbStringEncoding
            | Feature::WarnsAboutFinalPrivateMethods
            | Feature::CastsNumbersToStringsOnLooseComparison
            | Feature::NonNumericStringAndIntegerIsFalseOnLooseComparison
            | Feature::AbstractTraitMethods
            | Feature::StaticReturnTypeHint
            | Feature::AccessClassOnObject
            | Feature::Attribute
            | Feature::MixedTypeHint
            | Feature::MatchExpression
            | Feature::NullSafeOperator
            | Feature::TrailingCommaInClosureUseList
            | Feature::FalseCompoundTypeHint
            | Feature::NullCompoundTypeHint => self.0 >= 0x08_00_00,
            Feature::FinalConstants
            | Feature::ReadonlyProperties
            | Feature::Enums
            | Feature::PureIntersectionTypes
            | Feature::TentativeReturnTypes
            | Feature::NeverTypeHint
            | Feature::ClosureCreation
            | Feature::ArrayUnpackingWithStringKeys
            | Feature::SerializableRequiresMagicMethods => self.0 >= 0x08_01_00,
            Feature::ConstantsInTraits
            | Feature::StrSplitReturnsEmptyArray
            | Feature::DisjunctiveNormalForm
            | Feature::ReadonlyClasses
            | Feature::NeverReturnTypeInArrowFunction
            | Feature::PregCaptureOnlyNamedGroups
            | Feature::TrueTypeHint
            | Feature::FalseTypeHint
            | Feature::NullTypeHint => self.0 >= 0x08_02_00,
            Feature::JsonValidate
            | Feature::TypedClassLikeConstants
            | Feature::DateTimeExceptions
            | Feature::OverrideAttribute
            | Feature::DynamicClassConstantAccess
            | Feature::ReadonlyAnonymousClasses => self.0 >= 0x08_03_00,
            Feature::AsymmetricVisibility
            | Feature::LazyObjects
            | Feature::HighlightStringDoesNotReturnFalse
            | Feature::PropertyHooks
            | Feature::NewWithoutParentheses => self.0 >= 0x08_04_00,
            Feature::ClosureInConstantExpressions | Feature::ConstantAttribute => self.0 >= 0x08_05_00,
            Feature::CallableInstanceMethods
            | Feature::LegacyConstructor
            | Feature::UnsetCast
            | Feature::CaseInsensitiveConstantNames
            | Feature::ArrayFunctionsReturnNullWithNonArray
            | Feature::SubstrReturnFalseInsteadOfEmptyString
            | Feature::CurlUrlOptionCheckingFileSchemeWithOpenBasedir
            | Feature::EmptyStringValidAliasForNoneInMbSubstituteCharacter
            | Feature::NumericStringValidArgInMbSubstituteCharacter => self.0 < 0x08_00_00,
            Feature::InterfaceConstantImplicitlyFinal => self.0 < 0x08_01_00,
            Feature::PassNoneEncodings => self.0 < 0x07_03_00,
            _ => true,
        }
    }

    /// Checks if a given [`Feature`] is deprecated in this PHP version.
    ///
    /// Returns `true` if the feature is *considered deprecated* at or above
    /// certain version thresholds. The threshold logic is encoded within the `match`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    /// use mago_php_version::feature::Feature;
    ///
    /// let version = PHPVersion::new(8, 0, 0);
    /// assert!(version.is_deprecated(Feature::RequiredParameterAfterOptional));
    /// assert!(!version.is_deprecated(Feature::DynamicProperties)); // that is 8.2+
    /// ```
    pub const fn is_deprecated(&self, feature: Feature) -> bool {
        match feature {
            Feature::DynamicProperties | Feature::CallStaticMethodOnTrait => self.0 >= 0x08_02_00,
            Feature::ImplicitlyNullableParameterTypes => self.0 >= 0x08_04_00,
            Feature::RequiredParameterAfterOptionalUnionOrMixed => self.0 >= 0x08_03_00,
            Feature::RequiredParameterAfterOptionalNullableAndDefaultNull => self.0 >= 0x08_01_00,
            Feature::RequiredParameterAfterOptional => self.0 >= 0x08_00_00,
            _ => false,
        }
    }

    /// Converts this `PHPVersion` into a raw version ID (e.g. `80400` for `8.4.0`).
    ///
    /// This is the inverse of [`from_version_id`].
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_php_version::PHPVersion;
    ///
    /// let version = PHPVersion::new(8, 4, 0);
    /// assert_eq!(version.to_version_id(), 0x080400);
    /// ```
    pub const fn to_version_id(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for PHPVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}

impl Serialize for PHPVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PHPVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        s.parse().map_err(serde::de::Error::custom)
    }
}

impl FromStr for PHPVersion {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParsingError::InvalidFormat);
        }

        let parts = s.split('.').collect::<Vec<_>>();
        match parts.len() {
            1 => {
                let major = parts[0].parse()?;

                Ok(Self::new(major, 0, 0))
            }
            2 => {
                let major = parts[0].parse()?;
                let minor = parts[1].parse()?;

                Ok(Self::new(major, minor, 0))
            }
            3 => {
                let major = parts[0].parse()?;
                let minor = parts[1].parse()?;
                let patch = parts[2].parse()?;

                Ok(Self::new(major, minor, patch))
            }
            _ => Err(ParsingError::InvalidFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = PHPVersion::new(7, 4, 0);
        assert_eq!(version.major(), 7);
        assert_eq!(version.minor(), 4);
        assert_eq!(version.patch(), 0);
    }

    #[test]
    fn test_display() {
        let version = PHPVersion::new(7, 4, 0);
        assert_eq!(version.to_string(), "7.4.0");
    }

    #[test]
    fn test_from_str_single_segment() {
        let v: PHPVersion = "7".parse().unwrap();
        assert_eq!(v.major(), 7);
        assert_eq!(v.minor(), 0);
        assert_eq!(v.patch(), 0);
        assert_eq!(v.to_string(), "7.0.0");
    }

    #[test]
    fn test_from_str_two_segments() {
        let v: PHPVersion = "7.4".parse().unwrap();
        assert_eq!(v.major(), 7);
        assert_eq!(v.minor(), 4);
        assert_eq!(v.patch(), 0);
        assert_eq!(v.to_string(), "7.4.0");
    }

    #[test]
    fn test_from_str_three_segments() {
        let v: PHPVersion = "8.1.2".parse().unwrap();
        assert_eq!(v.major(), 8);
        assert_eq!(v.minor(), 1);
        assert_eq!(v.patch(), 2);
        assert_eq!(v.to_string(), "8.1.2");
    }

    #[test]
    fn test_from_str_invalid() {
        let err = "7.4.0.1".parse::<PHPVersion>().unwrap_err();
        assert_eq!(format!("{}", err), "Invalid version format, expected 'major.minor.patch'.");

        let err = "".parse::<PHPVersion>().unwrap_err();
        assert_eq!(format!("{}", err), "Invalid version format, expected 'major.minor.patch'.");

        let err = "foo.4.0".parse::<PHPVersion>().unwrap_err();
        assert_eq!(format!("{}", err), "Failed to parse integer component of version: invalid digit found in string.");

        let err = "7.foo.0".parse::<PHPVersion>().unwrap_err();
        assert_eq!(format!("{}", err), "Failed to parse integer component of version: invalid digit found in string.");

        let err = "7.4.foo".parse::<PHPVersion>().unwrap_err();
        assert_eq!(format!("{}", err), "Failed to parse integer component of version: invalid digit found in string.");
    }

    #[test]
    fn test_is_supported_features_before_8() {
        let v_7_4_0 = PHPVersion::new(7, 4, 0);

        assert!(v_7_4_0.is_supported(Feature::NullCoalesceAssign));
        assert!(!v_7_4_0.is_supported(Feature::NamedArguments));

        assert!(v_7_4_0.is_supported(Feature::CallableInstanceMethods));
        assert!(v_7_4_0.is_supported(Feature::LegacyConstructor));
    }

    #[test]
    fn test_is_supported_features_8_0_0() {
        let v_8_0_0 = PHPVersion::new(8, 0, 0);

        assert!(v_8_0_0.is_supported(Feature::NamedArguments));
        assert!(!v_8_0_0.is_supported(Feature::CallableInstanceMethods));
    }

    #[test]
    fn test_is_deprecated_features() {
        let v_7_4_0 = PHPVersion::new(7, 4, 0);
        assert!(!v_7_4_0.is_deprecated(Feature::DynamicProperties));
        assert!(!v_7_4_0.is_deprecated(Feature::RequiredParameterAfterOptional));

        let v_8_0_0 = PHPVersion::new(8, 0, 0);
        assert!(v_8_0_0.is_deprecated(Feature::RequiredParameterAfterOptional));
        assert!(!v_8_0_0.is_deprecated(Feature::DynamicProperties));

        let v_8_2_0 = PHPVersion::new(8, 2, 0);
        assert!(v_8_2_0.is_deprecated(Feature::DynamicProperties));
    }

    #[test]
    fn test_serde_serialize() {
        let v_7_4_0 = PHPVersion::new(7, 4, 0);
        let json = serde_json::to_string(&v_7_4_0).unwrap();
        assert_eq!(json, "\"7.4.0\"");
    }

    #[test]
    fn test_serde_deserialize() {
        let json = "\"7.4.0\"";
        let v: PHPVersion = serde_json::from_str(json).unwrap();
        assert_eq!(v.major(), 7);
        assert_eq!(v.minor(), 4);
        assert_eq!(v.patch(), 0);

        let json = "\"7.4\"";
        let v: PHPVersion = serde_json::from_str(json).unwrap();
        assert_eq!(v.major(), 7);
        assert_eq!(v.minor(), 4);
        assert_eq!(v.patch(), 0);
    }

    #[test]
    fn test_serde_round_trip() {
        let original = PHPVersion::new(8, 1, 5);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: PHPVersion = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
        assert_eq!(serialized, "\"8.1.5\"");
    }
}
