use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[cfg(not(feature = "sso"))]
use std::ptr::NonNull;

use crate::interner::Entry;
use crate::interner::intern;

/// Maximum byte length stored inline in a [`Word`] when the `sso` feature is enabled.
///
/// One byte of the 16-byte representation is reserved for the tag, leaving 15 for content.
#[cfg(feature = "sso")]
pub const INLINE_CAPACITY: usize = 15;

#[cfg(feature = "sso")]
const TAG_HEAP: u8 = 0xFF;

/// A globally-interned, byte-string handle.
///
/// `Word` is `Copy` and cheap to compare and hash — equality is bitwise, hashing is
/// of the canonical bit pattern, and two `Word`s with the same content are guaranteed
/// to have identical bits.
///
/// # Storage
///
/// - With `sso` enabled (the default), `Word` is 16 bytes. Sequences up to
///   [`INLINE_CAPACITY`] are stored inline in the handle itself; longer sequences are
///   stored in a leaked global arena and the handle is a pointer to that arena.
/// - Without `sso`, every `Word` is an interner pointer. Same size as a `NonNull`.
///
/// In both configurations the bit pattern of the handle is canonical: two `Word`s
/// representing the same content compare equal byte-for-byte.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Word {
    repr: Repr,
}

#[cfg(feature = "sso")]
#[derive(Copy, Clone)]
#[repr(C)]
union Repr {
    inline: Inline,
    heap: Heap,
    bytes: [u8; 16],
}

#[cfg(feature = "sso")]
#[derive(Copy, Clone)]
#[repr(C)]
struct Inline {
    tag: u8,
    bytes: [u8; INLINE_CAPACITY],
}

#[cfg(feature = "sso")]
#[derive(Copy, Clone)]
#[repr(C)]
struct Heap {
    tag: u8,
    _pad: [u8; 7],
    ptr: usize,
}

#[cfg(not(feature = "sso"))]
#[derive(Copy, Clone)]
#[repr(transparent)]
struct Repr {
    ptr: NonNull<Entry>,
}

// SAFETY: arena memory is never freed, never mutated, and is reachable from any thread.
unsafe impl Send for Word {}
// SAFETY: same as `Send`: the interned bytes are immutable and live for the program's
// lifetime, so concurrent reads are safe.
unsafe impl Sync for Word {}

impl Word {
    /// Interns `bytes` and returns its canonical [`Word`].
    #[inline]
    #[must_use]
    pub fn new(bytes: &[u8]) -> Word {
        Word::from_bytes(bytes)
    }

    #[cfg(feature = "sso")]
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Word {
        if bytes.len() <= INLINE_CAPACITY {
            let mut inline = Inline { tag: bytes.len() as u8, bytes: [0u8; INLINE_CAPACITY] };
            inline.bytes[..bytes.len()].copy_from_slice(bytes);
            Word { repr: Repr { inline } }
        } else {
            let ptr = intern(bytes).as_ptr() as usize;
            let heap = Heap { tag: TAG_HEAP, _pad: [0u8; 7], ptr };
            Word { repr: Repr { heap } }
        }
    }

    #[cfg(not(feature = "sso"))]
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Word {
        Word { repr: Repr { ptr: intern(bytes) } }
    }

    /// Returns the byte content of this `Word`.
    #[inline]
    #[must_use]
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    pub fn as_bytes(&self) -> &[u8] {
        #[cfg(feature = "sso")]
        // SAFETY: the `tag` field aliases the first byte of either union arm, and inline
        // bytes are always initialized at construction; when `tag == TAG_HEAP`, `heap.ptr`
        // is the address of an `Entry` written by the interner, so the deref and `as_bytes`
        // (whose contract is documented on `Entry::as_bytes`) are sound.
        unsafe {
            let tag = self.repr.inline.tag;
            if tag == TAG_HEAP {
                let entry = self.repr.heap.ptr as *const Entry;
                (*entry).as_bytes()
            } else {
                &self.repr.inline.bytes[..tag as usize]
            }
        }

        #[cfg(not(feature = "sso"))]
        // SAFETY: `repr.ptr` is the NonNull written at construction; the entry it points to
        // lives for the lifetime of the process.
        unsafe {
            self.repr.ptr.as_ref().as_bytes()
        }
    }

