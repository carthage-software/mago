use ahash::HashMap;
use ahash::HashSet;

use mago_atom::empty_atom;
use mago_database::file::FileId;

use crate::diff::CodebaseDiff;
use crate::diff::DeletionRange;
use crate::diff::DiffHunk;
use crate::signature::DefSignatureNode;
use crate::signature::FileSignature;

/// Computes the difference between an old file signature and a new file signature.
///
/// This function uses the Myers diff algorithm to efficiently identify changes between
/// two versions of a file's AST. Unlike Hakana which differentiates between signature
/// and body changes, we use a single hash approach: any change triggers re-analysis.
///
/// # Arguments
///
/// * `file_id` - The identifier of the file being compared (used for `diff_map`)
/// * `old_signature` - The previous file signature (None if this is a new file)
/// * `new_signature` - The current file signature
///
/// # Returns
///
/// A `CodebaseDiff` containing:
/// - `keep`: Symbols that are unchanged
/// - `changed`: Symbols that are new, deleted, or modified
/// - `diff_map`: Position mappings for symbols that moved
/// - `deletion_ranges_map`: Ranges of deleted code
#[must_use]
pub fn compute_file_diff(
    file_id: FileId,
    old_signature: Option<&FileSignature>,
    new_signature: Option<&FileSignature>,
) -> CodebaseDiff {
    let Some(new_signature) = new_signature else {
        return CodebaseDiff::new();
    };

    match old_signature {
        None => {
            // New file - all symbols are changed
            mark_all_as_changed(new_signature)
        }
        Some(old_sig) => {
            // Existing file - use Myers diff
            myers_diff(file_id, old_sig, new_signature)
        }
    }
}

/// Marks all symbols in a file signature as changed (used for new files).
fn mark_all_as_changed(signature: &FileSignature) -> CodebaseDiff {
    let mut changed = HashSet::default();

    for node in &signature.ast_nodes {
        // Add top-level symbol
        changed.insert((node.name, empty_atom()));

        // Add all children (methods, properties, etc.)
        for child in &node.children {
            changed.insert((node.name, child.name));
        }
    }

    CodebaseDiff::new().with_changed(changed)
}

