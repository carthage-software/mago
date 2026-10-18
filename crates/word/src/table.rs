//! Serialize each distinct [`Word`] once.
//!
//! Inside [`encode`], a `Word` serializes as an index into a table of the distinct words it
//! met, the most frequent first so they take the shortest indices; inside [`decode`], it
//! deserializes from such an index. Storing the table before the data lets a reader intern
//! every distinct word once, instead of hashing and interning it again at every occurrence.
//! Outside both, words serialize as bytes.

use std::cell::Cell;
use std::ptr::NonNull;

use crate::Word;
use crate::WordMap;

struct Encoding {
    indices: WordMap<u32>,
    words: Vec<Word>,
    counts: Vec<u32>,
    counting: bool,
}

thread_local! {
    static ENCODING: Cell<Option<NonNull<Encoding>>> = const { Cell::new(None) };
    static DECODING: Cell<Option<(NonNull<Word>, usize)>> = const { Cell::new(None) };
}

/// Run `serialize` with words written as table indices on this thread.
///
/// It runs twice: once to count the words, then to write them. Returns the second result
/// and the table, which a reader passes to [`decode`].
pub fn encode<T>(mut serialize: impl FnMut() -> T) -> (T, Vec<Word>) {
    let mut encoding = Encoding { indices: WordMap::default(), words: Vec::new(), counts: Vec::new(), counting: true };
    with_encoding(&mut encoding, &mut serialize);

    let mut order: Vec<usize> = (0..encoding.words.len()).collect();
    order.sort_by_key(|index| std::cmp::Reverse(encoding.counts[*index]));
    let words: Vec<Word> = order.iter().map(|index| encoding.words[*index]).collect();
    encoding.indices = words.iter().enumerate().map(|(index, word)| (*word, index as u32)).collect();
    encoding.counts = vec![0; words.len()];
    encoding.words = words;
    encoding.counting = false;
    let value = with_encoding(&mut encoding, &mut serialize);

    (value, encoding.words)
}

fn with_encoding<T>(encoding: &mut Encoding, serialize: &mut impl FnMut() -> T) -> T {
    struct Restore(Option<NonNull<Encoding>>);

    impl Drop for Restore {
        fn drop(&mut self) {
            ENCODING.set(self.0);
        }
    }

    let _restore = Restore(ENCODING.replace(Some(NonNull::from(encoding))));
    serialize()
}

/// Run `deserialize` with words read as indices into `table` on this thread.
pub fn decode<T>(table: &[Word], deserialize: impl FnOnce() -> T) -> T {
    struct Restore(Option<(NonNull<Word>, usize)>);

    impl Drop for Restore {
        fn drop(&mut self) {
            DECODING.set(self.0);
        }
    }

    let table = (NonNull::from(table).cast::<Word>(), table.len());
    let _restore = Restore(DECODING.replace(Some(table)));

    deserialize()
}

/// The index of `word` while [`encode`] runs on this thread.
pub(crate) fn encoded(word: Word) -> Option<u32> {
    let mut encoding = ENCODING.get()?;
    // SAFETY: `encode` stores this pointer only while it exclusively borrows the state,
    // on this thread, and nothing else reaches it during that time.
    let Encoding { indices, words, counts, counting } = unsafe { encoding.as_mut() };
    let index = *indices.entry(word).or_insert_with(|| {
        words.push(word);
        counts.push(0);
        (words.len() - 1) as u32
    });
    if *counting {
        counts[index as usize] += 1;
    }

    Some(index)
}

/// The table a word index refers to while [`decode`] runs on this thread.
pub(crate) fn decoding() -> Option<Table> {
    DECODING.get().map(|(words, len)| Table { words, len })
}

pub(crate) struct Table {
    words: NonNull<Word>,
    len: usize,
}

impl Table {
    pub(crate) fn get(&self, index: u32) -> Option<Word> {
        let index = index as usize;
        if index >= self.len {
            return None;
        }

        // SAFETY: `index` is in bounds of the table `decode` borrows while it stores it.
        let word = unsafe { self.words.add(index) };
        // SAFETY: as above, the table outlives this read on this thread.
        Some(unsafe { word.read() })
    }
}
