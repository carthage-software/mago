use std::hash::BuildHasher;
use std::hash::Hasher;
use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::OnceLock;

use foldhash::fast::FixedState;

use crate::bumpalloc::LeakyBumpAlloc;

/// Header that precedes the bytes of an interned [`Word`](crate::Word) entry.
///
/// Memory layout, written contiguously by the arena, with no padding between the header
/// and the bytes (entry alignment is the natural alignment of [`Entry`] itself, and the
/// bytes follow immediately):
///
/// ```text
/// | hash: u64 | len: u32 | _pad: u32 | bytes: [u8; len] |
///   ^ Entry                            ^ Entry::bytes()
/// ```
#[repr(C)]
pub(crate) struct Entry {
    pub(crate) hash: u64,
    pub(crate) len: u32,
    _pad: u32,
}

impl Entry {
    /// Returns the interned byte slice that follows an entry's header in the arena.
    ///
    /// Takes a raw pointer rather than `&self` on purpose: a `&Entry` carries provenance
    /// over only the header, so reading the trailing bytes through it is out of bounds
    /// under Stacked Borrows. `ptr` comes straight from the arena allocation and carries
    /// provenance over the whole header-plus-bytes block.
    ///
    /// # Safety
    ///
    /// `ptr` must point at an `Entry` produced by the interner: a header followed by `len`
    /// initialized bytes inside the leaked arena, with provenance over that whole block.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    pub(crate) unsafe fn bytes(ptr: *const Entry) -> &'static [u8] {
        // SAFETY: `ptr` is a valid pointer to an `Entry` header, so reading the `len` field is safe.
        unsafe {
            let len = (*ptr).len as usize;
            let data = ptr.cast::<u8>().add(std::mem::size_of::<Entry>());
            std::slice::from_raw_parts(data, len)
        }
    }
}

/// Number of shards. Each shard owns its own mutex, table, and arena, so concurrent
/// interning rarely contends.
const NUM_SHARDS: usize = 64;

/// Initial table capacity (slots) per shard. Sized so the prelude warm-up doesn't
/// trigger many resizes.
const INITIAL_TABLE_CAPACITY: usize = 1 << 14; // 16K slots/shard, 1M total

/// Initial bytes per shard arena. Grows by doubling on overflow.
const INITIAL_ARENA_BYTES: usize = 64 * 1024;

struct Shard {
    /// Open-addressed table of pointers to entries in the arena. `null` is the
    /// sentinel for an empty slot. Load factor is kept under 0.5 by [`grow`].
    entries: Vec<*mut Entry>,
    mask: usize,
    num_entries: usize,
    /// Current arena. Old arenas are kept alive in [`old_allocs`] so previously-
    /// handed-out pointers remain valid for the lifetime of the process.
    alloc: LeakyBumpAlloc,
    old_allocs: Vec<LeakyBumpAlloc>,
}

impl Shard {
    fn new() -> Shard {
        let capacity = INITIAL_TABLE_CAPACITY;
        Shard {
            entries: vec![std::ptr::null_mut(); capacity],
            mask: capacity - 1,
            num_entries: 0,
            alloc: LeakyBumpAlloc::new(INITIAL_ARENA_BYTES, std::mem::align_of::<Entry>()),
            old_allocs: Vec::new(),
        }
    }

    /// Looks up `bytes` in this shard, or inserts a new entry and returns its pointer.
    /// The returned pointer is stable for the lifetime of the process.
    #[allow(clippy::multiple_unsafe_ops_per_block)]
    #[allow(clippy::cast_ptr_alignment)]
    #[allow(clippy::expect_used)]
    #[allow(clippy::similar_names)]
    fn intern(&mut self, bytes: &[u8], hash: u64) -> NonNull<Entry> {
        let mut pos = (hash as usize) & self.mask;
        let mut probe: usize = 0;

        loop {
            // SAFETY: `pos` is masked with `self.mask = entries.len() - 1`, so it is always
            // within bounds of `entries`.
            let slot = unsafe { *self.entries.get_unchecked(pos) };
            if slot.is_null() {
                break;
            }

            // SAFETY: every non-null slot points at an `Entry` written by a previous `intern`
            // call; the arena outlives the program. Reading the header fields and trailing
            // bytes through the raw `slot` keeps its whole-block provenance (see `Entry::bytes`).
            let matched = unsafe {
                (*slot).hash == hash && (*slot).len as usize == bytes.len() && Entry::bytes(slot) == bytes
            };

            if matched {
                // SAFETY: `slot` was checked non-null above.
                return unsafe { NonNull::new_unchecked(slot) };
            }

            probe += 1;
            debug_assert!(probe <= self.mask, "interner probe walked the entire shard");
            pos = (pos + probe) & self.mask;
        }

        let entry_size = std::mem::size_of::<Entry>() + bytes.len();
        let aligned_size = round_up(entry_size, std::mem::align_of::<Entry>());
        if self.alloc.allocated() + aligned_size > self.alloc.capacity() {
            // `expect` here: doubling a usize past `usize::MAX` is impossible while the
            // process has free virtual memory; this is a programmer-bug guard, not runtime.
            let new_capacity =
                self.alloc.capacity().checked_mul(2).expect("arena capacity doubled overflow").max(aligned_size);
            let new_alloc = LeakyBumpAlloc::new(new_capacity, std::mem::align_of::<Entry>());
            let old_alloc = std::mem::replace(&mut self.alloc, new_alloc);
            self.old_allocs.push(old_alloc);
        }

        // SAFETY: `allocate` returns a pointer to a freshly-reserved `aligned_size`-byte block
        // inside the current arena. The block is aligned to `align_of::<Entry>()` (matching
        // the arena's construction), so casting to `*mut Entry` is sound.
        let ptr = unsafe { self.alloc.allocate(aligned_size) }.cast::<Entry>();
        // SAFETY: `ptr` is the freshly-reserved aligned block. We write the header followed
        // by exactly `bytes.len()` bytes; the offset `size_of::<Entry>()` stays inside the
        // `aligned_size` region we just reserved.
        unsafe {
            std::ptr::write(
                ptr,
                Entry { hash, len: u32::try_from(bytes.len()).expect("word length exceeds u32::MAX"), _pad: 0 },
            );

            let dest_ptr = ptr.cast::<u8>().add(std::mem::size_of::<Entry>());
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest_ptr, bytes.len());
        }

