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
use std::arch::x86_64::__m256i;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_add_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_and_si256;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_cmpeq_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_cmpgt_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_loadu_si256;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_movemask_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_or_si256;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_set1_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm256_sub_epi8;

use crate::Word;

/// The maximum size in bytes for a byte sequence to be processed on the stack.
const STACK_BUF_SIZE: usize = 256;

thread_local! {
    static EMPTY_WORD: Word = Word::new(b"");
}

/// Returns the canonical [`Word`] for the empty byte sequence.
///
/// This is a very cheap operation.
#[inline]
#[must_use]
pub fn empty_word() -> Word {
    EMPTY_WORD.with(|&w| w)
}

/// Joins a slice of words with `separator`, interning only the complete result.
///
/// Results up to 256 bytes are assembled on the stack. This is preferable to
/// repeatedly concatenating an accumulator because intermediate prefixes never
/// enter the global interner.
#[inline]
#[must_use]
pub fn join_words(words: &[Word], separator: &[u8]) -> Word {
    let Some((first, rest)) = words.split_first() else {
        return empty_word();
    };

    if rest.is_empty() {
        return *first;
    }

    let words_len = words.iter().map(|word| word.len()).sum::<usize>();
    let total_len = words_len + separator.len() * rest.len();

    if total_len <= STACK_BUF_SIZE {
        let mut buffer = [0u8; STACK_BUF_SIZE];
        let mut offset = 0;

        for (index, word) in words.iter().enumerate() {
            if index != 0 {
                buffer[offset..offset + separator.len()].copy_from_slice(separator);
                offset += separator.len();
            }

            let bytes = word.as_bytes();
            buffer[offset..offset + bytes.len()].copy_from_slice(bytes);
            offset += bytes.len();
        }

        return Word::new(&buffer[..total_len]);
    }

    let mut result = Vec::with_capacity(total_len);
    result.extend_from_slice(first.as_bytes());
    for word in rest {
        result.extend_from_slice(separator);
        result.extend_from_slice(word.as_bytes());
    }

    Word::new(&result)
}

/// A macro to concatenate between 2 and 12 byte sequences into a single [`Word`].
///
/// Each argument may be anything that implements `AsRef<[u8]>` (e.g. `&[u8]`, `&str`,
/// `Vec<u8>`, `String`, [`Word`]). The macro dispatches to a specialized,
/// zero-heap-allocation function based on the number of arguments. Inputs whose total
/// length fits in a stack buffer (~256 bytes) avoid the heap entirely.
///
/// # Panics
///
/// Panics at compile time if called with 0, 1, or more than 12 arguments.
#[macro_export]
macro_rules! concat_word {
    ($s1:expr, $s2:expr $(,)?) => {
        $crate::concat_word2($s1.as_ref(), $s2.as_ref())
    };
    ($s1:expr, $s2:expr, $s3:expr $(,)?) => {
        $crate::concat_word3($s1.as_ref(), $s2.as_ref(), $s3.as_ref())
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr $(,)?) => {
        $crate::concat_word4($s1.as_ref(), $s2.as_ref(), $s3.as_ref(), $s4.as_ref())
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr $(,)?) => {
        $crate::concat_word5($s1.as_ref(), $s2.as_ref(), $s3.as_ref(), $s4.as_ref(), $s5.as_ref())
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr $(,)?) => {
        $crate::concat_word6($s1.as_ref(), $s2.as_ref(), $s3.as_ref(), $s4.as_ref(), $s5.as_ref(), $s6.as_ref())
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr $(,)?) => {
        $crate::concat_word7(
            $s1.as_ref(),
            $s2.as_ref(),
            $s3.as_ref(),
            $s4.as_ref(),
            $s5.as_ref(),
            $s6.as_ref(),
            $s7.as_ref(),
        )
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr $(,)?) => {
        $crate::concat_word8(
            $s1.as_ref(),
            $s2.as_ref(),
            $s3.as_ref(),
            $s4.as_ref(),
            $s5.as_ref(),
            $s6.as_ref(),
            $s7.as_ref(),
            $s8.as_ref(),
        )
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr $(,)?) => {
        $crate::concat_word9(
            $s1.as_ref(),
            $s2.as_ref(),
            $s3.as_ref(),
            $s4.as_ref(),
            $s5.as_ref(),
            $s6.as_ref(),
            $s7.as_ref(),
            $s8.as_ref(),
            $s9.as_ref(),
        )
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr, $s10:expr $(,)?) => {
        $crate::concat_word10(
            $s1.as_ref(),
            $s2.as_ref(),
            $s3.as_ref(),
            $s4.as_ref(),
            $s5.as_ref(),
            $s6.as_ref(),
            $s7.as_ref(),
            $s8.as_ref(),
            $s9.as_ref(),
            $s10.as_ref(),
        )
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr, $s10:expr, $s11:expr $(,)?) => {
        $crate::concat_word11(
            $s1.as_ref(),
            $s2.as_ref(),
            $s3.as_ref(),
            $s4.as_ref(),
            $s5.as_ref(),
            $s6.as_ref(),
            $s7.as_ref(),
            $s8.as_ref(),
            $s9.as_ref(),
            $s10.as_ref(),
            $s11.as_ref(),
        )
    };
    ($s1:expr, $s2:expr, $s3:expr, $s4:expr, $s5:expr, $s6:expr, $s7:expr, $s8:expr, $s9:expr, $s10:expr, $s11:expr, $s12:expr $(,)?) => {
        $crate::concat_word12(
            $s1.as_ref(),
            $s2.as_ref(),
            $s3.as_ref(),
            $s4.as_ref(),
            $s5.as_ref(),
            $s6.as_ref(),
            $s7.as_ref(),
            $s8.as_ref(),
            $s9.as_ref(),
            $s10.as_ref(),
            $s11.as_ref(),
            $s12.as_ref(),
        )
    };
    ($($arg:expr),+ $(,)?) => {
        compile_error!("concat_word! macro supports between 2 and 12 arguments only")
    };
}

