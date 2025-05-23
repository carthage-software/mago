use memchr::memchr;
use memchr::memmem::find;

use mago_source::SourceIdentifier;
use mago_span::Position;

/// A struct representing the input code being lexed.
///
/// The `Input` struct provides methods to read, peek, consume, and skip characters
/// from the bytes input code while keeping track of the current position (line, column, offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Input<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) length: usize,
    pub(crate) offset: usize,
    pub(crate) starting_position: Position,
}

impl<'a> Input<'a> {
    /// Creates a new `Input` instance from the given input.
    ///
    /// # Arguments
    ///
    /// * `input` - A byte slice representing the input code to be processed.
    ///
    /// # Returns
    ///
    /// A new `Input` instance initialized at the beginning of the input.
    pub fn new(source: SourceIdentifier, bytes: &'a [u8]) -> Self {
        let length = bytes.len();

        Self { bytes, length, offset: 0, starting_position: Position::start_of(source) }
    }

    /// Creates a new `Input` instance representing a byte slice that is
    /// "anchored" at a specific absolute position within a larger source file.
    ///
    /// This is useful when lexing a subset (slice) of a source file, as it allows
    /// generated tokens to retain accurate absolute positions and spans relative
    /// to the original file.
    ///
    /// The internal cursor (`offset`) starts at 0 relative to the `bytes` slice,
    /// but the absolute position is calculated relative to the `anchor_position`.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A byte slice representing the input code subset to be lexed.
    /// * `anchor_position` - The absolute `Position` in the original source file where
    ///   the provided `bytes` slice begins.
    ///
    /// # Returns
    ///
    /// A new `Input` instance ready to lex the `bytes`, maintaining positions
    /// relative to `anchor_position`.
    pub fn anchored_at(bytes: &'a [u8], anchor_position: Position) -> Self {
        let length = bytes.len();

        Self { bytes, length, offset: 0, starting_position: anchor_position }
    }

    /// Returns the source identifier of the input code.
    #[inline]
    pub const fn source_identifier(&self) -> SourceIdentifier {
        self.starting_position.source
    }

    /// Returns the absolute current `Position` of the lexer within the original source file.
    ///
    /// It calculates this by adding the internal offset (progress within the current byte slice)
    /// to the `starting_position` the `Input` was initialized with.
    #[inline]
    pub const fn current_position(&self) -> Position {
        // Calculate absolute position by adding internal offset to the starting base
        self.starting_position.forward(self.offset)
    }

    /// Returns the current internal byte offset relative to the start of the input slice.
    ///
    /// This indicates how many bytes have been consumed from the current `bytes` slice.
    /// To get the absolute position in the original source file, use `current_position()`.
    #[inline]
    pub const fn current_offset(&self) -> usize {
        self.offset
    }

    /// Returns `true` if the input slice is empty (length is zero).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns the total length in bytes of the input slice being processed.
    #[inline]
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Checks if the current position is at the end of the input.
    ///
    /// # Returns
    ///
    /// `true` if the current offset is greater than or equal to the input length; `false` otherwise.
    #[inline]
    pub const fn has_reached_eof(&self) -> bool {
        self.offset >= self.length
    }

    /// Advances the current position by one character, updating line and column numbers.
    ///
    /// Handles different line endings (`\n`, `\r`, `\r\n`) and updates line and column counters accordingly.
    ///
    /// If the end of input is reached, no action is taken.
    #[inline]
    pub fn next(&mut self) {
        if !self.has_reached_eof() {
            self.offset += 1;
        }
    }

    /// Skips the next `count` characters, advancing the position accordingly.
    ///
    /// Updates line and column numbers as it advances.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of characters to skip.
    #[inline]
    pub fn skip(&mut self, count: usize) {
        self.offset = (self.offset + count).min(self.length);
    }

    /// Consumes the next `count` characters and returns them as a slice.
    ///
    /// Advances the position by `count` characters.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of characters to consume.
    ///
    /// # Returns
    ///
    /// A byte slice containing the consumed characters.
    #[inline]
    pub fn consume(&mut self, count: usize) -> &'a [u8] {
        let (from, until) = self.calculate_bound(count);

        self.skip(count);

