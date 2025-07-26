use serde::Deserialize;
use serde::Serialize;

use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TypeString {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ReturnTypeTag {
    pub span: Span,
    pub type_string: TypeString,
    pub description: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TypeTag {
    pub span: Span,
    pub name: String,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ImportTypeTag {
    pub span: Span,
    pub name: String,
    pub from: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ParameterTag {
    pub span: Span,
    pub name: String,
    pub type_string: TypeString,
    pub description: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ParameterOutTag {
    pub span: Span,
    pub name: String,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThisOutTag {
    pub span: Span,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct IfThisIsTag {
    pub span: Span,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThrowsTag {
    pub span: Span,
    pub type_string: TypeString,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(i8)]
pub enum TemplateModifier {
    Of,
    As,
    Super,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TemplateTag {
    /// The full span of the original content parsed (e.g., "T as Foo").
    pub span: Span,
    /// The name of the template parameter (e.g., "T").
    pub name: String,
    /// The optional modifier (`as`, `of`, `super`).
    pub modifier: Option<TemplateModifier>,
    /// The optional constraint type string following the modifier, with its span.
    pub type_string: Option<TypeString>,
    /// Whether the template was declared as covariant (`@template-covariant`).
    pub covariant: bool,
    /// Whether the template was declared as contravariant (`@template-contravariant`).
    pub contravariant: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AssertionTag {
    pub span: Span,
    pub type_string: TypeString,
    pub parameter_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VarTag {
    pub span: Span,
    pub type_string: TypeString,
    pub variable_name: Option<String>,
}

/// Parses the content string of a `@template` or `@template-covariant` tag.
///
/// Extracts the template name, an optional modifier (`as`, `of`, `super`),
/// and an optional constraint type following the modifier.
///
/// Examples:
///
/// - "T" -> name="T", modifier=None, type=None
/// - "T of U" -> name="T", modifier=Of, type="U"
/// - "T as string" -> name="T", modifier=As, type="string"
/// - "T super \\My\\Class" -> name="T", modifier=Super, type="\\My\\Class"
/// - "T string" -> name="T", modifier=None, type=None (ignores "string")
/// - "T of" -> name="T", modifier=Of, type=None
///
/// # Arguments
///
/// * `content` - The string slice content following `@template` or `@template-covariant`.
/// * `span` - The original `Span` of the `content` slice within its source file.
/// * `covariant` - `true` if the tag was `@template-covariant`.
/// * `contravariant` - `true` if the tag was `@template-contravariant`.
///
/// # Returns
///
/// A `Result` containing the parsed `TemplateTag` or a `TemplateParseError`.
#[inline]
pub fn parse_template_tag(
    content: &str,
    span: Span,
    mut covariant: bool,
    mut contravariant: bool,
) -> Option<TemplateTag> {
    // Find start offset of trimmed content relative to original `content`
    let trim_start_offset_rel = content.find(|c: char| !c.is_whitespace()).unwrap_or(0);
    let trimmed_content = content.trim();

    if trimmed_content.is_empty() {
        return None;
    }

    let mut parts = trimmed_content.split_whitespace();

    let mut name_part = parts.next()?;
    if name_part.starts_with('+') && !contravariant && !covariant {
        covariant = true;
        name_part = &name_part[1..];
    } else if name_part.starts_with('-') && !contravariant && !covariant {
        contravariant = true;
        name_part = &name_part[1..];
    }

    let name = name_part.to_string();

    let mut modifier: Option<TemplateModifier> = None;
    let mut type_string_opt: Option<TypeString> = None;

    // Track current position relative to the start of the *original* content string
    // Start after the name part
    let mut current_offset_rel = trim_start_offset_rel + name_part.len();

    // 2. Check for optional modifier
    // Need to peek into the *original* content slice to find the next non-whitespace char
    let remaining_after_name = content.get(current_offset_rel..).unwrap_or("");
    let whitespace_len1 = remaining_after_name.find(|c: char| !c.is_whitespace()).unwrap_or(0);
    let after_whitespace1_offset_rel = current_offset_rel + whitespace_len1;
    let potential_modifier_slice = remaining_after_name.trim_start();

    if !potential_modifier_slice.is_empty() {
        let mut modifier_parts = potential_modifier_slice.split_whitespace().peekable();
        if let Some(potential_modifier_str) = modifier_parts.peek().copied() {
            let modifier_val = match potential_modifier_str.to_ascii_lowercase().as_str() {
                "as" => Some(TemplateModifier::As),
                "of" => Some(TemplateModifier::Of),
                "super" => Some(TemplateModifier::Super),
                _ => None,
            };

            if modifier_val.is_some() {
                modifier = modifier_val;
                modifier_parts.next();
                current_offset_rel = after_whitespace1_offset_rel + potential_modifier_str.len();

                // 3. If modifier found, look for the type string part
                let remaining_after_modifier = content.get(current_offset_rel..).unwrap_or("");
                let whitespace_len2 = remaining_after_modifier.find(|c: char| !c.is_whitespace()).unwrap_or(0);
                let type_start_offset_rel = current_offset_rel + whitespace_len2;
                let type_part_str = remaining_after_modifier.split_whitespace().next();

                if let Some(ts) = type_part_str
                    && !ts.is_empty()
                {
                    let type_len = ts.len();
                    let type_start_pos = span.start.forward(type_start_offset_rel);
                    let type_span = Span::new(type_start_pos, type_start_pos.forward(type_len));
                    type_string_opt = Some(TypeString { value: ts.to_string(), span: type_span });
                }
            }
        }
    }

    Some(TemplateTag { span, name, modifier, type_string: type_string_opt, covariant, contravariant })
}

/// Parses the content string of a `@param` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@param`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(ParamTag)` if parsing is successful, `None` otherwise.
pub fn parse_param_tag(content: &str, span: Span) -> Option<ParameterTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return None;
    }

    if rest_slice.is_empty() {
        // Variable name is mandatory
        return None;
    }

    let mut rest_parts = rest_slice.split_whitespace();

    let name_part = rest_parts.next()?;
    if name_part.len() <= 1 || (!name_part.starts_with('$') && !name_part.starts_with("...$")) {
        return None;
    }

    let name = name_part.to_owned();
    let description = if rest_parts.peekable().peek().is_some() {
        let desc_start_byte = rest_slice.find(name_part).map_or(0, |i| i + name_part.len());
        let desc_slice = rest_slice[desc_start_byte..].trim_start();

        desc_slice.to_owned()
    } else {
        String::new()
    };

    Some(ParameterTag { span, name, type_string, description })
}

/// Parses the content string of a `@param-out` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@param-out`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(ParamOutTag)` if parsing is successful, `None` otherwise.
pub fn parse_param_out_tag(content: &str, span: Span) -> Option<ParameterOutTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return None;
    }

    if rest_slice.is_empty() {
        return None;
    }

    let name_part = rest_slice.split_whitespace().next()?;
    if !name_part.starts_with('$') || name_part.len() <= 1 {
        return None;
    }
    let name = name_part.to_owned();

    Some(ParameterOutTag { span, name, type_string })
}

/// Parses the content string of a `@return` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@return`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(ReturnTypeTag)` if parsing is successful, `None` otherwise.
pub fn parse_return_tag(content: &str, span: Span) -> Option<ReturnTypeTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type cannot start with '{'
    if type_string.value.starts_with('{') {
        return None;
    }

    let description = rest_slice.to_owned();

    Some(ReturnTypeTag { span, type_string, description })
}

/// Parses the content string of a `@throws` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@throws`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(ThrowsTag)` if parsing is successful, `None` otherwise.
pub fn parse_throws_tag(content: &str, span: Span) -> Option<ThrowsTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type cannot start with '{'
    if type_string.value.starts_with('{') {
        return None;
    }

    // Type cannot start with '$' unless it is "$this"
    if type_string.value.starts_with('$') && type_string.value != "$this" {
        return None;
    }

    let description = rest_slice.to_owned();

    Some(ThrowsTag { span, type_string, description })
}

/// Parses the content string of a `@if-this-is` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(IfThisIsTag)` if parsing is successful, `None` otherwise.
pub fn parse_if_this_is_tag(content: &str, span: Span) -> Option<IfThisIsTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type cannot start with '{'
    if type_string.value.starts_with('{') {
        return None;
    }

    // Type cannot start with '$' unless it is "$this"
    if type_string.value.starts_with('$') && type_string.value != "$this" {
        return None;
    }

    if rest_slice.is_empty() {
        return None;
    }

    Some(IfThisIsTag { span, type_string })
}

/// Parses the content string of a `@this-out` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(ThisOutTag)` if parsing is successful, `None` otherwise.
pub fn parse_this_out_tag(content: &str, span: Span) -> Option<ThisOutTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type cannot start with '{'
    if type_string.value.starts_with('{') {
        return None;
    }

    // Type cannot start with '$' unless it is "$this"
    if type_string.value.starts_with('$') && type_string.value != "$this" {
        return None;
    }

    if rest_slice.is_empty() {
        return None;
    }

    Some(ThisOutTag { span, type_string })
}

/// Parses the content string of an `@assert`, `@assert-if-true`, or `@assert-if-false` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(AssertionTag)` if parsing is successful, `None` otherwise.
pub fn parse_assertion_tag(content: &str, span: Span) -> Option<AssertionTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return None;
    }

    if rest_slice.is_empty() {
        // Variable name is mandatory
        return None;
    }

    let mut rest_parts = rest_slice.split_whitespace();

    let name_part = rest_parts.next()?;
    if !name_part.starts_with('$') || name_part.len() <= 1 {
        return None;
    }

    let param_name = name_part.to_owned();

    Some(AssertionTag { span, parameter_name: param_name, type_string })
}

/// Parses the content string of a `@var` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(VarTag)` if parsing is successful, `None` otherwise.
pub fn parse_var_tag(content: &str, span: Span) -> Option<VarTag> {
    let (type_string, rest_slice) = split_tag_content(content, span)?;

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return None;
    }

    let variable_name = if rest_slice.is_empty() {
        None
    } else {
        let var_part = rest_slice.split_whitespace().next()?;
        if var_part.starts_with('$') && var_part.len() > 1 { Some(var_part.to_owned()) } else { None }
    };

    Some(VarTag { span, type_string, variable_name })
}