/// Lowercases the namespace part of a PHP-style fully-qualified constant name,
/// preserving the constant name itself.
///
/// For `\Foo\Bar\CONST`, returns the [`Word`] for `\foo\bar\CONST`. ASCII case folding
/// only — bytes outside ASCII are left unchanged. Uses a stack buffer when the input
/// fits in [`STACK_BUF_SIZE`] bytes, otherwise allocates on the heap.
#[inline]
#[must_use]
pub fn ascii_lowercase_constant_name_word(name: &[u8]) -> Word {
    if let Some(last_slash_idx) = name.iter().rposition(|&b| b == b'\\') {
        let (namespace, rest) = name.split_at(last_slash_idx);
        let const_name = &rest[1..];

        if name.len() > STACK_BUF_SIZE {
            let mut buffer = Vec::with_capacity(name.len());
            buffer.extend(namespace.iter().map(u8::to_ascii_lowercase));
            buffer.push(b'\\');
            buffer.extend_from_slice(const_name);
            return Word::new(&buffer);
        }

        let mut stack_buf = [0u8; STACK_BUF_SIZE];
        let mut index = 0;

        for &byte in namespace {
            stack_buf[index] = byte.to_ascii_lowercase();
            index += 1;
        }

        stack_buf[index] = b'\\';
        index += 1;

        stack_buf[index..index + const_name.len()].copy_from_slice(const_name);
        index += const_name.len();

        Word::new(&stack_buf[..index])
    } else {
        Word::new(name)
    }
}

/// Returns a [`Word`] of the ASCII-lowercased form of `bytes`.
///
/// Bytes outside ASCII are passed through unchanged. Uses a fast scan first: if no
/// uppercase ASCII byte is present, the original is interned without copying. Otherwise
/// the lowercased form is built on the stack for inputs up to [`STACK_BUF_SIZE`] bytes,
/// or on the heap for longer inputs.
#[inline]
#[must_use]
pub fn ascii_lowercase_word(bytes: &[u8]) -> Word {
    if !bytes.iter().any(u8::is_ascii_uppercase) {
        return Word::new(bytes);
    }

    if bytes.len() <= STACK_BUF_SIZE {
        let mut stack_buf = [0u8; STACK_BUF_SIZE];
        for (i, &b) in bytes.iter().enumerate() {
            stack_buf[i] = b.to_ascii_lowercase();
        }
        return Word::new(&stack_buf[..bytes.len()]);
    }

    let lowered: Vec<u8> = bytes.iter().map(u8::to_ascii_lowercase).collect();
    Word::new(&lowered)
}

