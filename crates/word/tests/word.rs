use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::word;

fn hash_of(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn empty_word_round_trips() {
    let w = Word::new(b"");
    assert_eq!(w.as_bytes(), b"");
    assert_eq!(w.len(), 0);
    assert!(w.is_empty());
}

#[test]
fn short_word_round_trips() {
    let w = Word::new(b"hello");
    assert_eq!(w.as_bytes(), b"hello");
    assert_eq!(w.len(), 5);
    assert_eq!(w.as_str(), Some("hello"));
}

#[test]
fn long_word_round_trips() {
    let needle = b"this_is_definitely_longer_than_fifteen_bytes_to_force_heap_path";
    let w = Word::new(needle);
    assert_eq!(w.as_bytes(), needle);
    assert_eq!(w.len(), needle.len());
}

#[test]
fn same_content_produces_canonical_handles() {
    let a = Word::new(b"identity_check_with_a_long_name_to_force_heap");
    let b = Word::new(b"identity_check_with_a_long_name_to_force_heap");
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));

    let c = Word::new(b"short");
    let d = Word::new(b"short");
    assert_eq!(c, d);
    assert_eq!(hash_of(&c), hash_of(&d));
}

#[test]
fn distinct_content_produces_distinct_handles() {
    let a = Word::new(b"foo");
    let b = Word::new(b"bar");
    assert_ne!(a, b);
}

#[test]
fn inline_boundary_at_fifteen_bytes_stays_inline_when_sso_is_on() {
    // 15 bytes is the boundary: still inline when sso is enabled.
    let fifteen = Word::new(b"012345678901234");
    assert_eq!(fifteen.len(), 15);
    #[cfg(feature = "sso")]
    assert!(fifteen.is_inline(), "15-byte word should be inline with sso");

    // 16 bytes spills to heap when sso is on.
    let sixteen = Word::new(b"0123456789012345");
    assert_eq!(sixteen.len(), 16);
    #[cfg(feature = "sso")]
    assert!(!sixteen.is_inline(), "16-byte word should be heap with sso");
}

#[test]
fn non_utf8_bytes_round_trip() {
    let bytes: &[u8] = &[0xFF, 0xFE, 0xFD, b'a', b'b', 0xC3, 0x28];
    let w = Word::new(bytes);
    assert_eq!(w.as_bytes(), bytes);
    assert_eq!(w.as_str(), None, "invalid UTF-8 should refuse decode");
    let lossy = w.as_str_lossy();
    assert!(lossy.contains('\u{FFFD}'), "lossy decode should produce replacement chars");
}

#[test]
fn long_non_utf8_bytes_round_trip() {
    let mut bytes = vec![0u8; 64];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = (i as u8).wrapping_add(0x80);
    }
    let w = Word::new(&bytes);
    assert_eq!(w.as_bytes(), bytes.as_slice());
}

#[test]
fn word_map_keys_resolve_by_content() {
    let mut map: WordMap<i32> = WordMap::default();
    map.insert(Word::new(b"a"), 1);
    map.insert(Word::new(b"b"), 2);
    map.insert(Word::new(b"a_quite_long_key_to_force_heap_path"), 3);

    assert_eq!(map.get(&Word::new(b"a")), Some(&1));
    assert_eq!(map.get(&Word::new(b"b")), Some(&2));
    assert_eq!(map.get(&Word::new(b"a_quite_long_key_to_force_heap_path")), Some(&3));
}

#[test]
fn word_set_dedups_by_content() {
    let mut set: WordSet = WordSet::default();
    set.insert(Word::new(b"x"));
    set.insert(Word::new(b"x"));
    set.insert(Word::new(b"yyyyyyyyyyyyyyyyyyyy_long_heap_value"));
    set.insert(Word::new(b"yyyyyyyyyyyyyyyyyyyy_long_heap_value"));
    assert_eq!(set.len(), 2);
}

#[test]
fn convenience_word_function_matches_word_new() {
    assert_eq!(word("hello"), Word::new(b"hello"));
    assert_eq!(word(b"hello".as_slice()), Word::new(b"hello"));
}

#[test]
fn from_str_and_from_bytes_match() {
    let from_str: Word = "hello".into();
    let from_bytes: Word = (b"hello" as &[u8]).into();
    assert_eq!(from_str, from_bytes);
}

#[test]
fn ordering_matches_byte_ordering() {
    let a = Word::new(b"apple");
    let b = Word::new(b"banana");
    let c = Word::new(b"cherry_with_a_quite_long_tail_to_force_heap");
    assert!(a < b);
    assert!(b < c);
}

#[test]
fn parallel_intern_is_canonical() {
    const THREADS: usize = 8;
    const KEYS_PER_THREAD: usize = 256;

    let barrier = Arc::new(Barrier::new(THREADS));
    let handles: Vec<_> = (0..THREADS)
        .map(|t| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                let mut local = Vec::with_capacity(KEYS_PER_THREAD);
                for i in 0..KEYS_PER_THREAD {
                    // Mix in both short and long keys to exercise both paths.
                    let suffix = format!("__parallel_intern_long_suffix_{}", i);
                    let key = format!("k{}{}", i, if i % 2 == 0 { "" } else { &suffix });
                    local.push((key.clone(), Word::new(key.as_bytes())));
                    let _other = Word::new(format!("thread{}", t).as_bytes());
                }
                local
            })
        })
        .collect();

    let mut all_results = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(local) => all_results.push(local),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    // Every thread that interned the same string should have gotten the same Word.
    let reference = &all_results[0];
    for thread_results in &all_results[1..] {
        for (i, (key, word)) in thread_results.iter().enumerate() {
            assert_eq!(*word, reference[i].1, "word mismatch for key {key}");
        }
    }
}

#[test]
fn display_escapes_invalid_utf8() {
    // `Display` must never lossily fold non-UTF-8 bytes to `U+FFFD`: distinct byte
    // sequences would then render identically. Invalid bytes are escaped as `\xHH`.
    let w = Word::new(&[0xFF, 0xFE]);
    let formatted = format!("{w}");
    assert_eq!(formatted, "\\xFF\\xFE");
    assert!(!formatted.contains('\u{FFFD}'));
}

#[test]
fn display_keeps_valid_utf8_verbatim_around_escaped_bytes() {
    let w = Word::new(b"Caf\xC9\xE9");
    assert_eq!(format!("{w}"), "Caf\\xC9\\xE9");
}

#[test]
fn debug_renders_quoted_string_for_valid_utf8() {
    let w = Word::new(b"hello");
    let formatted = format!("{w:?}");
    assert_eq!(formatted, "\"hello\"");
}
