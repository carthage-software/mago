/// First-byte classification used by the `lex_expression` fast dispatch.
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ByteClass {
    Other = 0,
    Whitespace,
    Digit,
    IdentifierStart,
    SingleQuote,
    DoubleQuote,
    Hash,
}

/// 256-entry table mapping a byte to its [`ByteClass`].
pub(crate) const BYTE_CLASS: [ByteClass; 256] = {
    let mut table = [ByteClass::Other; 256];
    let mut i = 0usize;
    while i < 256 {
        let b = i as u8;
        table[i] = match b {
            b' ' | b'\t' | b'\r' | b'\n' | 0x0B | 0x0C => ByteClass::Whitespace,
            b'0'..=b'9' => ByteClass::Digit,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => ByteClass::IdentifierStart,
            0x80..=0xFF => ByteClass::IdentifierStart,
            b'\'' => ByteClass::SingleQuote,
            b'"' => ByteClass::DoubleQuote,
            b'#' => ByteClass::Hash,
            _ => ByteClass::Other,
        };

        i += 1;
    }

    table
};

/// 256-entry table: `true` for bytes that may continue an identifier.
pub(crate) const IDENT_PART: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0usize;
    while i < 256 {
        let b = i as u8;
        table[i] = matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | 0x80..=0xFF);
        i += 1;
    }

    table
};
