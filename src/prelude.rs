//! Decode the embedded prelude, which is stored as independently compressed frames.
//!
//! A [`Loading`] decompresses the frames in order on the thread that finishes it. Work that
//! ends early can [`help`](Loading::help) by queueing the frames nobody claimed yet on the
//! global thread pool. The finishing thread waits only for frames another thread is
//! decompressing, never inside the pool, so it cannot pick up unrelated work meanwhile.

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;

use mago_prelude::Prelude;
use zstd::zstd_safe::WriteBuf;

include!(concat!(env!("OUT_DIR"), "/prelude_frames.rs"));

static PRELUDE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/prelude.zst"));

const PENDING: u8 = 0;
const CLAIMED: u8 = 1;
const DONE: u8 = 2;

/// Decode the prelude on this thread, with help from the global thread pool if it has more than one.
pub fn load() -> Prelude {
    let loading = Loading::new();
    if rayon::current_num_threads() > 1 {
        loading.help();
    }

    loading.finish()
}

#[derive(Clone)]
pub struct Loading(Arc<Decompression>);

impl Loading {
    pub fn new() -> Self {
        Self(Arc::new(Decompression::new()))
    }

    /// Queue the frames nobody claimed yet on the global thread pool.
    pub fn help(&self) {
        for frame in (0..FRAMES.len()).rev() {
            if self.0.states[frame].load(Ordering::Relaxed) == PENDING {
                let decompression = Arc::clone(&self.0);
                rayon::spawn(move || decompression.run(frame, &mut None));
            }
        }
    }

    pub fn finish(&self) -> Prelude {
        Prelude::decode(self.0.finish()).expect("Failed to decode embedded prelude")
    }
}

struct Decompression {
    bytes: Box<[UnsafeCell<MaybeUninit<u8>>]>,
    states: [AtomicU8; FRAMES.len()],
}

// SAFETY: a frame's bytes are written only by the thread that claims it, and read only
// after every frame is done; the state of each frame orders those accesses.
unsafe impl Sync for Decompression {}

impl Decompression {
    fn new() -> Self {
        let bytes = Box::<[MaybeUninit<u8>]>::new_uninit_slice(FRAMES.iter().map(|(_, size)| size).sum());
        // SAFETY: `UnsafeCell<T>` has the same layout as `T`.
        let bytes = unsafe { Box::from_raw(Box::into_raw(bytes) as *mut [UnsafeCell<MaybeUninit<u8>>]) };

        Self { bytes, states: std::array::from_fn(|_| AtomicU8::new(PENDING)) }
    }

    /// Decompress the frame unless another thread claimed it.
    fn run(&self, frame: usize, decompressor: &mut Option<zstd::bulk::Decompressor<'static>>) {
        if self.states[frame].compare_exchange(PENDING, CLAIMED, Ordering::Acquire, Ordering::Relaxed).is_err() {
            return;
        }

        let compressed_start: usize = FRAMES[..frame].iter().map(|(compressed, _)| compressed).sum();
        let start: usize = FRAMES[..frame].iter().map(|(_, size)| size).sum();
        let (compressed, size) = FRAMES[frame];
        let bytes = UnsafeCell::raw_get(self.bytes.as_ptr()).cast::<u8>();
        // SAFETY: the frame's bytes lie within the buffer, and claiming the frame made this
        // thread the only one accessing them.
        let mut output = Region { start: unsafe { bytes.add(start) }, capacity: size, filled: 0 };
        let decompressor = match decompressor {
            Some(decompressor) => decompressor,
            None => {
                decompressor.insert(zstd::bulk::Decompressor::new().expect("Failed to decompress embedded prelude"))
            }
        };
        let written = decompressor
            .decompress_to_buffer(&PRELUDE[compressed_start..compressed_start + compressed], &mut output)
            .expect("Failed to decompress embedded prelude");
        assert_eq!(written, size, "Failed to decompress embedded prelude");

        self.states[frame].store(DONE, Ordering::Release);
    }

    fn finish(&self) -> &[u8] {
        let mut decompressor = None;
        for frame in 0..FRAMES.len() {
            self.run(frame, &mut decompressor);
        }

        for state in &self.states {
            while state.load(Ordering::Acquire) != DONE {
                std::hint::spin_loop();
            }
        }

        // SAFETY: every frame is done, so each byte is initialized and no thread writes it anymore.
        unsafe { std::slice::from_raw_parts(UnsafeCell::raw_get(self.bytes.as_ptr()).cast::<u8>(), self.bytes.len()) }
    }
}

/// A frame's part of the buffer, written by zstd before anything reads it.
struct Region {
    start: *mut u8,
    capacity: usize,
    filled: usize,
}

// SAFETY: `as_slice` covers only the bytes zstd reported as written.
unsafe impl WriteBuf for Region {
    fn as_slice(&self) -> &[u8] {
        // SAFETY: the first `filled` bytes are initialized.
        unsafe { std::slice::from_raw_parts(self.start, self.filled) }
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.start
    }

    unsafe fn filled_until(&mut self, n: usize) {
        self.filled = n;
    }
}

#[cfg(test)]
mod tests {
    use mago_database::DatabaseReader;

    use super::*;

    #[test]
    fn the_embedded_prelude_matches_its_plain_encoding() {
        let (plain, _): (Prelude, usize) = bincode::serde::decode_from_slice(
            include_bytes!(concat!(env!("OUT_DIR"), "/prelude.bin")),
            bincode::config::standard(),
        )
        .unwrap();
        let loaded = load();

        assert!(plain.metadata.class_likes.keys().eq(loaded.metadata.class_likes.keys()));
        assert_eq!(plain.metadata.class_likes.capacity(), loaded.metadata.class_likes.capacity());
        assert_eq!(plain.metadata, loaded.metadata);
        assert_eq!(plain.symbol_references, loaded.symbol_references);
        let files = |prelude: &Prelude| {
            let mut files: Vec<_> =
                prelude.database.files().map(|file| (file.id, file.name.clone(), file.contents.clone())).collect();
            files.sort_unstable_by_key(|(id, ..)| *id);
            files
        };
        assert!(files(&plain) == files(&loaded));
    }
}
