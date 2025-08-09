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
pub fn get_docblock_for_node<'a>(program: &'a Program, file: &File, node: impl HasSpan) -> Option<&'a Trivia> {
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
pub fn get_docblock_before_position<'a>(
    file: &File,
    trivias: &'a [Trivia],
    node_start_offset: usize,
) -> Option<&'a Trivia> {
    let candidate_partition_idx = trivias.partition_point(|trivia| trivia.span.start.offset < node_start_offset);
    if candidate_partition_idx == 0 {
        return None;
    }

    for i in (0..candidate_partition_idx).rev() {
        let trivia = &trivias[i];

        match trivia.kind {
            TriviaKind::DocBlockComment => {
                let docblock_end_offset = trivia.span().end.offset;

                // Get the slice between docblock end and class start
                let code_between_slice =
                    file.contents.as_bytes().get(docblock_end_offset..node_start_offset).unwrap_or(&[]);

                if code_between_slice.iter().all(|b| b.is_ascii_whitespace()) {
                    // It's the correct docblock!
                    return Some(trivia);
                } else {
                    // There was non-whitespace code between this docblock and the class.
                    // This docblock doesn't apply. Stop searching.
                    return None;
                }
            }
            TriviaKind::WhiteSpace => {
                continue;
            }
            _ => {
                return None;
            }
        }
    }

    // Iterated through all preceding trivia without finding a suitable docblock.
    None
}
