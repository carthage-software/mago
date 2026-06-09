use std::borrow::Cow;

#[derive(Clone, Copy)]
pub(crate) enum LetterCase {
    Lower,
    Upper,
}

#[derive(Clone, Copy)]
pub(crate) struct CamelLikeOptions {
    pub new_word: bool,
    pub first_word: bool,
    pub separator: u8,
    pub has_separator: bool,
    pub inverted: bool,
    pub concat_num: bool,
}

#[inline(always)]
pub(crate) const fn is_ascii_lowercase(byte: u8) -> bool {
    byte.wrapping_sub(b'a') <= (b'z' - b'a')
}

#[inline(always)]
pub(crate) const fn is_ascii_uppercase(byte: u8) -> bool {
    byte.wrapping_sub(b'A') <= (b'Z' - b'A')
}

#[inline(always)]
pub(crate) const fn is_ascii_digit(byte: u8) -> bool {
    byte.wrapping_sub(b'0') <= (b'9' - b'0')
}

#[inline(always)]
pub(crate) const fn is_ascii_alphanumeric(byte: u8) -> bool {
    is_ascii_lowercase(byte) || is_ascii_uppercase(byte) || is_ascii_digit(byte)
}

#[inline(always)]
pub(crate) const fn to_ascii_lowercase(byte: u8) -> u8 {
    if is_ascii_uppercase(byte) { byte + 32 } else { byte }
}

#[inline(always)]
pub(crate) const fn to_ascii_uppercase(byte: u8) -> u8 {
    if is_ascii_lowercase(byte) { byte - 32 } else { byte }
}

#[inline(always)]
fn is_separator(byte: u8) -> bool {
    !is_ascii_alphanumeric(byte)
}

#[inline]
fn trim_right(input: &[u8]) -> &[u8] {
    let mut end = input.len();

    while end > 0 && !is_ascii_alphanumeric(input[end - 1]) {
        end -= 1;
    }

    &input[..end]
}

#[inline]
fn emit_checked(input: &[u8], output_index: &mut usize, emitted: u8) -> bool {
    if *output_index >= input.len() || input[*output_index] != emitted {
        return false;
    }

    *output_index += 1;
    true
}

#[inline]
pub(crate) fn is_case_camel_like(input: &[u8], options: CamelLikeOptions) -> bool {
    let input_trimmed = trim_right(input);
    let mut output_index = 0;

    let mut new_word = options.new_word;
    let mut first_word = options.first_word;
    let mut last_char = b' ';
    let mut found_real_char = false;

    let mut index = 0;
    while index < input_trimmed.len() {
        let byte = input_trimmed[index];

        if is_separator(byte) && found_real_char {
            new_word = true;
            index += 1;
            continue;
        }

        if !found_real_char && is_separator(byte) {
            index += 1;
            continue;
        }

        if is_ascii_digit(byte) && options.concat_num {
            found_real_char = true;
            new_word = true;

            if !emit_checked(input, &mut output_index, byte) {
                return false;
            }

            index += 1;
            continue;
        }

        if new_word || ((is_ascii_lowercase(last_char) && is_ascii_uppercase(byte)) && last_char != b' ') {
            found_real_char = true;
            new_word = false;

            if options.has_separator && !first_word && !emit_checked(input, &mut output_index, options.separator) {
                return false;
            }

            let emitted =
                if !options.inverted || first_word { to_ascii_uppercase(byte) } else { to_ascii_lowercase(byte) };

            if !emit_checked(input, &mut output_index, emitted) {
                return false;
            }

            first_word = false;
        } else {
            found_real_char = true;
            last_char = byte;

            if !emit_checked(input, &mut output_index, to_ascii_lowercase(byte)) {
                return false;
            }
        }

        index += 1;
    }

    output_index == input.len()
}

