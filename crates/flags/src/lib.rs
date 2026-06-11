//! Compact, type-tagged bitsets.
//!
//! This crate provides a family of bitset wrappers ([`U8Flags`], [`U16Flags`],
//! [`U32Flags`], [`U64Flags`], and [`U128Flags`]), each a single primitive
//! integer carrying the set bits plus a zero-sized `PhantomData<T>` tag.
//!
//! The tag is the point. `U8Flags<MethodModifier>` and `U8Flags<ClassModifier>`
//! are distinct, incompatible types even though both are one byte wide, so a
//! flag from one domain can never be set on, or compared against, another.
//!
//! A flag type is any `Copy` value that converts into the backing integer
//! (`T: Into<uN>`), typically a field-less enum whose variants are powers of two:
//!
//! ```
//! use mago_flags::U8Flags;
//!
//! #[derive(Clone, Copy)]
//! enum Modifier {
//!     Public  = 1 << 0,
//!     Static  = 1 << 1,
//!     Final   = 1 << 2,
//! }
//!
//! impl From<Modifier> for u8 {
//!     fn from(modifier: Modifier) -> u8 {
//!         modifier as u8
//!     }
//! }
//!
//! let mut flags = U8Flags::<Modifier>::empty();
//! flags.set(Modifier::Public);
//! flags.set(Modifier::Static);
//!
//! assert!(flags.contains(Modifier::Public));
//! assert!(!flags.contains(Modifier::Final));
//! assert_eq!(flags.count(), 2);
//! ```
//!
//! All five widths share the same API and the standard bitwise operators
//! (`|`, `&`, `^`, and their assigning forms). With the `serde` feature enabled,
//! each bitset serializes transparently as its backing integer.

#[cfg(feature = "serde")]
use serde::Deserialize;
#[cfg(feature = "serde")]
use serde::Serialize;

