use crate::document::Document;
use crate::document::IndentIfBreak;

pub fn will_break(document: &mut Document<'_>) -> bool {
    let check_array = |array: &mut Vec<Document<'_>>| array.iter_mut().rev().any(|doc| will_break(doc));

    match document {
        Document::BreakParent => true,
        Document::Group(group) => {
            if group.should_break {
                return true;
            }
            if let Some(expanded_states) = &mut group.expanded_states {
                if expanded_states.iter_mut().rev().any(will_break) {
                    return true;
                }
            }
            check_array(&mut group.contents)
        }
        Document::IfBreak(d) => will_break(&mut d.break_contents),
        Document::Array(arr)
        | Document::Indent(arr)
        | Document::LineSuffix(arr)
        | Document::IndentIfBreak(IndentIfBreak { contents: arr, .. }) => check_array(arr),
        Document::Fill(doc) => check_array(&mut doc.parts),
        Document::Line(doc) => doc.hard,
        Document::String(_) => false,
    }
}
