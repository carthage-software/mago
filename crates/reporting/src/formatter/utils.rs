use crate::Issue;

/// XML-encode a string by escaping special characters.
pub fn xml_encode(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    // the result will never be smaller than the input,
    // so we can preallocate the result with the same capacity.
    let mut result = String::with_capacity(input.len());

    for c in input.chars() {
        let next = match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&apos;",
            '\n' => "&#10;",
            '\r' => "&#13;",
            _ => {
                result.push(c);

                continue;
            }
        };

        result.push_str(next);
    }

    result
}

/// Build a long message from an issue including notes, help, and links.
pub fn long_message(issue: &Issue) -> String {
    let mut message = issue.message.clone();
    if !issue.notes.is_empty() {
        message.push('\n');

        for note in issue.notes.iter() {
            message.push('\n');
            message.push_str(note.as_str());
        }
    }

    if let Some(help) = issue.help.as_ref() {
        message.push_str("\n\nHelp: ");
        message.push_str(help.as_str());
    }

    if let Some(link) = issue.link.as_ref() {
        message.push_str("\n\nMore information: ");
        message.push_str(link.as_str());
    }

    message
}