/// Parses the content string of a `@type` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(TypeTag)` if parsing is successful, `None` otherwise.
pub fn parse_type_tag(content: &str, span: Span) -> Option<TypeTag> {
    let equals_index = content.find('=')?;

    let (name, rest) = content.split_at(equals_index);
    let name = name.trim();

    if !is_valid_identifier_start(name, false) || rest.is_empty() {
        return None;
    }

    let (type_string, _) = split_tag_content(&rest[1..], span.subspan(equals_index, 0))?;

    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return None;
    }

    Some(TypeTag { span, name: name.to_owned(), type_string })
}

/// Parses the content string of an `@import-type` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(ImportTypeTag)` if parsing is successful, `None` otherwise.
pub fn parse_import_type_tag(content: &str, span: Span) -> Option<ImportTypeTag> {
    let (name, rest) = content.split_once(" ")?;
    let name = name.trim();
    let rest = rest.trim();

    if !is_valid_identifier_start(name, false) || rest.is_empty() {
        return None;
    }

    let (from, rest) = rest.split_once(" ")?;
    if !from.eq_ignore_ascii_case("from") || rest.is_empty() {
        return None;
    }

    let (imported_from, rest) = rest.split_once(" ")?;
    if !is_valid_identifier_start(imported_from, true) {
        return None;
    }

    let rest = rest.trim();
    let mut alias = None;
    if !rest.is_empty() {
        let (r#as, rest) = rest.split_once(" ")?;
        if r#as.eq_ignore_ascii_case("as") && !rest.is_empty() {
            alias = Some(rest.split_whitespace().next()?.trim().to_owned());
        }
    }

    Some(ImportTypeTag { span, name: name.to_owned(), from: imported_from.to_owned(), alias })
}

