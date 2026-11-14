use mago_formatter_core::utils::column_after;

#[test]
fn tabs_advance_to_tab_stops() {
    for (text, start, width, end) in [
        (b"x\t".as_slice(), 0, 4, 4),
        (b"\t", 1, 4, 4),
        (b" \t", 0, 4, 4),
        (b"\t\t", 1, 4, 8),
        (b"a\tb", 7, 0, 9),
        (b"\xe7\x95\x8c\t", 0, 4, 4),
        (b"\xc3\xa9\t", 0, 4, 4),
    ] {
        assert_eq!(column_after(text, start, width), end);
    }
}

#[test]
fn newlines_reset_columns() {
    for (text, start, end) in
        [(b"abc\nx\t".as_slice(), 20, 4), (b"\r\nx", 20, 1), (b"\n", 20, 0), (b"abc", 7, 10), (b"", 7, 7)]
    {
        assert_eq!(column_after(text, start, 4), end);
    }
}