/// Computes a detailed diff between two file signatures using Myers algorithm.
///
/// This function implements a two-level Myers diff:
/// 1. Top-level diff: compares classes, functions, and constants
/// 2. Member-level diff: compares methods, properties, and class constants within each class
///
/// For each symbol, it checks both structural changes (add/remove/keep) and content changes
/// (hash comparison) to determine if re-analysis is needed.
///
/// # Implementation
///
/// Adapted from Hakana's implementation, but uses a single fingerprint hash per symbol
/// instead of separate signature/body hashes. Any hash change triggers re-analysis.
///
/// See: <https://github.com/slackhq/hakana/blob/35890f99ded7897e4203a896fd1636bda300bad6/src/orchestrator/ast_differ.rs#L10-L13>
///
/// # Arguments
///
/// * `file_id` - File being compared (for tracking position changes)
/// * `old_signature` - Previous file signature
/// * `new_signature` - Current file signature
///
/// # Returns
///
/// A `CodebaseDiff` with:
/// - `keep`: Unchanged symbols
/// - `changed`: Added, removed, or modified symbols
/// - `diff_map`: Position changes for issue mapping
/// - `deletion_ranges_map`: Deleted code ranges for issue filtering
fn myers_diff(file_id: FileId, old_signature: &FileSignature, new_signature: &FileSignature) -> CodebaseDiff {
    let mut keep = HashSet::default();
    let mut changed = HashSet::default();
    let mut file_diffs: Vec<DiffHunk> = vec![];
    let mut deletion_ranges: Vec<DeletionRange> = vec![];

    let Ok((trace, x, y)) = calculate_trace(&old_signature.ast_nodes, &new_signature.ast_nodes) else {
        tracing::warn!("Myers diff algorithm failed to converge for file {file_id:?}, marking all symbols as changed");

        return mark_all_as_changed(new_signature);
    };

    let diff = extract_diff(&trace, x, y, &old_signature.ast_nodes, &new_signature.ast_nodes);

    for diff_elem in diff {
        match diff_elem {
            AstDiffElem::Keep(a, b) => {
                let mut has_child_change = false;

                let Ok((class_trace, class_x, class_y)) = calculate_trace(&a.children, &b.children) else {
                    changed.insert((a.name, empty_atom()));
                    for child in &a.children {
                        changed.insert((a.name, child.name));
                    }

                    for child in &b.children {
                        changed.insert((b.name, child.name));
                    }

                    continue;
                };

                let class_diff = extract_diff(&class_trace, class_x, class_y, &a.children, &b.children);

                for class_diff_elem in class_diff {
                    match class_diff_elem {
                        AstDiffElem::Keep(a_child, b_child) => {
                            // Check if the child's hash changed
                            if a_child.hash == b_child.hash {
                                keep.insert((a.name, a_child.name));
                            } else {
                                has_child_change = true;
                                changed.insert((a.name, a_child.name));
                            }

                            // Track position changes for issue mapping
                            if b_child.start_offset != a_child.start_offset || b_child.start_line != a_child.start_line
                            {
                                file_diffs.push((
                                    a_child.start_offset as usize,
                                    a_child.end_offset as usize,
                                    b_child.start_offset as isize - a_child.start_offset as isize,
                                    b_child.start_line as isize - a_child.start_line as isize,
                                ));
                            }
                        }
                        AstDiffElem::Remove(child_node) => {
                            has_child_change = true;
                            changed.insert((a.name, child_node.name));
                            deletion_ranges.push((child_node.start_offset as usize, child_node.end_offset as usize));
                        }
                        AstDiffElem::Add(child_node) => {
                            has_child_change = true;
                            changed.insert((a.name, child_node.name));
                        }
                    }
                }

                // Check if parent's hash changed or if any child changed
                if has_child_change || a.hash != b.hash {
                    changed.insert((a.name, empty_atom()));
                } else {
                    keep.insert((a.name, empty_atom()));

                    // Track position changes for issue mapping
                    if b.start_offset != a.start_offset || b.start_line != a.start_line {
                        file_diffs.push((
                            a.start_offset as usize,
                            a.end_offset as usize,
                            b.start_offset as isize - a.start_offset as isize,
                            b.start_line as isize - a.start_line as isize,
                        ));
                    }
                }
            }
            AstDiffElem::Remove(node) => {
                changed.insert((node.name, empty_atom()));
                deletion_ranges.push((node.start_offset as usize, node.end_offset as usize));

                // Also mark all children as removed
                for child in &node.children {
                    changed.insert((node.name, child.name));
                }
            }
            AstDiffElem::Add(node) => {
                changed.insert((node.name, empty_atom()));

                // Also mark all children as added
                for child in &node.children {
                    changed.insert((node.name, child.name));
                }
            }
        }
    }

    let mut diff = CodebaseDiff::new().with_keep(keep).with_changed(changed);

    if !file_diffs.is_empty() {
        diff.add_diff_map_entry(file_id, file_diffs);
    }

    if !deletion_ranges.is_empty() {
        diff.add_deletion_ranges_entry(file_id, deletion_ranges);
    }

    diff
}

/// Type alias for the Myers diff trace structure.
///
/// - Vec<`HashMap`<isize, usize>>: The trace of the search path
/// - usize: Final position in the old sequence
/// - usize: Final position in the new sequence
type DiffTrace = (Vec<HashMap<isize, usize>>, usize, usize);