/// Splits tag content into the type string part and the rest, respecting brackets/quotes.
/// Calculates the absolute span of the identified type string.
///
/// Returns None if parsing fails or input is empty.
///
/// Output: `Some((TypeString, rest_slice))` or `None`
#[inline]
pub fn split_tag_content(content: &str, input_span: Span) -> Option<(TypeString, &str)> {
    // Find start byte offset of trimmed content relative to original `content` slice
    let trim_start_offset = content.find(|c: char| !c.is_whitespace()).unwrap_or(0);
    // Calculate the absolute start position of the trimmed content
    let trimmed_start_pos = input_span.start.forward(trim_start_offset);

    // Get the trimmed slice reference to iterate over
    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        return None;
    }

    let mut bracket_stack: Vec<char> = Vec::with_capacity(8);
    let mut quote_char: Option<char> = None;
    let mut escaped = false;
    let mut last_char_was_significant = false;
    // Potential split point *relative to trimmed_content*
    let mut split_point_rel: Option<usize> = None;

    let mut iter = trimmed_content.char_indices().peekable();

    while let Some((i, char)) = iter.next() {
        if let Some(q) = quote_char {
            if char == q && !escaped {
                quote_char = None;
            } else {
                escaped = char == '\\' && !escaped;
            }
            last_char_was_significant = true;
            continue;
        }
        if char == '\'' || char == '"' {
            quote_char = Some(char);
            last_char_was_significant = true;
            continue;
        }
        match char {
            '<' | '(' | '[' | '{' => bracket_stack.push(char),
            '>' | ')' | ']' | '}' => {
                match bracket_stack.pop() {
                    Some(opening) if brackets_match(&opening, &char) => {}
                    _ => return None, // Mismatch or unbalanced
                }
            }
            _ => {}
        }

        // if we are at `:` then consider it significant and consume following
        // whitespaces, and continue processing
        if char == ':' {
            last_char_was_significant = true;
            while let Some(&(_, next_char)) = iter.peek() {
                if next_char.is_whitespace() {
                    iter.next();
                } else {
                    break;
                }
            }

            continue;
        }

        if char == '/' && iter.peek().is_some_and(|&(_, c)| c == '/') {
            if !bracket_stack.is_empty() {
                last_char_was_significant = true;
                continue;
            }

            // Split point is BEFORE the comment start
            split_point_rel = Some(i);

            // Stop processing line here, rest will be handled outside loop
            break;
        }

        if char.is_whitespace() {
            if bracket_stack.is_empty() && last_char_was_significant {
                // Found the first potential split point
                split_point_rel = Some(i);
                break;
            }
            last_char_was_significant = false;
        } else {
            last_char_was_significant = true;
        }
    }

    // After loop checks
    if !bracket_stack.is_empty() || quote_char.is_some() {
        return None;
    }

    match split_point_rel {
        Some(split_idx_rel) => {
            // Split occurred
            let type_part_slice = trimmed_content[..split_idx_rel].trim_end();
            let rest_part_slice = trimmed_content[split_idx_rel..].trim_start();

            // Calculate span relative to the *start* of the trimmed content
            let type_span = Span::new(trimmed_start_pos, trimmed_start_pos.forward(type_part_slice.len()));
            Some((TypeString { value: type_part_slice.to_owned(), span: type_span }, rest_part_slice))
        }
        None => {
            // No split, entire trimmed content is the type
            let type_part_slice = trimmed_content;
            let type_span = Span::new(trimmed_start_pos, trimmed_start_pos.forward(type_part_slice.len()));
            Some((TypeString { value: type_part_slice.to_owned(), span: type_span }, ""))
        }
    }
}