        &self.bytes[from..until]
    }

    /// Consumes all remaining characters from the current position to the end of input.
    ///
    /// Advances the position to EOF.
    ///
    /// # Returns
    ///
    /// A byte slice containing the remaining characters.
    #[inline]
    pub fn consume_remaining(&mut self) -> &'a [u8] {
        if self.has_reached_eof() {
            return &[];
        }

        let from = self.offset;
        self.offset = self.length;

        &self.bytes[from..]
    }

    /// Consumes characters until the given byte slice is found.
    ///
    /// Advances the position to the start of the search slice if found,
    /// or to EOF if not found.
    ///
    /// # Arguments
    ///
    /// * `search` - The byte slice to search for.
    /// * `ignore_ascii_case` - Whether to ignore ASCII case when comparing characters.
    ///
    /// # Returns
    ///
    /// A byte slice containing the consumed characters.
    #[inline]
    pub fn consume_until(&mut self, search: &[u8], ignore_ascii_case: bool) -> &'a [u8] {
        let start = self.offset;
        if !ignore_ascii_case {
            // For a single-byte search, use memchr.
            if search.len() == 1 {
                if let Some(pos) = memchr(search[0], &self.bytes[self.offset..]) {
                    self.offset += pos;
                    &self.bytes[start..self.offset]
                } else {
                    self.offset = self.length;
                    &self.bytes[start..self.length]
                }
            } else if let Some(pos) = find(&self.bytes[self.offset..], search) {
                self.offset += pos;
                &self.bytes[start..self.offset]
            } else {
                self.offset = self.length;
                &self.bytes[start..self.length]
            }
        } else {
            while !self.has_reached_eof() && !self.is_at(search, ignore_ascii_case) {
                self.offset += 1;
            }

            &self.bytes[start..self.offset]
        }
    }

    #[inline]
    pub fn consume_through(&mut self, search: u8) -> &'a [u8] {
        let start = self.offset;
        if let Some(pos) = memchr::memchr(search, &self.bytes[self.offset..]) {
            self.offset += pos + 1;

            &self.bytes[start..self.offset]
        } else {
            self.offset = self.length;

            &self.bytes[start..self.length]
        }
    }

    /// Consumes whitespaces until a non-whitespace character is found.
    ///
    /// # Returns
    ///
    /// A byte slice containing the consumed whitespaces.
    #[inline]
    pub fn consume_whitespaces(&mut self) -> &'a [u8] {
        let start = self.offset;
        let bytes = self.bytes;
        let len = self.length;
        while self.offset < len && bytes[self.offset].is_ascii_whitespace() {
            self.offset += 1;
        }

        &bytes[start..self.offset]
    }

    /// Reads the next `n` characters without advancing the position.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of characters to read.
    ///
    /// # Returns
    ///
    /// A byte slice containing the next `n` characters.
    #[inline]
    pub fn read(&self, n: usize) -> &'a [u8] {
        let (from, until) = self.calculate_bound(n);

        &self.bytes[from..until]
    }

    /// Reads a single byte at a specific byte offset within the input slice,
    /// without advancing the internal cursor.
    ///
    /// This provides direct, low-level access to the underlying byte data.
    ///
    /// # Arguments
    ///
    /// * `at` - The zero-based byte offset within the input slice (`self.bytes`)
    ///   from which to read the byte.
    ///
    /// # Returns
    ///
    /// A reference to the byte located at the specified offset `at`.
    ///
    /// # Panics
    ///
    /// This method **panics** if the provided `at` offset is out of bounds
    /// for the input byte slice (i.e., if `at >= self.bytes.len()`).
    pub fn read_at(&self, at: usize) -> &'a u8 {
        &self.bytes[at]
    }

    /// Checks if the input at the current position matches the given byte slice.
    ///
    /// # Arguments
    ///
    /// * `search` - The byte slice to compare against the input.
    /// * `ignore_ascii_case` - Whether to ignore ASCII case when comparing.
    ///
    /// # Returns
    ///
    /// `true` if the next bytes match `search`; `false` otherwise.
    #[inline]
    pub fn is_at(&self, search: &[u8], ignore_ascii_case: bool) -> bool {
        let (from, until) = self.calculate_bound(search.len());
        let slice = &self.bytes[from..until];

        if ignore_ascii_case { slice.eq_ignore_ascii_case(search) } else { slice == search }
    }

    /// Attempts to match the given byte sequence at the current position, ignoring whitespace in the input.
    ///
    /// This method tries to match the provided byte slice `search` against the input starting
    /// from the current position, possibly ignoring ASCII case. Whitespace characters in the input
    /// are skipped during matching, but their length is included in the returned length.
    ///
    /// Importantly, the method **does not include** any trailing whitespace **after** the matched sequence
    /// in the returned length.
    ///
    /// For example, to match the sequence `(string)`, the input could be `(string)`, `( string )`, `(  string )`, etc.,
    /// and this method would return the total length of the input consumed to match `(string)`,
    /// including any whitespace within the matched sequence, but **excluding** any whitespace after it.
    ///
    /// # Arguments
    ///
    /// * `search` - The byte slice to match against the input.
    /// * `ignore_ascii_case` - If `true`, ASCII case is ignored during comparison.
    ///
    /// # Returns
    ///
    /// * `Some(length)` - If the input matches `search` (ignoring whitespace within the sequence), returns the total length
    ///   of the input consumed to match `search`, including any skipped whitespace **within** the matched sequence.
    /// * `None` - If the input does not match `search`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mago_syntax_core::input::Input;
    /// use mago_source::SourceIdentifier;
    ///
    /// let source = SourceIdentifier::dummy();
    ///
    /// // Given input "( string ) x", starting at offset 0:
    /// let input = Input::new(source.clone(), b"( string ) x");
    /// assert_eq!(input.match_sequence_ignore_whitespace(b"(string)", true), Some(10)); // 10 bytes consumed up to ')'
    ///
    /// // Given input "(int)", with no whitespace:
    /// let input = Input::new(source.clone(), b"(int)");
    /// assert_eq!(input.match_sequence_ignore_whitespace(b"(int)", true), Some(5)); // 5 bytes consumed
    ///
    /// // Given input "(  InT   )abc", ignoring ASCII case:
    /// let input = Input::new(source.clone(), b"(  InT   )abc");
    /// assert_eq!(input.match_sequence_ignore_whitespace(b"(int)", true), Some(10)); // 10 bytes consumed up to ')'
    ///
    /// // Given input "(integer)", attempting to match "(int)":
    /// let input = Input::new(source.clone(), b"(integer)");
    /// assert_eq!(input.match_sequence_ignore_whitespace(b"(int)", false), None); // Does not match
    ///
    /// // Trailing whitespace after ')':
    /// let input = Input::new(source.clone(), b"(int)   x");
    /// assert_eq!(input.match_sequence_ignore_whitespace(b"(int)", true), Some(5)); // Length up to ')', excludes spaces after ')'
    /// ```
    #[inline]
    pub const fn match_sequence_ignore_whitespace(&self, search: &[u8], ignore_ascii_case: bool) -> Option<usize> {
        let mut offset = self.offset;
        let mut search_offset = 0;
        let mut length = 0;
        let bytes = self.bytes;
        let total = self.length;
        while search_offset < search.len() {
            // Skip whitespace in the input.
            while offset < total && bytes[offset].is_ascii_whitespace() {
                offset += 1;
                length += 1;
            }

            if offset >= total {
                return None;
            }

            let input_byte = bytes[offset];
            let search_byte = search[search_offset];
            let matched = if ignore_ascii_case {
                input_byte.eq_ignore_ascii_case(&search_byte)
            } else {
                input_byte == search_byte
            };

            if matched {
                offset += 1;
                length += 1;
                search_offset += 1;
            } else {
                return None;
            }
        }

        Some(length)
    }

    /// Peeks ahead `i` characters and reads the next `n` characters without advancing the position.
    ///
    /// # Arguments
    ///
    /// * `offset` - The number of characters to skip before reading.
    /// * `n` - The number of characters to read after skipping.
    ///
    /// # Returns
    ///
    /// A byte slice containing the peeked characters.
    #[inline]
    pub fn peek(&self, offset: usize, n: usize) -> &'a [u8] {
        let from = self.offset + offset;
        if from >= self.length {
            return &self.bytes[self.length..self.length];
        }

        let mut until = from + n;
        if until >= self.length {
            until = self.length;
        }

        &self.bytes[from..until]
    }

    /// Calculates the bounds for slicing the input safely.
    ///
    /// Ensures that slicing does not go beyond the input length.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of characters to include in the slice.
    ///
    /// # Returns
    ///
    /// A tuple `(from, until)` representing the start and end indices for slicing.
    #[inline]
    const fn calculate_bound(&self, n: usize) -> (usize, usize) {
        if self.has_reached_eof() {
            return (self.length, self.length);
        }

        let mut until = self.offset + n;

        if until >= self.length {
            until = self.length;
        }

        (self.offset, until)
    }
}

