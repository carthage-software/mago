#![allow(clippy::too_many_arguments)]

//! A high-performance, globally-interned string library for the Mago ecosystem.
//!
//! This crate provides `Atom`, a canonical string type that guarantees any given
//! string is stored in memory only once. It acts as a wrapper for the `ustr` crate and adds
//! highly-optimized constructors for common string manipulations like lowercasing,
//! concatenation, and number formatting.
//!
//! The key feature is the ability to perform these operations without heap allocations
//! for common cases by using stack-allocated buffers, making this crate ideal for
//! performance-critical code.
//!
//! # Usage
//!
//! ```
//! use mago_atom::*;
//!
//! // Create an Atom. This is a cheap lookup in a global cache.
//! let s1 = atom("Hello");
//!
//! // Use an optimized, zero-heap-allocation constructor.
//! let s2 = ascii_lowercase_atom("Hello");
//!
//! assert_eq!(s2.as_str(), "hello");
//!
//! // Use the specialized, high-performance map.
//! let mut map = AtomMap::default();
//! map.insert(s1, 123);
//! ```

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

use ustr::IdentityHasher;

pub use ustr::Ustr as Atom;
pub use ustr::ustr as atom;

/// A high-performance `HashMap` using `Atom` as the key.
///
/// This map is significantly faster than a standard `HashMap` because it uses the
/// `Atom`'s pre-computed hash instead of hashing the string content on every lookup.
pub type AtomMap<V> = HashMap<Atom, V, BuildHasherDefault<IdentityHasher>>;

/// A high-performance `HashSet` using `Atom` as the key.
///
/// This set is significantly faster than a standard `HashSet` because it uses the
/// `Atom`'s pre-computed hash.
pub type AtomSet = HashSet<Atom, BuildHasherDefault<IdentityHasher>>;

/// The maximum size in bytes for a string to be processed on the stack.
const STACK_BUF_SIZE: usize = 256;

/// Returns the canonical `Atom` for an empty string.
///
/// This is a very cheap operation.
#[inline]
#[must_use]
pub fn empty_atom() -> Atom {
    atom("")
}

/// A macro to concatenate between 2 and 12 string slices into a single `Atom`.
///
/// This macro dispatches to a specialized, zero-heap-allocation function based on the
/// number of arguments provided, making it highly performant for a known number of inputs.
/// It uses a stack-allocated buffer to avoid hitting the heap.
///
/// # Panics
///
/// Panics at compile time if called with 0, 1, or more than 12 arguments.
#[macro_export]
macro_rules! concat_atom {
    ($s1:expr, $s2:expr $(,)?) => {
        $crate::concat_atom2(&$s1, &$s2)
    };
    ($s1:expr, $s2:expr, $s3:expr $(,)?) => {
        $crate::concat_atom3(&$s1, &$s2, &$s3)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr $(,)?) => {
        $crate::concat_atom4(&$s1, &$s2, &$s3, &$s4)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr $(,)?) => {
        $crate::concat_atom5(&$s1, &$s2, &$s3, &$s4, &$s5)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr $(,)?) => {
        $crate::concat_atom6(&$s1, &$s2, &$s3, &$s4, &$s5, &$s6)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr $(,)?) => {
        $crate::concat_atom7(&$s1, &$s2, &$s3, &$s4, &$s5, &$s6, &$s7)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr $(,)?) => {
        $crate::concat_atom8(&$s1, &$s2, &$s3, &$s4, &$s5, &$s6, &$s7, &$s8)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr $(,)?) => {
        $crate::concat_atom9(&$s1, &$s2, &$s3, &$s4, &$s5, &$s6, &$s7, &$s8, &$s9)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr, $s10:expr $(,)?) => {
        $crate::concat_atom10(&$s1, &$s2, &$s3, &$s4, &$s5, &$s6, &$s7, &$s8, &$s9, &$s10)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr, $s10:expr, $s11:expr $(,)?) => {
        $crate::concat_atom11(&$s1, &$s2, &$s3, &$s4, &$s5, &$s6, &$s7, &$s8, &$s9, &$s10, &$s11)
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr, $s10:expr, $s11:expr, $s12:expr $(,)?) => {
        $crate::concat_atom12(&$s1, &$s2, &$s3, &$s4, &$s5, &$s6, &$s7, &$s8, &$s9, &$s10, &$s11, &$s12)
    };
    ($($arg:expr),+ $(,)?) => {
        compile_error!("concat_atom! macro supports between 2 and 12 arguments only")
    };
}

/// Creates an `Atom` from a constant name, lowercasing only the namespace part.
///
/// This function is optimized to avoid heap allocations for constant names up to
/// `STACK_BUF_SIZE` bytes by building the new string on the stack. For names
/// longer than the buffer, it falls back to a heap allocation.
#[inline]
#[must_use]
pub fn ascii_lowercase_constant_name_atom(name: &str) -> Atom {
    if let Some(last_slash_idx) = name.rfind('\\') {
        let (namespace, const_name) = name.split_at(last_slash_idx);
        let const_name = &const_name[1..];

        if name.len() > STACK_BUF_SIZE {
            let mut lowercased_namespace = namespace.to_ascii_lowercase();
            lowercased_namespace.push('\\');
            lowercased_namespace.push_str(const_name);
            return atom(&lowercased_namespace);
        }

        let mut stack_buf = [0u8; STACK_BUF_SIZE];
        let mut index = 0;

        for byte in namespace.bytes() {
            stack_buf[index] = byte.to_ascii_lowercase();
            index += 1;
        }

        stack_buf[index] = b'\\';
        index += 1;

        let const_bytes = const_name.as_bytes();
        stack_buf[index..index + const_bytes.len()].copy_from_slice(const_bytes);
        index += const_bytes.len();

        atom(
            // SAFETY: We only write valid UTF-8 bytes into the stack buffer.
            unsafe { std::str::from_utf8_unchecked(&stack_buf[..index]) },
        )
    } else {
        atom(name)
    }
}