/// Checks if an opening bracket matches a closing one.
#[inline]
const fn brackets_match(open: &char, close: &char) -> bool {
    matches!((open, close), ('<', '>') | ('(', ')') | ('[', ']') | ('{', '}'))
}

/// Checks if the identifier is valid
#[inline]
fn is_valid_identifier_start(mut identifier: &str, allow_qualified: bool) -> bool {
    if allow_qualified && identifier.starts_with("\\") {
        identifier = &identifier[1..];
    }

    !identifier.is_empty()
        && identifier.chars().all(|c| c.is_alphanumeric() || c == '_' || (allow_qualified && c == '\\'))
        && identifier.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
}

#[cfg(test)]
mod tests {
    use mago_source::SourceIdentifier;
    use mago_span::Position;
    use mago_span::Span;

    use super::*;

    fn test_span(input: &str, start_offset: usize) -> Span {
        let source = SourceIdentifier::dummy();
        let base_start = Position::new(source, start_offset);
        Span::new(base_start, base_start.forward(input.len()))
    }

    fn test_span_for(s: &str) -> Span {
        test_span(s, 0)
    }

    fn make_span(start: usize, end: usize) -> Span {
        let source = SourceIdentifier::dummy();
        Span::new(Position::new(source, start), Position::new(source, end))
    }

    #[test]
    fn test_splitter_brackets() {
        let input = "array<int, (string|bool)> desc";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<int, (string|bool)>");
        assert_eq!(ts.span, make_span(0, "array<int, (string|bool)>".len()));
        assert_eq!(rest, "desc");

        let input = "array<int, string> desc";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<int, string>");
        assert_eq!(ts.span, make_span(0, "array<int, string>".len()));
        assert_eq!(rest, "desc");

        assert!(split_tag_content("array<int", test_span_for("array<int")).is_none()); // Unclosed
        assert!(split_tag_content("array<int)", test_span_for("array<int)")).is_none()); // Mismatched
        assert!(split_tag_content("array(int>", test_span_for("array(int>")).is_none()); // Mismatched
        assert!(split_tag_content("string>", test_span_for("string>")).is_none()); // Closing without opening
    }

    #[test]
    fn test_splitter_quotes() {
        let input = " 'inside quote' outside ";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "'inside quote'");
        assert_eq!(ts.span, make_span(1, "'inside quote'".len() + 1));
        assert_eq!(rest, "outside");

