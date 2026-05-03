use mago_span::HasSpan;

use crate::ast::Program;
use crate::ast::Trivia;
use crate::ast::TriviaKind;

/// An iterator that yields docblock trivia nodes preceding a position, walking
/// backwards through stacked docblocks using the same gap-checking logic as
/// `get_docblock_before_position`.
///
/// By default all docblocks are yielded. Call `important_only(patterns)` to
/// restrict the iterator to docblocks whose text contains at least one of the
/// given substrings, skipping non-matching entries rather than stopping.
pub struct PrecedingDocblocks<'arena, 'pat> {
    trivia: &'arena [Trivia<'arena>],
    start: u32,
    important_patterns: &'pat [&'pat str],
}

impl<'arena> PrecedingDocblocks<'arena, 'static> {
    #[must_use]
    pub fn new(trivia: &'arena [Trivia<'arena>], start_offset: u32) -> Self {
        Self { trivia, start: start_offset, important_patterns: &[] }
    }
}

impl<'arena> PrecedingDocblocks<'arena, '_> {
    /// Restrict this iterator to docblocks whose text contains at least one of
    /// `patterns`. Non-matching docblocks are skipped; the search continues
    /// backward past them.
    #[must_use]
    pub fn important_only<'new_pat>(self, patterns: &'new_pat [&'new_pat str]) -> PrecedingDocblocks<'arena, 'new_pat> {
        PrecedingDocblocks { trivia: self.trivia, start: self.start, important_patterns: patterns }
    }
}

impl<'arena> Iterator for PrecedingDocblocks<'arena, '_> {
    type Item = &'arena Trivia<'arena>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trivia = get_docblock_before_position(self.trivia, self.start)?;
            self.start = trivia.span.start_offset();
            if self.important_patterns.is_empty() || self.important_patterns.iter().any(|p| trivia.value.contains(*p)) {
                return Some(trivia);
            }
        }
    }
}

/// Retrieves the docblock comment associated with a given node in the program.
/// If the node is preceded by a docblock comment, it returns that comment.
///
/// This function searches for the last docblock comment that appears before the node's start position,
/// ensuring that it is directly preceding the node without any non-whitespace characters in between.
///
/// # Arguments
///
/// * `program` - The program containing the trivia.
/// * `node` - The node for which to find the preceding docblock comment.
///
/// # Returns
///
/// An `Option` containing a reference to the `Trivia` representing the docblock comment if found,
/// or `None` if no suitable docblock comment exists before the node.
#[inline]
#[must_use]
pub fn get_docblock_for_node<'arena>(
    program: &'arena Program<'arena>,
    node: impl HasSpan,
) -> Option<&'arena Trivia<'arena>> {
    get_docblock_before_position(program.trivia.as_slice(), node.span().start.offset)
}

/// Retrieves the docblock comment that appears before a specific position in the source code.
///
/// This function scans the trivia associated with the source code and returns the last docblock comment
/// that appears before the specified position, ensuring that it is directly preceding the node
/// without any non-whitespace characters in between.
///
/// # Arguments
///
/// * `trivias` - A slice of trivia associated with the source code.
/// * `node_start_offset` - The start offset of the node for which to find the preceding docblock comment.
///
/// # Returns
///
/// An `Option` containing a reference to the `Trivia` representing the docblock comment if found,
#[must_use]
pub fn get_docblock_before_position<'arena>(
    trivias: &'arena [Trivia<'arena>],
    node_start_offset: u32,
) -> Option<&'arena Trivia<'arena>> {
    let candidate_partition_idx = trivias.partition_point(|trivia| trivia.span.start.offset < node_start_offset);
    if candidate_partition_idx == 0 {
        return None;
    }

    // Track the earliest position we've "covered" by trivia.
    // Start from node_start_offset and work backwards.
    // As we iterate, we verify that each trivia connects to the next (no code gaps).
    // Since the parser captures all whitespace as WhiteSpace trivia, any gap not covered
    // by a trivia node is actual code, so we just check for contiguity.
    let mut covered_from = node_start_offset;

    for i in (0..candidate_partition_idx).rev() {
        let trivia = &trivias[i];
        let trivia_end = trivia.span.end_offset();

        if trivia_end != covered_from {
            // Gap between this trivia and our covered region contains code.
            return None;
        }

        match trivia.kind {
            TriviaKind::DocBlockComment => {
                // Found a docblock with no code between it and the node.
                return Some(trivia);
            }
            TriviaKind::WhiteSpace
            | TriviaKind::SingleLineComment
            | TriviaKind::MultiLineComment
            | TriviaKind::HashComment => {
                covered_from = trivia.span.start_offset();
            }
        }
    }

    // Iterated through all preceding trivia without finding a suitable docblock.
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bumpalo::Bump;
    use mago_database::file::FileId;
    use mago_span::HasSpan;

    use crate::parser::parse_file_content;

    use super::get_docblock_before_position;

    #[test]
    fn whitespace_between_docblock_and_class_is_trivia() {
        // The parser emits WhiteSpace trivia for all whitespace, so there is no
        // code gap between the docblock's end offset and the class's start offset.
        // This verifies the assumption that strict trivia contiguity == no code gap.
        let arena = Bump::new();
        let program = parse_file_content(&arena, FileId::zero(), "<?php\n\n/** @return int */\n\nclass Foo {}");
        // statements[0] is the <?php opening tag; statements[1] is the class.
        let class_start = program.statements.iter().nth(1).unwrap().span().start.offset;
        let docblock = get_docblock_before_position(program.trivia.as_slice(), class_start);
        assert!(docblock.is_some(), "expected docblock to be found across whitespace");
        assert!(docblock.unwrap().value.contains("@return int"));
    }

    #[test]
    fn code_between_docblock_and_function_blocks_attribution() {
        let arena = Bump::new();
        let program =
            parse_file_content(&arena, FileId::zero(), "<?php\n/** @return int */\necho 1;\nfunction foo() {}");
        // statements: [0]=OpeningTag, [1]=Echo, [2]=Function
        let func_start = program.statements.iter().nth(2).unwrap().span().start.offset;
        let docblock = get_docblock_before_position(program.trivia.as_slice(), func_start);
        assert!(docblock.is_none(), "expected no docblock when code intervenes");
    }
}