    /// Returns the length, in bytes, of this `Word`.
    #[inline]
    #[must_use]
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    pub fn len(&self) -> usize {
        #[cfg(feature = "sso")]
        // SAFETY: see `as_bytes` — the same union discipline applies.
        unsafe {
            let tag = self.repr.inline.tag;
            if tag == TAG_HEAP {
                let entry = self.repr.heap.ptr as *const Entry;
                (*entry).len as usize
            } else {
                tag as usize
            }
        }

        #[cfg(not(feature = "sso"))]
        // SAFETY: see `as_bytes` — `repr.ptr` is the canonical interner pointer.
        unsafe {
            self.repr.ptr.as_ref().len as usize
        }
    }

    /// Returns `true` if this `Word` has zero length.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the bytes interpreted as UTF-8 if they are valid; otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_bytes()).ok()
    }

    /// Returns the bytes as UTF-8, replacing invalid sequences with the replacement
    /// character (`U+FFFD`).
    #[inline]
    #[must_use]
    pub fn as_str_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    /// Returns `true` if this `Word`'s content is stored inline in the handle.
    ///
    /// Always `false` when the `sso` feature is disabled.
    #[inline]
    #[must_use]
    pub fn is_inline(&self) -> bool {
        #[cfg(feature = "sso")]
        // SAFETY: `inline.tag` aliases the first byte of the 16-byte representation; reading
        // it is valid regardless of which arm of the union currently holds data.
        unsafe {
            self.repr.inline.tag != TAG_HEAP
        }
        #[cfg(not(feature = "sso"))]
        {
            false
        }
    }
}

#[cfg(feature = "sso")]
impl PartialEq for Word {
    #[inline]
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: `repr.bytes` is the canonical 16-byte representation of any `Word` regardless
        // of which union arm is active, so comparing the two byte arrays is well-defined.
        unsafe { self.repr.bytes == other.repr.bytes }
    }
}

#[cfg(not(feature = "sso"))]
impl PartialEq for Word {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.repr.ptr == other.repr.ptr
    }
}

impl Eq for Word {}

#[cfg(feature = "sso")]
impl Hash for Word {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        // SAFETY: `repr.bytes` is the canonical 16-byte representation; hashing it gives a
        // value-determined hash regardless of which arm of the union is active.
        unsafe { state.write(&self.repr.bytes) }
    }
}

#[cfg(not(feature = "sso"))]
impl Hash for Word {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_usize(self.repr.ptr.as_ptr() as usize);
    }
}

impl PartialOrd for Word {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Word {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(s) => fmt::Debug::fmt(s, f),
            None => fmt::Debug::fmt(self.as_bytes(), f),
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        mago_bytes::write_escaped(f, self.as_bytes())
    }
}

impl From<&[u8]> for Word {
    #[inline]
    fn from(bytes: &[u8]) -> Word {
        Word::new(bytes)
    }
}

impl From<&str> for Word {
    #[inline]
    fn from(s: &str) -> Word {
        Word::new(s.as_bytes())
    }
}

impl AsRef<[u8]> for Word {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Word {
    #[inline]
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser.serialize_bytes(self.as_bytes())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Word {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;

        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Word;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a byte sequence")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Word::new(v))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Word::new(&v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Word::new(v.as_bytes()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Word::new(v.as_bytes()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut out: Vec<u8> = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(b) = seq.next_element::<u8>()? {
                    out.push(b);
                }
                Ok(Word::new(&out))
            }
        }

        de.deserialize_bytes(V)
    }
}