        let input = r#""string \" with escape" $var"#;
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, r#""string \" with escape""#);
        assert_eq!(ts.span, make_span(0, r#""string \" with escape""#.len()));
        assert_eq!(rest, "$var");

        assert!(split_tag_content("\"unterminated", test_span_for("\"unterminated")).is_none());
    }

    #[test]
    fn test_splitter_comments() {
        let input = "(string // comment \n | int) $var";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "(string // comment \n | int)");
        assert_eq!(ts.span, make_span(0, "(string // comment \n | int)".len()));
        assert_eq!(rest, "$var");

        let input = "string // comment goes to end";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "string");
        assert_eq!(ts.span, make_span(0, "string".len()));
        assert_eq!(rest, "// comment goes to end");

        let input = "array<string // comment\n> $var";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<string // comment\n>");
        assert_eq!(ts.span, make_span(0, "array<string // comment\n>".len()));
        assert_eq!(rest, "$var");
    }

    #[test]
    fn test_splitter_whole_string_is_type() {
        let input = " array<int, string> ";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<int, string>");
        assert_eq!(ts.span, make_span(1, "array<int, string>".len() + 1));
        assert_eq!(rest, ""); // No rest part
    }

    #[test]
    fn test_param_basic() {
        let offset = 10;
        let content = " string|int $myVar Description here ";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();

        assert_eq!(result.type_string.value, "string|int"); // Check owned string value
        assert_eq!(result.type_string.span.start.offset, offset + 1); // Span of type part
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "string|int".len());
        assert_eq!(result.name, "$myVar");
        assert_eq!(result.description, "Description here");
        assert_eq!(result.span, span); // Check overall span
    }

    #[test]
    fn test_param_complex_type_no_desc() {
        let offset = 5;
        let content = " array<int, string> $param ";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, "array<int, string>"); // Check owned string
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, string>".len());
        assert_eq!(result.name, "$param");
        assert_eq!(result.description, "");
    }

    #[test]
    fn test_param_type_with_comment() {
        let offset = 20;
        let content = " (string // comment \n | int) $var desc";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, "(string // comment \n | int)");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "(string // comment \n | int)".len());
        assert_eq!(result.name, "$var");
        assert_eq!(result.description, "desc");
    }

    #[test]
    fn test_param_no_type() {
        let content = " $param Description here ";
        let span = test_span(content, 0);
        assert!(parse_param_tag(content, span).is_none()); // No type before var
    }

    #[test]
    fn test_return_basic() {
        let offset = 10;
        let content = " string Description here ";
        let span = test_span(content, offset);
        let result = parse_return_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, "string");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "string".len());
        assert_eq!(result.description, "Description here");
        assert_eq!(result.span, span);
    }

    #[test]
    fn test_return_complex_type_with_desc() {
        let offset = 0;
        let content = " array<int, (string|null)> Description ";
        let span = test_span(content, offset);
        let result = parse_return_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, "array<int, (string|null)>");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, (string|null)>".len());
        assert_eq!(result.description, "Description");
    }

    #[test]
    fn test_return_complex_type_no_desc() {
        let offset = 0;
        let content = " array<int, (string|null)> ";
        let span = test_span(content, offset);
        let result = parse_return_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, "array<int, (string|null)>");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, (string|null)>".len());
        assert_eq!(result.description, "");
    }

    #[test]
    fn test_param_out_no_type() {
        let content = " $myVar ";
        let span = test_span(content, 0);
        assert!(parse_param_out_tag(content, span).is_none());
    }

    #[test]
    fn test_param_out_no_var() {
        let content = " string ";
        let span = test_span(content, 0);
        assert!(parse_param_out_tag(content, span).is_none());
    }

    #[test]
    fn test_type() {
        let content = "MyType = string";
        let span = test_span_for(content);
        let result = parse_type_tag(content, span).unwrap();
        assert_eq!(result.name, "MyType");
        assert_eq!(result.type_string.value, "string");
        assert_eq!(result.type_string.span.start.offset, 8);
        assert_eq!(result.type_string.span.end.offset, 8 + "string".len());
        assert_eq!(result.span, span);
    }

    #[test]
    fn test_import_type() {
        let content = "MyType from \\My\\Namespace\\Class as Alias";
        let span = test_span_for(content);
        let result = parse_import_type_tag(content, span).unwrap();
        assert_eq!(result.name, "MyType");
        assert_eq!(result.from, "\\My\\Namespace\\Class");
        assert_eq!(result.alias, Some("Alias".to_owned()));
        assert_eq!(result.span, span);
    }
}
