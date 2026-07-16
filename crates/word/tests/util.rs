use mago_word::Word;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::ascii_lowercase_word;
use mago_word::concat_word;
use mago_word::concat_word2;
use mago_word::concat_word12;
use mago_word::empty_word;
use mago_word::f32_word;
use mago_word::f64_word;
use mago_word::i32_word;
use mago_word::i64_word;
use mago_word::join_words;
use mago_word::starts_with_ignore_case;
use mago_word::u32_word;
use mago_word::usize_word;

#[test]
fn empty_word_is_canonical() {
    let a = empty_word();
    let b = empty_word();
    let c = Word::new(b"");
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(a.len(), 0);
}

#[test]
fn ascii_lowercase_word_already_lowercase_passes_through() {
    let w = ascii_lowercase_word(b"already_lowercase");
    assert_eq!(w.as_bytes(), b"already_lowercase");
}

#[test]
fn ascii_lowercase_word_with_uppercase_lowers() {
    let w = ascii_lowercase_word(b"HelloWorld");
    assert_eq!(w.as_bytes(), b"helloworld");
}

#[test]
fn ascii_lowercase_word_preserves_non_ascii_bytes() {
    let input: &[u8] = &[b'F', b'O', b'O', 0xFF, 0xC3, 0x28];
    let w = ascii_lowercase_word(input);
    assert_eq!(w.as_bytes(), &[b'f', b'o', b'o', 0xFF, 0xC3, 0x28]);
}

#[test]
fn ascii_lowercase_word_handles_strings_longer_than_stack_buffer() {
    let input: Vec<u8> = (0..400).map(|i| if i % 2 == 0 { b'A' } else { b'b' }).collect();
    let w = ascii_lowercase_word(&input);
    assert_eq!(w.len(), 400);
    assert!(w.as_bytes().iter().all(|&b| !b.is_ascii_uppercase()));
}

#[test]
fn ascii_lowercase_constant_name_word_lowers_namespace_only() {
    let w = ascii_lowercase_constant_name_word(b"Foo\\Bar\\MY_CONST");
    assert_eq!(w.as_bytes(), b"foo\\bar\\MY_CONST");
}

#[test]
fn ascii_lowercase_constant_name_word_without_backslash_lowers_full() {
    let w = ascii_lowercase_constant_name_word(b"MY_CONST");
    assert_eq!(w.as_bytes(), b"MY_CONST");
}

#[test]
fn ascii_lowercase_constant_name_word_handles_long_input() {
    let mut input = b"Foo\\".to_vec();
    input.extend(std::iter::repeat_n(b'X', 400));
    let w = ascii_lowercase_constant_name_word(&input);
    let bytes = w.as_bytes();
    assert!(bytes.starts_with(b"foo\\"));
    assert_eq!(&bytes[4..], &input[4..]);
}

#[test]
fn starts_with_ignore_case_short() {
    assert!(starts_with_ignore_case(b"HelloWorld", b"hello"));
    assert!(starts_with_ignore_case(b"FOOBAR", b"FooBar"));
    assert!(!starts_with_ignore_case(b"hello", b"world"));
    assert!(!starts_with_ignore_case(b"hi", b"hello"));
}

#[test]
fn starts_with_ignore_case_long_triggers_simd_path() {
    let haystack = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz_trailing";
    let prefix = b"abcdefghijklmnopqrstuvwxyz_ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    assert!(starts_with_ignore_case(haystack, prefix));
}

#[test]
fn starts_with_ignore_case_mismatch_in_simd_chunk() {
    let haystack = b"ABCDEFGHIJKLMNOP_DIFFERENT_TAIL_HERE";
    let prefix = b"abcdefghijklmnopqrstuvwxyz_ABCDEFG";
    assert!(!starts_with_ignore_case(haystack, prefix));
}

#[test]
fn integer_word_constructors_format_correctly() {
    assert_eq!(i32_word(42).as_bytes(), b"42");
    assert_eq!(i32_word(-7).as_bytes(), b"-7");
    assert_eq!(u32_word(0).as_bytes(), b"0");
    assert_eq!(i64_word(i64::MIN).as_bytes(), b"-9223372036854775808");
    assert_eq!(usize_word(100).as_bytes(), b"100");
}

#[test]
fn float_word_constructors_format_correctly() {
    assert_eq!(f32_word(1.5).as_bytes(), b"1.5");
    assert_eq!(f64_word(0.25).as_bytes(), b"0.25");
}

#[test]
fn concat_word2_concatenates_short_bytes() {
    let w = concat_word2(b"hello", b" world");
    assert_eq!(w.as_bytes(), b"hello world");
}

#[test]
fn concat_word12_concatenates_twelve_pieces() {
    let w = concat_word12(b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8", b"9", b"10", b"11", b"12");
    assert_eq!(w.as_bytes(), b"123456789101112");
}

#[test]
fn concat_word_macro_accepts_str_and_bytes() {
    let word_value = Word::new(b"suffix");
    let w = concat_word!("prefix_", b"middle_", word_value);
    assert_eq!(w.as_bytes(), b"prefix_middle_suffix");
}

#[test]
fn concat_word_macro_handles_long_inputs() {
    let long: Vec<u8> = std::iter::repeat_n(b'a', 200).collect();
    let w = concat_word!(b"start_", long.as_slice(), b"_end");
    assert_eq!(w.len(), 6 + 200 + 4);
    assert!(w.as_bytes().starts_with(b"start_"));
    assert!(w.as_bytes().ends_with(b"_end"));
}

#[test]
fn join_words_handles_empty_and_single_inputs() {
    assert_eq!(join_words(&[], b"|").as_bytes(), b"");

    let only = Word::new(b"only");
    assert_eq!(join_words(&[only], b"|").as_bytes(), b"only");
}

#[test]
fn join_words_joins_short_inputs() {
    let words = [Word::new(b"alpha"), Word::new(b"beta"), Word::new(b"gamma")];
    assert_eq!(join_words(&words, b"|").as_bytes(), b"alpha|beta|gamma");
}

#[test]
fn join_words_joins_inputs_larger_than_stack_buffer() {
    let words = [Word::new(&[b'a'; 200]), Word::new(&[b'b'; 200])];
    let joined = join_words(&words, b"::");

    let mut expected = vec![b'a'; 200];
    expected.extend_from_slice(b"::");
    expected.extend_from_slice(&[b'b'; 200]);
    assert_eq!(joined.as_bytes(), expected);
}
