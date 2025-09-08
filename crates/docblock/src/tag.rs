use mago_span::Span;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Variable {
    pub name: String,          // normalized: includes `$`, excludes `...` and `&`
    pub is_variadic: bool,     // true if `...` was present
    pub is_by_reference: bool, // true if `&` was present
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_by_reference {
            f.write_str("&")?;
        }
        if self.is_variadic {
            f.write_str("...")?;
        }
        f.write_str(&self.name)
    }
}

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
    pub variable: Variable,
    pub type_string: TypeString,
    pub description: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ParameterOutTag {
    pub span: Span,
    pub variable: Variable,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThrowsTag {
    pub span: Span,
    pub type_string: TypeString,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum WhereModifier {
    Is,
    Colon,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct WhereTag {
    /// The full span of the original content parsed (e.g., "T is Foo").
    pub span: Span,
    /// The name of the template parameter (e.g., "T").
    pub name: String,
    /// The modifier (`is`, `:`).
    pub modifier: WhereModifier,
    /// The constraint type string following the modifier, with its span.
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AssertionTag {
    pub span: Span,
    pub type_string: TypeString,
    pub variable: Variable,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VarTag {
    pub span: Span,
    pub type_string: TypeString,
    pub variable: Option<Variable>,
}

/// Parses a PHPDoc variable token and returns a structured `Variable`.
///
/// Supports `$name`, `...$name`, and `&$name`.
/// The returned `Variable` stores a normalized `name` (with `$`, without leading `...` or `&`),
/// and sets flags `is_variadic` and `is_by_reference` that can be used for display/rendering.
///
/// Examples:
/// - "$foo"       → Some(Variable { name: "$foo", is_variadic: false, is_by_reference: false })
/// - "&$foo"      → Some(Variable { name: "$foo", is_variadic: false, is_by_reference: true })
/// - "...$ids)"   → Some(Variable { name: "$ids", is_variadic: true, is_by_reference: false })
/// - "$"          → None
/// - "...$"       → None
/// - "$1x"        → None
#[inline]
fn parse_var_ident(raw: &str) -> Option<Variable> {
    let is_by_reference = raw.starts_with('&');
    // tolerate "&$x" in docblocks
    let raw = raw.strip_prefix('&').unwrap_or(raw);

    // accept "$name" or "...$name"
    let (prefix_len, rest, is_variadic) = if let Some(r) = raw.strip_prefix("...$") {
        (4usize, r, true)
    } else if let Some(r) = raw.strip_prefix('$') {
        (1usize, r, false)
    } else {
        return None;
    };

    // PHP identifier rules (ASCII + underscore): [_A-Za-z][_A-Za-z0-9]*
    let bytes = rest.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let is_start = |b: u8| b == b'_' || b.is_ascii_alphabetic();
    let is_cont = |b: u8| is_start(b) || b.is_ascii_digit();

    if !is_start(bytes[0]) {
        return None;
    }

    let mut len = 1usize;
    while len < bytes.len() && is_cont(bytes[len]) {
        len += 1;
    }

    let token = &raw[..prefix_len + len];
    // normalized: remove variadic prefix if present, keep `$`
    let normalized = if is_variadic { &token[3..] } else { token };
    Some(Variable { name: normalized.to_owned(), is_variadic, is_by_reference })
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
                if let Some((type_string, _)) =
                    split_tag_content(remaining_after_modifier, span.subspan(current_offset_rel as u32, 0))
                {
                    type_string_opt = Some(type_string);
                }
            }
        }
    }

    Some(TemplateTag { span, name, modifier, type_string: type_string_opt, covariant, contravariant })
}

/// Parses the content string of a `@where` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@where`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Returns
///
/// `Some(WhereTag)` if parsing is successful, `None` otherwise.
pub fn parse_where_tag(content: &str, span: Span) -> Option<WhereTag> {
    let name_end_pos = content.find(char::is_whitespace)?;
    let (name_part, mut rest) = content.split_at(name_end_pos);

    if !is_valid_identifier_start(name_part, false) {
        return None;
    }

    rest = rest.trim_start();
    let modifier = if rest.starts_with("is") && rest.chars().nth(2).is_some_and(char::is_whitespace) {
        rest = &rest[2..];
        WhereModifier::Is
    } else if rest.starts_with(':') {
        rest = &rest[1..];
        WhereModifier::Colon
    } else {
        return None;
    };

    let consumed_len = content.len() - rest.len();
    let type_part_start_pos = span.start.forward(consumed_len as u32);
    let type_part_span = Span::new(span.file_id, type_part_start_pos, span.end);

    let (type_string, _rest) = split_tag_content(rest, type_part_span)?;

    Some(WhereTag { span, name: name_part.to_owned(), modifier, type_string })
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
    let raw_name = rest_parts.next()?;
    let variable = parse_var_ident(raw_name)?;

    let desc_start = rest_slice.find(&variable.name).map_or(0, |i| i + variable.name.len());
    let description = rest_slice[desc_start..].trim_start().to_owned();

    Some(ParameterTag { span, variable, type_string, description })
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

    let raw_name = rest_slice.split_whitespace().next()?;
    let variable = parse_var_ident(raw_name)?;

    Some(ParameterOutTag { span, variable, type_string })
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

    let raw_name = rest_parts.next()?;
    let variable = parse_var_ident(raw_name)?;

    Some(AssertionTag { span, variable, type_string })
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

    let variable = if rest_slice.is_empty() {
        None
    } else {
        let var_part = rest_slice.split_whitespace().next()?;
        parse_var_ident(var_part)
    };

    Some(VarTag { span, type_string, variable })
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

    let (type_string, _) = split_tag_content(&rest[1..], span.subspan(equals_index as u32, 0))?;

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
    let trimmed_start_pos = input_span.start.forward(trim_start_offset as u32);

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

        if char.is_whitespace() || char == '.' {
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
            let type_span = Span::new(
                input_span.file_id,
                trimmed_start_pos,
                trimmed_start_pos.forward(type_part_slice.len() as u32),
            );

            Some((TypeString { value: type_part_slice.to_owned(), span: type_span }, rest_part_slice))
        }
        None => {
            // No split, entire trimmed content is the type
            let type_part_slice = trimmed_content;
            let type_span = Span::new(
                input_span.file_id,
                trimmed_start_pos,
                trimmed_start_pos.forward(type_part_slice.len() as u32),
            );

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
    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;

    use super::*;

    fn test_span(input: &str, start_offset: u32) -> Span {
        let base_start = Position::new(start_offset);
        Span::new(FileId::zero(), base_start, base_start.forward(input.len() as u32))
    }

    fn test_span_for(s: &str) -> Span {
        test_span(s, 0)
    }

    fn make_span(start: u32, end: u32) -> Span {
        Span::new(FileId::zero(), Position::new(start), Position::new(end))
    }

    #[test]
    fn test_parse_var_ident() {
        struct Expect<'a> {
            s: &'a str,
            variadic: bool,
            by_ref: bool,
        }
        let cases: &[(&str, Option<Expect>)] = &[
            ("$x", Some(Expect { s: "$x", variadic: false, by_ref: false })),
            ("&$refVar", Some(Expect { s: "$refVar", variadic: false, by_ref: true })),
            ("$foo,", Some(Expect { s: "$foo", variadic: false, by_ref: false })),
            ("...$ids)", Some(Expect { s: "$ids", variadic: true, by_ref: false })),
            ("...$items,", Some(Expect { s: "$items", variadic: true, by_ref: false })),
            ("$", None),
            ("...$", None),
            ("$1x", None),
            ("foo", None),
        ];

        for (input, expected) in cases {
            let got = parse_var_ident(input);
            match (got, expected) {
                (None, None) => {}
                (Some(v), Some(e)) => {
                    assert_eq!(v.name, e.s, "input={}", input);
                    assert_eq!(v.is_variadic, e.variadic, "input={}", input);
                    assert_eq!(v.is_by_reference, e.by_ref, "input={}", input);
                }
                _ => panic!("mismatch for input={}", input),
            }
        }
    }

    #[test]
    fn test_variable_display_and_raw() {
        let cases = vec![("$x", "$x"), ("&$x", "&$x"), ("...$x", "...$x"), ("...$x)", "...$x"), ("...$x,", "...$x")];

        for (input, expected_raw) in cases {
            let v = parse_var_ident(input).expect("should parse variable");
            assert_eq!(v.to_string(), expected_raw);
        }
    }

    #[test]
    fn test_splitter_brackets() {
        let input = "array<int, (string|bool)> desc";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<int, (string|bool)>");
        assert_eq!(ts.span, make_span(0, "array<int, (string|bool)>".len() as u32));
        assert_eq!(rest, "desc");

        let input = "array<int, string> desc";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<int, string>");
        assert_eq!(ts.span, make_span(0, "array<int, string>".len() as u32));
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
        assert_eq!(ts.span, make_span(1, "'inside quote'".len() as u32 + 1));
        assert_eq!(rest, "outside");

        let input = r#""string \" with escape" $var"#;
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, r#""string \" with escape""#);
        assert_eq!(ts.span, make_span(0, r#""string \" with escape""#.len() as u32));
        assert_eq!(rest, "$var");

        assert!(split_tag_content("\"unterminated", test_span_for("\"unterminated")).is_none());
    }

    #[test]
    fn test_splitter_comments() {
        let input = "(string // comment \n | int) $var";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "(string // comment \n | int)");
        assert_eq!(ts.span, make_span(0, "(string // comment \n | int)".len() as u32));
        assert_eq!(rest, "$var");

        let input = "string // comment goes to end";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "string");
        assert_eq!(ts.span, make_span(0, "string".len() as u32));
        assert_eq!(rest, "// comment goes to end");

        let input = "array<string // comment\n> $var";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<string // comment\n>");
        assert_eq!(ts.span, make_span(0, "array<string // comment\n>".len() as u32));
        assert_eq!(rest, "$var");
    }

    #[test]
    fn test_splitter_whole_string_is_type() {
        let input = " array<int, string> ";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "array<int, string>");
        assert_eq!(ts.span, make_span(1, "array<int, string>".len() as u32 + 1));
        assert_eq!(rest, ""); // No rest part
    }

    #[test]
    fn test_splitter_with_dot() {
        let input = "string[]. something";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "string[]");
        assert_eq!(ts.span, make_span(0, "string[]".len() as u32));
        assert_eq!(rest, ". something");
    }

    #[test]
    fn test_param_basic() {
        let offset = 10;
        let content = " string|int $myVar Description here ";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();

        assert_eq!(result.type_string.value, "string|int"); // Check owned string value
        assert_eq!(result.type_string.span.start.offset, offset + 1); // Span of type part
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "string|int".len() as u32);
        assert_eq!(result.variable.name, "$myVar");
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
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, string>".len() as u32);
        assert_eq!(result.variable.name, "$param");
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
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "(string // comment \n | int)".len() as u32);
        assert_eq!(result.variable.name, "$var");
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
        let offset = 10u32;
        let content = " string Description here ";
        let span = test_span(content, offset);
        let result = parse_return_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, "string");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "string".len() as u32);
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
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, (string|null)>".len() as u32);
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
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, (string|null)>".len() as u32);
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
        assert_eq!(result.type_string.span.end.offset, 8 + "string".len() as u32);
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

    #[test]
    fn test_param_trailing_comma_is_ignored_in_name() {
        let content = " string $foo, desc";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, "$foo");
        assert_eq!(result.description, ", desc");
    }

    #[test]
    fn test_param_variadic_trailing_paren_is_ignored_in_name() {
        let content = " list<int> ...$items) rest";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, "$items");
        assert_eq!(result.description, ") rest");
    }

    #[test]
    fn test_param_out_trailing_comma() {
        let content = " int $out,";
        let span = test_span_for(content);
        let result = parse_param_out_tag(content, span).unwrap();
        assert_eq!(result.variable.name, "$out");
    }

    #[test]
    fn test_assertion_trailing_comma() {
        let content = " int $x,";
        let span = test_span_for(content);
        let result = parse_assertion_tag(content, span).unwrap();
        assert_eq!(result.variable.name, "$x");
    }

    #[test]
    fn test_param_trailing_without_space() {
        let content = " string $foo,desc";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, "$foo");
        assert_eq!(result.description, ",desc");
    }

    #[test]
    fn test_param_variadic_trailing_paren_without_space() {
        let content = " list<int> ...$items)more";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, "$items");
        assert_eq!(result.description, ")more");
    }
}
