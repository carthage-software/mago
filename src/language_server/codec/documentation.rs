//! Render source documentation for hover and signature help.

use mago_server::{DocumentationTagKind, SourceDocumentation};

pub(super) fn render(document: &SourceDocumentation, markdown: bool) -> String {
    let mut out = String::new();
    let mut cursor = 0;
    for tag in &document.tags {
        out.push_str(&document.text[cursor..tag.range.start as usize]);
        if !out.is_empty() && !out.ends_with("\n\n") {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }

        let (label, parameter, type_name) = match &tag.kind {
            DocumentationTagKind::Parameter { parameter, type_name } => {
                ("Parameter", parameter.as_deref(), type_name.as_deref())
            }
            DocumentationTagKind::Returns { type_name } => ("Returns", None, Some(type_name.as_str())),
            DocumentationTagKind::Throws { type_name } => ("Throws", None, Some(type_name.as_str())),
            DocumentationTagKind::Deprecated => ("Deprecated", None, None),
        };

        out.push_str(&if markdown { format!("**{label}**") } else { label.to_owned() });
        for value in [parameter, type_name].into_iter().flatten() {
            out.push(' ');
            out.push_str(&if markdown { code_span(value) } else { value.to_owned() });
        }

        cursor = tag.range.end as usize;
    }

    out.push_str(&document.text[cursor..]);
    out
}

pub(super) fn code_span(value: &str) -> String {
    let longest = value.split(|character| character != '`').map(str::len).max().unwrap_or(0);
    let bound = "`".repeat(longest + 1);
    let pad = if value.starts_with('`') || value.ends_with('`') || value.starts_with(' ') && value.ends_with(' ') {
        " "
    } else {
        ""
    };

    format!("{bound}{pad}{value}{pad}{bound}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_server::{DocumentationTag, Range};

    #[test]
    fn labels_tags_and_keeps_the_remaining_source() {
        let text = "Summary.\n\n@param string $name The **name**.\n@return string\n@unknown Keep me.";
        let parameter = text.find("@param").unwrap() as u32;
        let returns = text.find("@return").unwrap() as u32;
        let document = SourceDocumentation {
            text: text.into(),
            tags: vec![
                DocumentationTag {
                    range: Range::new(parameter, parameter + "@param string $name".len() as u32),
                    kind: DocumentationTagKind::Parameter {
                        parameter: Some("$name".into()),
                        type_name: Some("string".into()),
                    },
                },
                DocumentationTag {
                    range: Range::new(returns, returns + "@return string".len() as u32),
                    kind: DocumentationTagKind::Returns { type_name: "string".into() },
                },
            ],
        };
        assert_eq!(
            render(&document, true),
            "Summary.\n\n**Parameter** `$name` `string` The **name**.\n\n**Returns** `string`\n@unknown Keep me.",
        );
        assert_eq!(
            render(&document, false),
            "Summary.\n\nParameter $name string The **name**.\n\nReturns string\n@unknown Keep me.",
        );
    }

    #[test]
    fn raw_documents_and_code_delimiters_survive() {
        let text = "~~~php\n@return int\n~~~\n\nUnknown {@custom value}.";
        let document = SourceDocumentation { text: text.into(), tags: vec![] };
        assert_eq!(render(&document, true), text);
        assert_eq!(code_span("'`'"), "``'`'``");
        assert_eq!(code_span("`value`"), "`` `value` ``");
    }
}