        // SAFETY: `pos` is within `entries` bounds (see top of function).
        unsafe { *self.entries.get_unchecked_mut(pos) = ptr };
        self.num_entries += 1;
        if self.num_entries * 2 > self.mask {
            // SAFETY: `grow` rehashes existing entries into a fresh table; no aliasing.
            unsafe { self.grow() };
        }

        // SAFETY: `ptr` was just written-to and is non-null (allocation didn't return null —
        // OOM aborts inside `LeakyBumpAlloc::allocate`).
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// Doubles the table size and re-inserts every existing pointer. Existing arena
    /// pointers are unchanged — only the index table grows.
    unsafe fn grow(&mut self) {
        let new_mask = self.mask * 2 + 1;
        let mut new_entries: Vec<*mut Entry> = vec![std::ptr::null_mut(); new_mask + 1];

        for slot in &self.entries {
            let ptr = *slot;
            if ptr.is_null() {
                continue;
            }

            // SAFETY: every non-null slot is a pointer to an `Entry` previously written by
            // `intern`; the arena is leaky so the pointer is still valid.
            let hash = unsafe { (*ptr).hash };
            let mut pos = (hash as usize) & new_mask;
            let mut probe: usize = 0;
            while !new_entries[pos].is_null() {
                probe += 1;
                debug_assert!(probe <= new_mask, "grow probe walked the entire shard");
                pos = (pos + probe) & new_mask;
            }

            new_entries[pos] = ptr;
        }

        self.entries = new_entries;
        self.mask = new_mask;
    }
}

// SAFETY: A `Shard` owns its arena and table exclusively; the surrounding `Mutex` ensures
// only one thread mutates at a time. Pointers in the table outlive every thread because the
// arena is leaked, so transferring ownership across threads is sound.
unsafe impl Send for Shard {}

/// Global interner. Shards are lazily initialized on first use through the
/// surrounding [`OnceLock`].
struct Interner {
    shards: [Mutex<Shard>; NUM_SHARDS],
    hasher: FixedState,
}

impl Interner {
    fn new() -> Interner {
        Interner { shards: std::array::from_fn(|_| Mutex::new(Shard::new())), hasher: FixedState::default() }
    }
}

static INTERNER: OnceLock<Interner> = OnceLock::new();

fn interner() -> &'static Interner {
    INTERNER.get_or_init(Interner::new)
}

/// Hashes `bytes` and returns the canonical entry pointer for them. Subsequent calls
/// with the same bytes return the same pointer.
#[allow(clippy::expect_used)]
pub(crate) fn intern(bytes: &[u8]) -> NonNull<Entry> {
    let interner = interner();

    let mut hasher = interner.hasher.build_hasher();
    hasher.write(bytes);
    let hash = hasher.finish();

    let shard_idx = (hash >> (u64::BITS as usize - SHARD_BITS)) as usize;
    // `expect` is acceptable here: a poisoned mutex means another thread panicked while
    // holding the interner lock, which is an unrecoverable invariant violation.
    let mut shard = interner.shards[shard_idx].lock().expect("interner shard mutex poisoned");
    shard.intern(bytes, hash)
}

/// Top-bits shift count for shard selection. Top bits of the hash, not the low bits,
/// because the low bits are also used for in-shard table indexing.
const SHARD_BITS: usize = 6; // log2(NUM_SHARDS)

const _: () = assert!(1 << SHARD_BITS == NUM_SHARDS, "SHARD_BITS must match NUM_SHARDS");

#[inline]
fn round_up(n: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "alignment must be a power of two");
    (n + align - 1) & !(align - 1)
}
