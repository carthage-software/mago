use mago_atom::Atom;
use mago_atom::atom;
use mago_span::Span;

pub mod assertion;
pub mod consts;
pub mod context;
pub mod diff;
pub mod flags;
pub mod identifier;
pub mod issue;
pub mod metadata;
pub mod misc;
pub mod populator;
pub mod reference;
pub mod scanner;
pub mod symbol;
pub mod ttype;
pub mod visibility;

mod utils;

pub fn get_anonymous_class_name(span: Span) -> Atom {
    use std::io::Write;

    // A 64-byte buffer on the stack. This is ample space for the prefix,
    // u64 file id, and 2 u32 integers, preventing any chance of a heap allocation.
    let mut buffer = [0u8; 64];

    // Use a block to limit the scope of the mutable writer
    // `writer` is a mutable slice that implements `std::io::Write`.
    let mut writer = &mut buffer[..];

    // SAFETY: We use `unwrap_unchecked` here because we are writing to a fixed-size buffer
    unsafe {
        write!(writer, "class@anonymous:{}-{}:{}", span.file_id, span.start.offset, span.end.offset).unwrap_unchecked()
    };

    // Determine how many bytes were written by checking the length of the original buffer
    // against what the `writer` had left. This is a common pattern for `io::Write` on slices.
    let written_len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

    atom(
        // SAFETY: We use `unwrap_unchecked` here because we are certain the bytes
        // up to `written_len` are valid UTF-8.
        unsafe { std::str::from_utf8(&buffer[..written_len]).unwrap_unchecked() },
    )
}
