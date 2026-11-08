use mago_allocator::Arena;
use mago_allocator::vec::Vec;

pub(super) fn write<A>(output: &mut Vec<'_, u8, A>, comment: &[u8])
where
    A: Arena,
{
    if memchr::memchr2(b'\r', b'\n', comment).is_some() || comment.starts_with(b"{##") {
        output.extend_from_slice(comment);
        return;
    }

    let start = 2 + usize::from(comment.get(2).is_some_and(|byte| matches!(byte, b'-' | b'~')));
    let end = comment.len()
        - 2
        - usize::from(comment.get(comment.len().saturating_sub(3)).is_some_and(|byte| matches!(byte, b'-' | b'~')));
    if start > end {
        output.extend_from_slice(comment);
        return;
    }

    let body = comment[start..end].trim_ascii();
    let single_trim_marker = (start == 3) != (end == comment.len() - 3);
    output.extend_from_slice(&comment[..start]);
    if !body.is_empty() || single_trim_marker {
        output.push(b' ');
    }
    output.extend_from_slice(body);
    if !body.is_empty() {
        output.push(b' ');
    }
    output.extend_from_slice(&comment[end..]);
}