#[cfg(test)]
mod tests {
    use mago_span::Position;

    use super::*;

    #[test]
    fn test_new() {
        let bytes = b"Hello, world!";
        let input = Input::new(SourceIdentifier::dummy(), bytes);

        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 0));
        assert_eq!(input.length, bytes.len());
        assert_eq!(input.bytes, bytes);
    }

    #[test]
    fn test_is_eof() {
        let bytes = b"";
        let input = Input::new(SourceIdentifier::dummy(), bytes);

        assert!(input.has_reached_eof());

        let bytes = b"data";
        let mut input = Input::new(SourceIdentifier::dummy(), bytes);

        assert!(!input.has_reached_eof());

        input.skip(4);

        assert!(input.has_reached_eof());
    }

    #[test]
    fn test_next() {
        let bytes = b"a\nb\r\nc\rd";
        let mut input = Input::new(SourceIdentifier::dummy(), bytes);

        // 'a'
        input.next();
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 1));

        // '\n'
        input.next();
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 2));

        // 'b'
        input.next();
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 3));

        // '\r\n' should be treated as one newline
        input.next();
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 4));

        // 'c'
        input.next();
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 5));

        // '\r'
        input.next();
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 6));

        // 'd'
        input.next();
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 7));
    }

    #[test]
    fn test_consume() {
        let bytes = b"abcdef";
        let mut input = Input::new(SourceIdentifier::dummy(), bytes);

        let consumed = input.consume(3);
        assert_eq!(consumed, b"abc");
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 3));

        let consumed = input.consume(3);
        assert_eq!(consumed, b"def");
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 6));

        let consumed = input.consume(1); // Should return empty slice at EOF
        assert_eq!(consumed, b"");
        assert!(input.has_reached_eof());
    }

    #[test]
    fn test_consume_remaining() {
        let bytes = b"abcdef";
        let mut input = Input::new(SourceIdentifier::dummy(), bytes);

        input.skip(2);
        let remaining = input.consume_remaining();
        assert_eq!(remaining, b"cdef");
        assert!(input.has_reached_eof());
    }

    #[test]
    fn test_read() {
        let bytes = b"abcdef";
        let input = Input::new(SourceIdentifier::dummy(), bytes);

        let read = input.read(3);
        assert_eq!(read, b"abc");
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 0));
        // Position should not change
    }

    #[test]
    fn test_is_at() {
        let bytes = b"abcdef";
        let mut input = Input::new(SourceIdentifier::dummy(), bytes);

        assert!(input.is_at(b"abc", false));
        input.skip(2);
        assert!(input.is_at(b"cde", false));
        assert!(!input.is_at(b"xyz", false));
    }

    #[test]
    fn test_is_at_ignore_ascii_case() {
        let bytes = b"AbCdEf";
        let mut input = Input::new(SourceIdentifier::dummy(), bytes);

        assert!(input.is_at(b"abc", true));
        input.skip(2);
        assert!(input.is_at(b"cde", true));
        assert!(!input.is_at(b"xyz", true));
    }

    #[test]
    fn test_peek() {
        let bytes = b"abcdef";
        let input = Input::new(SourceIdentifier::dummy(), bytes);

        let peeked = input.peek(2, 3);
        assert_eq!(peeked, b"cde");
        assert_eq!(input.current_position(), Position::new(SourceIdentifier::dummy(), 0));
        // Position should not change
    }

    #[test]
    fn test_to_bound() {
        let bytes = b"abcdef";
        let input = Input::new(SourceIdentifier::dummy(), bytes);

        let (from, until) = input.calculate_bound(3);
        assert_eq!((from, until), (0, 3));

        let (from, until) = input.calculate_bound(10); // Exceeds length
        assert_eq!((from, until), (0, 6));
    }
}
