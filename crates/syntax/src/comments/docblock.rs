use mago_database::file::File;
use mago_span::HasSpan;

use crate::ast::Program;
use crate::ast::Trivia;
use crate::ast::TriviaKind;

/// Retrieves the docblock comment associated with a given node in the program.
/// If the node is preceded by a docblock comment, it returns that comment.
///
/// This function searches for the last docblock comment that appears before the node's start position,
/// ensuring that it is directly preceding the node without any non-whitespace characters in between.
///
/// # Arguments
///
/// * `program` - The program containing the trivia.
/// * `file` - The file from which the trivia is derived.
/// * `node` - The node for which to find the preceding docblock comment.
///
/// # Returns
///
/// An `Option` containing a reference to the `Trivia` representing the docblock comment if found,
/// or `None` if no suitable docblock comment exists before the node.
#[inline]
pub fn get_docblock_for_node<'arena>(
    program: &'arena Program<'arena>,
    file: &File,
    node: impl HasSpan,
) -> Option<&'arena Trivia<'arena>> {
    get_docblock_before_position(file, program.trivia.as_slice(), node.span().start.offset)
}

/// Retrieves the docblock comment that appears before a specific position in the source code.
///
/// This function scans the trivia associated with the source code and returns the last docblock comment
/// that appears before the specified position, ensuring that it is directly preceding the node
/// without any non-whitespace characters in between.
///
/// # Arguments
///
/// * `file` - The file from which the trivia is derived.
/// * `trivias` - A slice of trivia associated with the source code.
/// * `node_start_offset` - The start offset of the node for which to find the preceding docblock comment.
///
/// # Returns
///
/// An `Option` containing a reference to the `Trivia` representing the docblock comment if found,
pub fn get_docblock_before_position<'arena>(
    file: &File,
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
    let mut covered_from = node_start_offset;

    for i in (0..candidate_partition_idx).rev() {
        let trivia = &trivias[i];
        let trivia_end = trivia.span.end_offset();

        // Check if there's a gap between this trivia and our covered region.
        // If there's non-whitespace content in the gap, there's actual code between them.
        let gap_slice = file.contents.as_bytes().get(trivia_end as usize..covered_from as usize).unwrap_or(&[]);

        if !gap_slice.iter().all(u8::is_ascii_whitespace) {
            // There's actual code in the gap. No docblock applies.
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