/// Implements the Myers diff algorithm.
///
/// Borrows from:
/// - <https://github.com/nikic/PHP-Parser/blob/master/lib/PhpParser/Internal/Differ.php>
/// - <https://github.com/slackhq/hakana/blob/35890f99ded7897e4203a896fd1636bda300bad6/src/orchestrator/ast_differ.rs#L151-L159>
///
/// Myers, Eugene W. "An O(ND) difference algorithm and its variations."
/// Algorithmica 1.1 (1986): 251-266.
///
/// Returns a Result containing a tuple of (trace, x, y) where:
/// - trace: A vector of hash maps representing the search path
/// - x: Final position in the old sequence
/// - y: Final position in the new sequence
///
/// Returns Err if the algorithm fails to converge (theoretically impossible but handled gracefully).
fn calculate_trace(a_nodes: &[DefSignatureNode], b_nodes: &[DefSignatureNode]) -> Result<DiffTrace, &'static str> {
    let n = a_nodes.len();
    let m = b_nodes.len();
    let max = n + m;
    let mut v: HashMap<isize, usize> = HashMap::default();
    v.insert(1, 0);
    let mut trace = vec![];

    for d in 0..=(max as isize) {
        trace.push(v.clone());
        let mut k = -d;
        while k <= d {
            let mut x = if k == -d || (k != d && v[&(k - 1)] < v[&(k + 1)]) { v[&(k + 1)] } else { v[&(k - 1)] + 1 };

            let mut y = (x as isize - k) as usize;

            // Advance along the diagonal while nodes are equal
            while x < n && y < m && is_equal(&a_nodes[x], &b_nodes[y]) {
                x += 1;
                y += 1;
            }

            v.insert(k, x);

            // Found the end
            if x >= n && y >= m {
                return Ok((trace, x, y));
            }

            k += 2;
        }
    }

    Err("Myers diff algorithm failed to converge")
}

/// Checks if two `DefSignatureNode` instances can be matched for diffing.
///
/// Two nodes are considered matchable if they have the same:
/// - name
/// - `is_function` flag
///
/// We don't check hash here because we want to match nodes even if their
/// content changed. The hash difference will be detected later to determine
/// if they belong in the "keep" or "changed" set.
fn is_equal(a_node: &DefSignatureNode, b_node: &DefSignatureNode) -> bool {
    a_node.name == b_node.name && a_node.is_function == b_node.is_function
}

/// Extracts the diff elements from the Myers trace.
///
/// Walks backward through the trace to build a sequence of Keep, Remove, and Add operations.
fn extract_diff<'a>(
    trace: &[HashMap<isize, usize>],
    mut x: usize,
    mut y: usize,
    a_nodes: &'a [DefSignatureNode],
    b_nodes: &'a [DefSignatureNode],
) -> Vec<AstDiffElem<'a>> {
    let mut result = vec![];
    let mut d = trace.len() as isize - 1;

    while d >= 0 {
        let v = &trace[d as usize];
        let k = (x as isize) - (y as isize);

        let prev_k = if k == -d || (k != d && v[&(k - 1)] < v[&(k + 1)]) { k + 1 } else { k - 1 };

        let prev_x = v[&prev_k];
        let prev_y = prev_x as isize - prev_k;

        // Walk diagonals (unchanged elements)
        while x > prev_x && y as isize > prev_y {
            result.push(AstDiffElem::Keep(&a_nodes[x - 1], &b_nodes[y - 1]));
            x -= 1;
            y -= 1;
        }

        if d == 0 {
            break;
        }

        // Deletions
        while x > prev_x {
            result.push(AstDiffElem::Remove(&a_nodes[x - 1]));
            x -= 1;
        }

        // Additions
        while y as isize > prev_y {
            result.push(AstDiffElem::Add(&b_nodes[y - 1]));
            y -= 1;
        }

        d -= 1;
    }

    result.reverse();
    result
}

/// Represents a single element in the AST diff.
#[derive(Debug)]
enum AstDiffElem<'a> {
    /// Node unchanged in both old and new versions
    Keep(&'a DefSignatureNode, &'a DefSignatureNode),
    /// Node was removed in the new version
    Remove(&'a DefSignatureNode),
    /// Node was added in the new version
    Add(&'a DefSignatureNode),
}
