#![cfg(feature = "serde")]
#![allow(clippy::unwrap_used)]

use bincode::config::standard;
use mago_word::Word;
use mago_word::table;
use serde::Serialize;
use serde::Serializer;

const LONG: &[u8] = b"a word longer than the inline capacity";

struct Bytes(&'static [u8]);

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0)
    }
}

#[test]
fn words_serialize_as_bytes_outside_a_table() {
    let word = Word::new(LONG);
    let bytes = bincode::serde::encode_to_vec(word, standard()).unwrap();

    assert_eq!(bytes, bincode::serde::encode_to_vec(Bytes(LONG), standard()).unwrap());
    assert_eq!(bincode::serde::decode_from_slice::<Word, _>(&bytes, standard()).unwrap().0, word);
}

#[test]
fn tables_store_each_word_once_and_decode_to_the_same_words() {
    let short = Word::new(b"short");
    let long = Word::new(LONG);
    let words = vec![short, long, long, long, short];
    let plain = bincode::serde::encode_to_vec(&words, standard()).unwrap();

    let (bytes, words_table) = table::encode(|| bincode::serde::encode_to_vec(&words, standard()).unwrap());
    assert_eq!(words_table, [long, short]);
    assert!(bytes.len() < plain.len());

    let decoded = table::decode(&words_table, || bincode::serde::decode_from_slice::<Vec<Word>, _>(&bytes, standard()));
    assert_eq!(decoded.unwrap().0, words);

    assert_eq!(bincode::serde::encode_to_vec(&words, standard()).unwrap(), plain);
    bincode::serde::decode_from_slice::<Vec<Word>, _>(&bytes, standard()).unwrap_err();
}

#[test]
fn indices_outside_the_table_fail_to_decode() {
    let (bytes, _) = table::encode(|| bincode::serde::encode_to_vec(Word::new(b"word"), standard()).unwrap());

    table::decode(&[], || bincode::serde::decode_from_slice::<Word, _>(&bytes, standard())).unwrap_err();
}