macro_rules! generate_flags_struct {
    ($($name:ident: $size:ty),+$(,)?) => {
        $(
            generate_flags_struct!($size, $name);
        )+
    };
    ($size:ty, $name:ident) => {
        #[doc = concat!("A type-tagged bitset backed by a `", stringify!($size), "`.")]
        ///
        /// The `T` parameter is a zero-sized marker identifying the flag domain;
        /// it carries no data and exists only to keep distinct flag sets from
        /// being mixed. See the [crate documentation](crate) for usage.
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
        pub struct $name<T: Copy>($size, std::marker::PhantomData<T>);

        impl<T: Copy> $name<T> {
            /// Creates an empty bitset with no flags set.
            #[inline]
            #[must_use]
            pub const fn new() -> Self {
                Self(0, std::marker::PhantomData)
            }

            /// Creates an empty bitset with no flags set.
            ///
            /// Alias for [`new`](Self::new).
            #[inline]
            #[must_use]
            pub const fn empty() -> Self {
                Self(0, std::marker::PhantomData)
            }

            /// Creates a bitset directly from a raw bit pattern.
            #[inline]
            #[must_use]
            pub const fn from_bits(bits: $size) -> Self {
                Self(bits, std::marker::PhantomData)
            }

            /// Returns the raw bit pattern backing this bitset.
            #[inline]
            #[must_use]
            pub const fn bits(&self) -> $size {
                self.0
            }

            /// Returns `true` if no flags are set.
            #[inline]
            #[must_use]
            pub const fn is_empty(&self) -> bool {
                self.0 == 0
            }

            /// Returns the number of flags currently set (the population count).
            #[inline]
            #[must_use]
            pub const fn count(&self) -> u32 {
                self.0.count_ones()
            }

            /// Returns `true` if every bit in `bits` is set.
            #[inline]
            #[must_use]
            pub const fn contains_bits(&self, bits: $size) -> bool {
                (self.0 & bits) == bits
            }

            /// Returns the union of `self` and `other` (the bits set in either).
            #[inline]
            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0, std::marker::PhantomData)
            }

            /// Returns the intersection of `self` and `other` (the bits set in both).
            #[inline]
            #[must_use]
            pub const fn intersection(self, other: Self) -> Self {
                Self(self.0 & other.0, std::marker::PhantomData)
            }

            /// Returns the difference of `self` and `other` (the bits set in `self` but not in `other`).
            #[inline]
            #[must_use]
            pub const fn difference(self, other: Self) -> Self {
                Self(self.0 & !other.0, std::marker::PhantomData)
            }

            /// Returns the symmetric difference of `self` and `other` (the bits set in exactly one of them).
            #[inline]
            #[must_use]
            pub const fn symmetric_difference(self, other: Self) -> Self {
                Self(self.0 ^ other.0, std::marker::PhantomData)
            }

            /// Clears all flags, leaving the bitset empty.
            #[inline]
            pub const fn clear(&mut self) {
                self.0 = 0;
            }
        }

        impl<T: Copy + Into<$size>> $name<T> {
            /// Sets every bit in `flag`.
            #[inline]
            pub fn set(&mut self, flag: T) {
                self.0 |= flag.into();
            }

            /// Unsets every bit in `flag`.
            #[inline]
            pub fn unset(&mut self, flag: T) {
                self.0 &= !flag.into();
            }

            /// Toggles every bit in `flag`.
            #[inline]
            pub fn toggle(&mut self, flag: T) {
                self.0 ^= flag.into();
            }

            /// Sets `flag` when `value` is `true`, unsets it otherwise.
            #[inline]
            pub fn set_value(&mut self, flag: T, value: bool) {
                if value {
                    self.set(flag);
                } else {
                    self.unset(flag);
                }
            }

            /// Returns `self` with every bit in `flag` additionally set.
            #[inline]
            #[must_use]
            pub fn with(mut self, flag: T) -> Self {
                self.set(flag);
                self
            }

            /// Returns `true` if every bit in `flag` is set.
            #[inline]
            #[must_use]
            pub fn contains(&self, flag: T) -> bool {
                let flag = flag.into();
                (self.0 & flag) == flag
            }

            /// Returns `true` if any bit in `flag` is set.
            #[inline]
            #[must_use]
            pub fn intersects(&self, flag: T) -> bool {
                (self.0 & flag.into()) != 0
            }
        }

        /// Extracts the raw bit pattern.
        impl<T: Copy> From<$name<T>> for $size {
            #[inline]
            fn from(flags: $name<T>) -> Self {
                flags.0
            }
        }

        /// Wraps a raw bit pattern as a bitset.
        impl<T: Copy> From<$size> for $name<T> {
            #[inline]
            fn from(bits: $size) -> Self {
                Self::from_bits(bits)
            }
        }

        /// Returns the union of the two bitsets.
        impl<T: Copy> std::ops::BitOr for $name<T> {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                self.union(rhs)
            }
        }

        /// Adds, in place, every bit present in `rhs`.
        impl<T: Copy> std::ops::BitOrAssign for $name<T> {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        /// Returns the intersection of the two bitsets.
        impl<T: Copy> std::ops::BitAnd for $name<T> {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                self.intersection(rhs)
            }
        }

        /// Retains, in place, only the bits also present in `rhs`.
        impl<T: Copy> std::ops::BitAndAssign for $name<T> {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        /// Returns the symmetric difference of the two bitsets.
        impl<T: Copy> std::ops::BitXor for $name<T> {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                self.symmetric_difference(rhs)
            }
        }

        /// Toggles, in place, every bit present in `rhs`.
        impl<T: Copy> std::ops::BitXorAssign for $name<T> {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }

        /// Returns an empty bitset.
        impl<T: Copy> std::default::Default for $name<T> {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }

        /// Serializes transparently as the backing integer.
        #[cfg(feature = "serde")]
        impl<T: Copy> Serialize for $name<T> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        /// Deserializes from the backing integer.
        #[cfg(feature = "serde")]
        impl<'de, T: Copy> Deserialize<'de> for $name<T> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let bits = <$size as Deserialize>::deserialize(deserializer)?;

                Ok(Self::from_bits(bits))
            }
        }
    }
}

generate_flags_struct! {
    U8Flags: u8,
    U16Flags: u16,
    U32Flags: u32,
    U64Flags: u64,
    U128Flags: u128,
}
