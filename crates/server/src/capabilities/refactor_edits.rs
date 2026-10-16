//! Apply planned edits before a refactor reparses the changed source.

use crate::TextReplacement;

/// Apply edits in their given order, preserving untouched bytes.
/// Adjacent edits and repeated insertions at one offset keep that order.
pub(super) fn apply_ordered_edits(source: &str, edits: &[TextReplacement]) -> Option<String> {
    let mut changed = String::with_capacity(source.len() + edits.iter().map(|edit| edit.new_text.len()).sum::<usize>());
    let mut cursor = 0;
    for edit in edits {
        let start = edit.range.start as usize;
        let end = edit.range.end as usize;
        source.get(start..end)?;
        changed.push_str(source.get(cursor..start)?);
        changed.push_str(&edit.new_text);
        cursor = end;
    }
    changed.push_str(source.get(cursor..)?);
    Some(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Range;

    fn edit(start: u32, end: u32, text: &str) -> TextReplacement {
        TextReplacement { range: Range::new(start, end), new_text: text.into() }
    }

    #[test]
    fn adjacent_edits_and_insertions_keep_order_and_untouched_bytes() {
        let source = "é\r\nabc🙂";
        let edits = [
            edit(4, 4, "first"),
            edit(4, 4, "second"),
            edit(4, 5, "A"),
            edit(5, 6, "B"),
            edit(6, 6, "!"),
            edit(11, 11, "\nend"),
        ];
        assert_eq!(apply_ordered_edits(source, &edits).as_deref(), Some("é\r\nfirstsecondAB!c🙂\nend"));
        assert_eq!(apply_ordered_edits(source, &[]).as_deref(), Some(source));
    }

    #[test]
    fn overlapping_or_unordered_edits_are_rejected() {
        for edits in
            [[edit(0, 2, "x"), edit(1, 3, "y")], [edit(1, 3, "x"), edit(2, 2, "y")], [edit(2, 2, "x"), edit(1, 1, "y")]]
        {
            assert!(apply_ordered_edits("abcd", &edits).is_none(), "{edits:?}");
        }
    }

    #[test]
    fn invalid_ranges_and_split_utf8_characters_are_rejected() {
        for (start, end) in [(0, 1), (1, 2), (2, 3), (3, 3), (0, 7), (7, 7), (4, 2)] {
            assert!(apply_ordered_edits("é🙂", &[edit(start, end, "x")]).is_none(), "{start}..{end}");
        }
        assert_eq!(apply_ordered_edits("é🙂", &[edit(0, 2, "界")]).as_deref(), Some("界🙂"));
    }
}
