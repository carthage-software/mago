use crate::document::{Align, BreakMode, Document, Group, IndentIfBreak};
use crate::printer::Printer;
use mago_allocator::Arena;
use mago_allocator::collections::HashSet;

impl<'arena, A: Arena> Printer<'arena, A> {
    #[inline]
    pub(super) fn propagate_breaks(
        propagated_breaks: &mut HashSet<'arena, usize, A>,
        mut doc: &Document<'_, A>,
    ) -> bool {
        loop {
            match doc {
                Document::BreakParent => return true,
                Document::Group(group) => return Self::propagate_group_breaks(propagated_breaks, group),
                Document::IfBreak(d) => doc = d.break_contents,
                Document::Array(arr)
                | Document::Indent(arr)
                | Document::Align(Align { contents: arr, .. })
                | Document::IndentIfBreak(IndentIfBreak { contents: arr, .. }) => {
                    return Self::propagate_slice_breaks(propagated_breaks, arr);
                }
                _ => return false,
            }
        }
    }

    pub(super) fn propagate_slice_breaks(
        propagated_breaks: &mut HashSet<'arena, usize, A>,
        documents: &[Document<'_, A>],
    ) -> bool {
        documents.iter().rev().any(|doc| Self::propagate_breaks(propagated_breaks, doc))
    }

    pub(super) fn propagate_group_breaks(
        propagated_breaks: &mut HashSet<'arena, usize, A>,
        group: &Group<'_, A>,
    ) -> bool {
        if *group.break_mode.borrow() == BreakMode::Force {
            return true;
        }

        let cache_key = group.contents.first().map(|document| std::ptr::from_ref(document).addr()).or_else(|| {
            group
                .expanded_states
                .as_ref()
                .and_then(|states| states.first())
                .map(|document| std::ptr::from_ref(document).addr())
        });
        if let Some(cache_key) = cache_key
            && propagated_breaks.contains(&cache_key)
        {
            return false;
        }

        let mut should_break = false;

        if let Some(expanded_states) = &group.expanded_states {
            should_break |= Self::propagate_slice_breaks(propagated_breaks, expanded_states);
        }

        should_break |= Self::propagate_slice_breaks(propagated_breaks, &group.contents);

        if group.expanded_states.is_none() && should_break && *group.break_mode.borrow() != BreakMode::Preserve {
            group.break_mode.replace(BreakMode::Force);
        }

        let should_break = *group.break_mode.borrow() == BreakMode::Force;
        if !should_break && let Some(cache_key) = cache_key {
            propagated_breaks.insert(cache_key);
        }

        should_break
    }
}
