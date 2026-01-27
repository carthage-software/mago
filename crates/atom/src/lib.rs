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

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vandq_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vceqq_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vcgeq_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vcleq_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vdupq_n_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vld1q_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vminvq_u8;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::vorrq_u8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
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

thread_local! {
    static EMPTY_ATOM: Atom = atom("");
}

/// Returns the canonical `Atom` for an empty string.
///
/// This is a very cheap operation.
#[inline]
#[must_use]
pub fn empty_atom() -> Atom {
    EMPTY_ATOM.with(|&atom| atom)
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
    let bytes = s.as_bytes();

    // Fast path: single pass to check if already lowercase ASCII
    // This combines the is_ascii() and any(is_ascii_uppercase) checks into one iteration
    let mut needs_lowercasing = false;
    let mut is_ascii = true;
    for &b in bytes {
        if b > 127 {
            is_ascii = false;
            break;
        }
        if b.is_ascii_uppercase() {
            needs_lowercasing = true;
        }
    }

    // If it's ASCII and already lowercase, return as-is
    if is_ascii && !needs_lowercasing {
        return atom(s);
    }

    // Fast path for ASCII-only strings: use simple byte manipulation
    if is_ascii && s.len() <= STACK_BUF_SIZE {
        let mut stack_buf = [0u8; STACK_BUF_SIZE];
        for (i, &b) in bytes.iter().enumerate() {
            stack_buf[i] = b.to_ascii_lowercase();
        }
        return atom(
            // SAFETY: ASCII lowercase of ASCII bytes is valid UTF-8
            unsafe { std::str::from_utf8_unchecked(&stack_buf[..s.len()]) },
        );
    }

    // Non-ASCII path: handle Unicode lowercasing
    if s.len() <= STACK_BUF_SIZE {
        let mut stack_buf = [0u8; STACK_BUF_SIZE];
        let mut index = 0;

        for c in s.chars() {
            for lower_c in c.to_lowercase() {
                let mut char_buf = [0u8; 4];
                let encoded = lower_c.encode_utf8(&mut char_buf).as_bytes();

                if index + encoded.len() > STACK_BUF_SIZE {
                    return atom(&s.to_lowercase());
                }

                stack_buf[index..index + encoded.len()].copy_from_slice(encoded);
                index += encoded.len();
            }
        }

        return atom(
            // SAFETY: We only write valid UTF-8 bytes into the stack buffer.
            unsafe { std::str::from_utf8_unchecked(&stack_buf[..index]) },
        );
    }

    atom(&s.to_lowercase())
}

/// Checks if `haystack` starts with `prefix`, ignoring ASCII case.
///
/// This function uses SIMD instructions (AVX2 on `x86_64`, NEON on aarch64)
/// when available and beneficial for the input size.
///
/// # Examples
///
/// ```
/// use mago_atom::starts_with_ignore_case;
///
/// assert!(starts_with_ignore_case("HelloWorld", "hello"));
/// assert!(starts_with_ignore_case("FOOBAR", "FooBar"));
/// assert!(starts_with_ignore_case("test", "TEST"));
/// assert!(!starts_with_ignore_case("hello", "world"));
/// assert!(!starts_with_ignore_case("hi", "hello"));
/// ```
#[inline]
#[must_use]
pub fn starts_with_ignore_case(haystack: &str, prefix: &str) -> bool {
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn starts_with_avx2(haystack: &str, prefix: &str, len: usize) -> bool {
        unsafe {
            let haystack_bytes = haystack.as_bytes();
            let prefix_bytes = prefix.as_bytes();

            let lower_a = _mm256_set1_epi8(b'a' as i8);
            let lower_z = _mm256_set1_epi8(b'z' as i8);
            let case_bit = _mm256_set1_epi8(0x20);

            let mut i = 0;
            while i + 32 <= len {
                let h = _mm256_loadu_si256(haystack_bytes.as_ptr().add(i) as *const __m256i);
                let p = _mm256_loadu_si256(prefix_bytes.as_ptr().add(i) as *const __m256i);

                // Convert haystack chunk to lowercase
                let h_is_lower = _mm256_and_si256(
                    _mm256_cmpgt_epi8(h, _mm256_sub_epi8(lower_a, _mm256_set1_epi8(1))),
                    _mm256_cmpgt_epi8(_mm256_add_epi8(lower_z, _mm256_set1_epi8(1)), h),
                );
                let h_lower = _mm256_or_si256(h, _mm256_and_si256(h_is_lower, case_bit));

                // Convert prefix chunk to lowercase
                let p_is_lower = _mm256_and_si256(
                    _mm256_cmpgt_epi8(p, _mm256_sub_epi8(lower_a, _mm256_set1_epi8(1))),
                    _mm256_cmpgt_epi8(_mm256_add_epi8(lower_z, _mm256_set1_epi8(1)), p),
                );
                let p_lower = _mm256_or_si256(p, _mm256_and_si256(p_is_lower, case_bit));

                let eq = _mm256_cmpeq_epi8(h_lower, p_lower);
                let mask = _mm256_movemask_epi8(eq);
                if mask != -1i32 {
                    return false;
                }

                i += 32;
            }

            // Handle remaining bytes
            haystack_bytes[i..len].eq_ignore_ascii_case(&prefix_bytes[i..len])
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn starts_with_neon(haystack: &str, prefix: &str, len: usize) -> bool {
        unsafe {
            let haystack_bytes = haystack.as_bytes();
            let prefix_bytes = prefix.as_bytes();

            let lower_a = vdupq_n_u8(b'a');
            let lower_z = vdupq_n_u8(b'z');
            let case_bit = vdupq_n_u8(0x20);

            let mut i = 0;
            while i + 16 <= len {
                let h = vld1q_u8(haystack_bytes.as_ptr().add(i));
                let p = vld1q_u8(prefix_bytes.as_ptr().add(i));

                // Convert haystack chunk to lowercase
                let h_ge_a = vcgeq_u8(h, lower_a);
                let h_le_z = vcleq_u8(h, lower_z);
                let h_is_lower = vandq_u8(h_ge_a, h_le_z);
                let h_lower = vorrq_u8(h, vandq_u8(h_is_lower, case_bit));

                // Convert prefix chunk to lowercase
                let p_ge_a = vcgeq_u8(p, lower_a);
                let p_le_z = vcleq_u8(p, lower_z);
                let p_is_lower = vandq_u8(p_ge_a, p_le_z);
                let p_lower = vorrq_u8(p, vandq_u8(p_is_lower, case_bit));

                let eq = vceqq_u8(h_lower, p_lower);
                let min = vminvq_u8(eq);
                if min != 0xFF {
                    return false;
                }

                i += 16;
            }

            // Handle remaining bytes
            haystack_bytes[i..len].eq_ignore_ascii_case(&prefix_bytes[i..len])
        }
    }

    let len = prefix.len();
    if haystack.len() < len {
        return false;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if len >= 32 && std::is_x86_feature_detected!("avx2") {
            // SAFETY: we've checked that AVX2 is available and haystack.len() >= len
            return unsafe { starts_with_avx2(haystack, prefix, len) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if len >= 16 {
            // SAFETY: NEON is always available on aarch64 and haystack.len() >= len
            return unsafe { starts_with_neon(haystack, prefix, len) };
        }
    }

    haystack.as_bytes()[..len].eq_ignore_ascii_case(prefix.as_bytes())
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