/// Creates an `Atom` from a lowercased version of a string slice.
///
/// This function is highly optimized. It performs a fast scan, and if the string
/// is already lowercase, it returns an `Atom` without any new allocations.
/// Otherwise, it builds the lowercase version on the stack for strings up to
/// `STACK_BUF_SIZE` bytes.
#[inline]
#[must_use]
pub fn ascii_lowercase_atom(s: &str) -> Atom {
    if s.is_ascii() && !s.bytes().any(|b| b.is_ascii_uppercase()) {
        return atom(s);
    }

    if s.len() <= STACK_BUF_SIZE {
        let mut stack_buf = [0u8; STACK_BUF_SIZE];
        let mut index = 0;

        for c in s.chars() {
            for lower_c in c.to_lowercase() {
                let mut char_buf = [0u8; 4];
                let bytes = lower_c.encode_utf8(&mut char_buf).as_bytes();

                if index + bytes.len() > STACK_BUF_SIZE {
                    return atom(&s.to_lowercase());
                }

                stack_buf[index..index + bytes.len()].copy_from_slice(bytes);
                index += bytes.len();
            }
        }

        return atom(
            // SAFETY: We only write valid UTF-8 bytes into the stack buffer.
            unsafe { std::str::from_utf8_unchecked(&stack_buf[..index]) },
        );
    }

    atom(&s.to_lowercase())
}

/// A helper macro to generate the specialized `*_atom` functions for integer types.
macro_rules! integer_to_atom_fns {
    ( $( $func_name:ident($num_type:ty) ),+ $(,)? ) => {
        $(
            #[doc = "Creates an `Atom` from a `"]
            #[doc = stringify!($num_type)]
            #[doc = "` value with zero heap allocations."]
            #[inline]
            #[must_use]
            pub fn $func_name(n: $num_type) -> Atom {
                let mut buffer = itoa::Buffer::new();
                let s = buffer.format(n);

                atom(s)
            }
        )+
    };
}

/// A helper macro to generate the specialized `*_atom` functions for float types.
macro_rules! float_to_atom_fns {
    ( $( $func_name:ident($num_type:ty) ),+ $(,)? ) => {
        $(
            #[doc = "Creates an `Atom` from a `"]
            #[doc = stringify!($num_type)]
            #[doc = "` value with zero heap allocations."]
            #[inline]
            #[must_use]
            pub fn $func_name(n: $num_type) -> Atom {
                let mut buffer = ryu::Buffer::new();
                let s = buffer.format(n);

                atom(s)
            }
        )+
    };
}

/// A helper macro to generate the specialized `concat_atomN` functions.
macro_rules! concat_fns {
    ( $( $func_name:ident($n:literal, $($s:ident),+) ),+ $(,)?) => {
        $(
            #[doc = "Creates an `Atom` as a result of concatenating "]
            #[doc = stringify!($n)]
            #[doc = " string slices."]
            #[inline]
            #[must_use]
            #[allow(unused_assignments)]
            pub fn $func_name($($s: &str),+) -> Atom {
                let total_len = 0 $(+ $s.len())+;

                if total_len <= STACK_BUF_SIZE {
                    let mut buffer = [0u8; STACK_BUF_SIZE];
                    let mut index = 0;
                    $(
                        buffer[index..index + $s.len()].copy_from_slice($s.as_bytes());
                        index += $s.len();
                    )+
                    return atom(unsafe { std::str::from_utf8_unchecked(&buffer[..total_len]) });
                }

                // Fallback to heap for very long strings.
                let mut result = String::with_capacity(total_len);
                $( result.push_str($s); )+
                atom(&result)
            }
        )+
    };
}

// Generate functions for integer types
integer_to_atom_fns!(
    i8_atom(i8),
    i16_atom(i16),
    i32_atom(i32),
    i64_atom(i64),
    i128_atom(i128),
    isize_atom(isize),
    u8_atom(u8),
    u16_atom(u16),
    u32_atom(u32),
    u64_atom(u64),
    u128_atom(u128),
    usize_atom(usize),
);

float_to_atom_fns!(f32_atom(f32), f64_atom(f64),);

concat_fns!(
    concat_atom2(2, s1, s2),
    concat_atom3(3, s1, s2, s3),
    concat_atom4(4, s1, s2, s3, s4),
    concat_atom5(5, s1, s2, s3, s4, s5),
    concat_atom6(6, s1, s2, s3, s4, s5, s6),
    concat_atom7(7, s1, s2, s3, s4, s5, s6, s7),
    concat_atom8(8, s1, s2, s3, s4, s5, s6, s7, s8),
    concat_atom9(9, s1, s2, s3, s4, s5, s6, s7, s8, s9),
    concat_atom10(10, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10),
    concat_atom11(11, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11),
    concat_atom12(12, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12),
);