/// Checks if `haystack` starts with `prefix`, ignoring ASCII case.
///
/// Uses SIMD when available (AVX2 on x86_64, NEON on aarch64) for long prefixes.
/// ASCII case folding only — bytes outside ASCII compare exactly.
///
/// # Examples
///
/// ```
/// use mago_word::starts_with_ignore_case;
///
/// assert!(starts_with_ignore_case(b"HelloWorld", b"hello"));
/// assert!(starts_with_ignore_case(b"FOOBAR", b"FooBar"));
/// assert!(!starts_with_ignore_case(b"hello", b"world"));
/// assert!(!starts_with_ignore_case(b"hi", b"hello"));
/// ```
#[inline]
#[must_use]
pub fn starts_with_ignore_case(haystack: &[u8], prefix: &[u8]) -> bool {
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn starts_with_avx2(haystack: &[u8], prefix: &[u8], len: usize) -> bool {
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        // SAFETY: caller has verified AVX2 is available and `haystack.len() >= len`; loads of 32 bytes from
        // `haystack[i..i+32]` and `prefix[i..i+32]` stay within bounds because the loop guard is `i + 32 <= len`.
        unsafe {
            let upper_a = _mm256_set1_epi8(b'A' as i8);
            let upper_z = _mm256_set1_epi8(b'Z' as i8);
            let case_bit = _mm256_set1_epi8(0x20);

            let mut i = 0;
            while i + 32 <= len {
                // SAFETY: `_mm256_loadu_si256` performs an unaligned load, so the pointer
                // need not satisfy `__m256i`'s 32-byte alignment requirement.
                #[allow(clippy::cast_ptr_alignment)]
                let h = _mm256_loadu_si256(haystack.as_ptr().add(i).cast::<__m256i>());
                #[allow(clippy::cast_ptr_alignment)]
                let p = _mm256_loadu_si256(prefix.as_ptr().add(i).cast::<__m256i>());

                let h_is_upper = _mm256_and_si256(
                    _mm256_cmpgt_epi8(h, _mm256_sub_epi8(upper_a, _mm256_set1_epi8(1))),
                    _mm256_cmpgt_epi8(_mm256_add_epi8(upper_z, _mm256_set1_epi8(1)), h),
                );
                let h_lower = _mm256_or_si256(h, _mm256_and_si256(h_is_upper, case_bit));

                let p_is_upper = _mm256_and_si256(
                    _mm256_cmpgt_epi8(p, _mm256_sub_epi8(upper_a, _mm256_set1_epi8(1))),
                    _mm256_cmpgt_epi8(_mm256_add_epi8(upper_z, _mm256_set1_epi8(1)), p),
                );
                let p_lower = _mm256_or_si256(p, _mm256_and_si256(p_is_upper, case_bit));

                let eq = _mm256_cmpeq_epi8(h_lower, p_lower);
                let mask = _mm256_movemask_epi8(eq);
                if mask != -1i32 {
                    return false;
                }

                i += 32;
            }

            haystack[i..len].eq_ignore_ascii_case(&prefix[i..len])
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn starts_with_neon(haystack: &[u8], prefix: &[u8], len: usize) -> bool {
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        // SAFETY: NEON is always available on aarch64 and the caller has verified `haystack.len() >= len`; loads of
        // 16 bytes from `haystack[i..i+16]` and `prefix[i..i+16]` stay within bounds because the loop guard is
        // `i + 16 <= len`.
        unsafe {
            let upper_a = vdupq_n_u8(b'A');
            let upper_z = vdupq_n_u8(b'Z');
            let case_bit = vdupq_n_u8(0x20);

            let mut i = 0;
            while i + 16 <= len {
                let h = vld1q_u8(haystack.as_ptr().add(i));
                let p = vld1q_u8(prefix.as_ptr().add(i));

                let h_ge_a = vcgeq_u8(h, upper_a);
                let h_le_z = vcleq_u8(h, upper_z);
                let h_is_upper = vandq_u8(h_ge_a, h_le_z);
                let h_lower = vorrq_u8(h, vandq_u8(h_is_upper, case_bit));

                let p_ge_a = vcgeq_u8(p, upper_a);
                let p_le_z = vcleq_u8(p, upper_z);
                let p_is_upper = vandq_u8(p_ge_a, p_le_z);
                let p_lower = vorrq_u8(p, vandq_u8(p_is_upper, case_bit));

                let eq = vceqq_u8(h_lower, p_lower);
                let min = vminvq_u8(eq);
                if min != 0xFF {
                    return false;
                }

                i += 16;
            }

            haystack[i..len].eq_ignore_ascii_case(&prefix[i..len])
        }
    }

    let len = prefix.len();
    if haystack.len() < len {
        return false;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if len >= 32 && std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 detected, haystack.len() >= len.
            return unsafe { starts_with_avx2(haystack, prefix, len) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if len >= 16 {
            // SAFETY: NEON is always available on aarch64, haystack.len() >= len.
            return unsafe { starts_with_neon(haystack, prefix, len) };
        }
    }

    haystack[..len].eq_ignore_ascii_case(prefix)
}

/// A helper macro to generate the specialized `*_word` functions for integer types.
macro_rules! integer_to_word_fns {
    ( $( $func_name:ident($num_type:ty) ),+ $(,)? ) => {
        $(
            #[doc = "Creates a `Word` from a `"]
            #[doc = stringify!($num_type)]
            #[doc = "` value with zero heap allocations."]
            #[inline]
            #[must_use]
            pub fn $func_name(n: $num_type) -> Word {
                let mut buffer = itoa::Buffer::new();
                let s = buffer.format(n);

                Word::new(s.as_bytes())
            }
        )+
    };
}

/// A helper macro to generate the specialized `*_word` functions for float types.
macro_rules! float_to_word_fns {
    ( $( $func_name:ident($num_type:ty) ),+ $(,)? ) => {
        $(
            #[doc = "Creates a `Word` from a `"]
            #[doc = stringify!($num_type)]
            #[doc = "` value with zero heap allocations."]
            #[inline]
            #[must_use]
            pub fn $func_name(n: $num_type) -> Word {
                let mut buffer = ryu::Buffer::new();
                let s = buffer.format(n);

                Word::new(s.as_bytes())
            }
        )+
    };
}

/// A helper macro to generate the specialized `concat_wordN` functions.
macro_rules! concat_fns {
    ( $( $func_name:ident($n:literal, $($s:ident),+) ),+ $(,)?) => {
        $(
            #[doc = "Creates a `Word` as a result of concatenating "]
            #[doc = stringify!($n)]
            #[doc = " byte sequences."]
            #[inline]
            #[must_use]
            #[allow(unused_assignments)]
            #[allow(clippy::too_many_arguments)]
            pub fn $func_name($($s: &[u8]),+) -> Word {
                let total_len = 0 $(+ $s.len())+;

                if total_len <= STACK_BUF_SIZE {
                    let mut buffer = [0u8; STACK_BUF_SIZE];
                    let mut index = 0;
                    $(
                        buffer[index..index + $s.len()].copy_from_slice($s);
                        index += $s.len();
                    )+

                    return Word::new(&buffer[..total_len]);
                }

                let mut result = Vec::with_capacity(total_len);
                $( result.extend_from_slice($s); )+
                Word::new(&result)
            }
        )+
    };
}

integer_to_word_fns!(
    i8_word(i8),
    i16_word(i16),
    i32_word(i32),
    i64_word(i64),
    i128_word(i128),
    isize_word(isize),
    u8_word(u8),
    u16_word(u16),
    u32_word(u32),
    u64_word(u64),
    u128_word(u128),
    usize_word(usize),
);

float_to_word_fns!(f32_word(f32), f64_word(f64),);

concat_fns!(
    concat_word2(2, s1, s2),
    concat_word3(3, s1, s2, s3),
    concat_word4(4, s1, s2, s3, s4),
    concat_word5(5, s1, s2, s3, s4, s5),
    concat_word6(6, s1, s2, s3, s4, s5, s6),
    concat_word7(7, s1, s2, s3, s4, s5, s6, s7),
    concat_word8(8, s1, s2, s3, s4, s5, s6, s7, s8),
    concat_word9(9, s1, s2, s3, s4, s5, s6, s7, s8, s9),
    concat_word10(10, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10),
    concat_word11(11, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11),
    concat_word12(12, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12),
);