#[inline]
fn convert_camel_like(input: &[u8], options: CamelLikeOptions) -> Vec<u8> {
    let input = trim_right(input);
    let mut output = Vec::with_capacity(input.len() * 2);

    let mut new_word = options.new_word;
    let mut first_word = options.first_word;
    let mut last_char = b' ';
    let mut found_real_char = false;

    let mut index = 0;
    while index < input.len() {
        let byte = input[index];

        if is_separator(byte) && found_real_char {
            new_word = true;
            index += 1;
            continue;
        }

        if !found_real_char && is_separator(byte) {
            index += 1;
            continue;
        }

        if is_ascii_digit(byte) && options.concat_num {
            found_real_char = true;
            new_word = true;
            output.push(byte);
            index += 1;
            continue;
        }

        if new_word || ((is_ascii_lowercase(last_char) && is_ascii_uppercase(byte)) && last_char != b' ') {
            found_real_char = true;
            new_word = false;

            if options.has_separator && !first_word {
                output.push(options.separator);
            }

            if !options.inverted || first_word {
                output.push(to_ascii_uppercase(byte));
            } else {
                output.push(to_ascii_lowercase(byte));
            }

            first_word = false;
        } else {
            found_real_char = true;
            last_char = byte;
            output.push(to_ascii_lowercase(byte));
        }

        index += 1;
    }

    output
}

#[inline]
pub(crate) fn to_case_camel_like(input: &[u8], options: CamelLikeOptions) -> Cow<'_, [u8]> {
    if is_case_camel_like(input, options) {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(convert_camel_like(input, options))
    }
}

#[inline(always)]
fn is_upper_like_for_boundary(byte: u8, digit_is_uppercase_boundary: bool) -> bool {
    if digit_is_uppercase_boundary { byte == to_ascii_uppercase(byte) } else { is_ascii_uppercase(byte) }
}

#[inline(always)]
fn has_lowercase_neighbor(input: &[u8], index: usize) -> bool {
    (index + 1 < input.len() && is_ascii_lowercase(input[index + 1]))
        || (index > 0 && is_ascii_lowercase(input[index - 1]))
}

#[inline]
pub(crate) fn is_case_snake_like(
    input: &[u8],
    separator: u8,
    case: LetterCase,
    digit_is_uppercase_boundary: bool,
) -> bool {
    let input_trimmed = trim_right(input);
    let mut output_index = 0;
    let mut first_character = true;

    let mut index = 0;
    while index < input_trimmed.len() {
        let byte = input_trimmed[index];

        if is_separator(byte) {
            if !first_character {
                first_character = true;
                if !emit_checked(input, &mut output_index, separator) {
                    return false;
                }
            }

            index += 1;
            continue;
        }

        if !first_character
            && is_upper_like_for_boundary(byte, digit_is_uppercase_boundary)
            && has_lowercase_neighbor(input_trimmed, index)
        {
            if !emit_checked(input, &mut output_index, separator) {
                return false;
            }

            let emitted = match case {
                LetterCase::Lower => to_ascii_lowercase(byte),
                LetterCase::Upper => to_ascii_uppercase(byte),
            };

            if !emit_checked(input, &mut output_index, emitted) {
                return false;
            }

            first_character = false;
            index += 1;
            continue;
        }

        let emitted = match case {
            LetterCase::Lower => to_ascii_lowercase(byte),
            LetterCase::Upper => to_ascii_uppercase(byte),
        };

        if !emit_checked(input, &mut output_index, emitted) {
            return false;
        }

        first_character = false;
        index += 1;
    }

    output_index == input.len()
}

#[inline]
fn convert_snake_like(input: &[u8], separator: u8, case: LetterCase, digit_is_uppercase_boundary: bool) -> Vec<u8> {
    let input = trim_right(input);
    let mut output = Vec::with_capacity(input.len() * 2);
    let mut first_character = true;

    let mut index = 0;
    while index < input.len() {
        let byte = input[index];

        if is_separator(byte) {
            if !first_character {
                first_character = true;
                output.push(separator);
            }

            index += 1;
            continue;
        }

        if !first_character
            && is_upper_like_for_boundary(byte, digit_is_uppercase_boundary)
            && has_lowercase_neighbor(input, index)
        {
            output.push(separator);
            output.push(match case {
                LetterCase::Lower => to_ascii_lowercase(byte),
                LetterCase::Upper => to_ascii_uppercase(byte),
            });

            first_character = false;
            index += 1;
            continue;
        }

        output.push(match case {
            LetterCase::Lower => to_ascii_lowercase(byte),
            LetterCase::Upper => to_ascii_uppercase(byte),
        });

        first_character = false;
        index += 1;
    }

    output
}

#[inline]
pub(crate) fn to_case_snake_like(
    input: &[u8],
    separator: u8,
    case: LetterCase,
    digit_is_uppercase_boundary: bool,
) -> Cow<'_, [u8]> {
    if is_case_snake_like(input, separator, case, digit_is_uppercase_boundary) {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(convert_snake_like(input, separator, case, digit_is_uppercase_boundary))
    }
}
