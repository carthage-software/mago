use crate::error::ParseError;
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Method {
    pub visibility: Visibility,
    pub is_static: bool,
    pub name: String,
    pub argument_list: Vec<Argument>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Argument {
    pub type_hint: Option<TypeString>,
    pub variable: Variable,
    pub has_default: bool,
    pub argument_span: Span,
    pub variable_span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PropertyTag {
    pub span: Span,
    pub type_string: Option<TypeString>,
    pub variable: Variable,
    pub is_read: bool,
    pub is_write: bool,
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

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    pub type_string: Option<TypeString>,
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MethodTag {
    pub span: Span,
    pub method: Method,
    pub type_string: TypeString,
    pub description: String,
}
/// Parses a `PHPDoc` variable token and returns a structured `Variable`.
///
/// If `allow_property_access` is false:
/// - Supports `$name`, `...$name`, and `&$name`.
///
/// If `allow_property_access` is true:
/// - Supports `$name` with optional property/array access like `$foo->bar` or `$foo['key']`
/// - Can be recursive: `$foo->bar->baz['key']->qux`
/// - Does NOT support `...` (variadic) or `&` (reference) prefixes
///
/// The returned `Variable` stores a normalized `name` (with `$`, without leading `...` or `&`),
/// and sets flags `is_variadic` and `is_by_reference` that can be used for display/rendering.
///
/// Examples (`allow_property_access` = false):
/// - "$foo"       → Some(Variable { name: "$foo", `is_variadic`: false, `is_by_reference`: false })
/// - "&$foo"      → Some(Variable { name: "$foo", `is_variadic`: false, `is_by_reference`: true })
/// - "...$ids"    → Some(Variable { name: "$ids", `is_variadic`: true, `is_by_reference`: false })
/// - "$"          → None
/// - "...$"       → None
/// - "$1x"        → None
///
/// Examples (`allow_property_access` = true):
/// - "$foo->bar"     → Some(Variable { name: "$foo->bar", `is_variadic`: false, `is_by_reference`: false })
/// - "$foo['key']"   → Some(Variable { name: "$foo['key']", `is_variadic`: false, `is_by_reference`: false })
/// - "$foo->bar->baz['key']" → Some(Variable { name: "$foo->bar->baz['key']", `is_variadic`: false, `is_by_reference`: false })
/// - "&$foo->bar"    → None (reference not allowed with property access)
/// - "...$foo->bar"  → None (variadic not allowed with property access)
#[inline]
fn parse_var_ident(raw: &str, allow_property_access: bool) -> Option<Variable> {
    if allow_property_access {
        // When property access is allowed, we don't support & or ...
        if raw.starts_with('&') || raw.starts_with("...") {
            return None;
        }

        // Must start with $
        if !raw.starts_with('$') {
            return None;
        }

        let rest = &raw[1..]; // Skip the $
        let bytes = rest.as_bytes();

        if bytes.is_empty() {
            return None;
        }

        // Parse the initial identifier
        let is_start = |b: u8| b == b'_' || b.is_ascii_alphabetic();
        let is_cont = |b: u8| is_start(b) || b.is_ascii_digit();

        if !is_start(bytes[0]) {
            return None;
        }

        let mut pos = 1;
        while pos < bytes.len() && is_cont(bytes[pos]) {
            pos += 1;
        }

        // Now parse any property/array access chains
        while pos < bytes.len() {
            if pos + 1 < bytes.len() && &bytes[pos..pos + 2] == b"->" {
                // Object property access: ->identifier
                pos += 2; // Skip ->

                if pos >= bytes.len() || !is_start(bytes[pos]) {
                    return None; // Invalid: -> must be followed by valid identifier
                }

                pos += 1;
                while pos < bytes.len() && is_cont(bytes[pos]) {
                    pos += 1;
                }
            } else if bytes[pos] == b'[' {
                // Array access: [...]
                pos += 1; // Skip [
                let mut bracket_depth = 1;

                while pos < bytes.len() && bracket_depth > 0 {
                    if bytes[pos] == b'[' {
                        bracket_depth += 1;
                    } else if bytes[pos] == b']' {
                        bracket_depth -= 1;
                    }
                    pos += 1;
                }

                if bracket_depth != 0 {
                    return None; // Unmatched brackets
                }
            } else {
                // End of valid property access chain
                break;
            }
        }

        // The full token should be consumed for a valid property access chain
        let token = &raw[..=pos]; // Include the initial $

        Some(Variable { name: token.to_owned(), is_variadic: false, is_by_reference: false })
    } else {
        // Original logic for when property access is not allowed
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
/// # Errors
///
/// Returns a [`ParseError`] if the template tag syntax is invalid.
#[inline]
pub fn parse_template_tag(
    content: &str,
    span: Span,
    mut covariant: bool,
    mut contravariant: bool,
) -> Result<TemplateTag, ParseError> {
    // Find start offset of trimmed content relative to original `content`
    let trim_start_offset_rel = content.find(|c: char| !c.is_whitespace()).unwrap_or(0);
    let trimmed_content = content.trim();

    if trimmed_content.is_empty() {
        return Err(ParseError::InvalidTemplateTag(span, "Expected template parameter name".to_string()));
    }

    let mut parts = trimmed_content.split_whitespace();

    let mut name_part = parts
        .next()
        .ok_or_else(|| ParseError::InvalidTemplateTag(span, "Expected template parameter name".to_string()))?;
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

    Ok(TemplateTag { span, name, modifier, type_string: type_string_opt, covariant, contravariant })
}

/// Parses the content string of a `@where` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@where`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the where tag syntax is invalid.
pub fn parse_where_tag(content: &str, span: Span) -> Result<WhereTag, ParseError> {
    let name_end_pos = content.find(char::is_whitespace).ok_or_else(|| {
        ParseError::InvalidWhereTag(span, "Expected template parameter name and constraint".to_string())
    })?;
    let (name_part, mut rest) = content.split_at(name_end_pos);

    if !is_valid_identifier_start(name_part, false) {
        return Err(ParseError::InvalidWhereTag(span, format!("Invalid template parameter name: '{name_part}'")));
    }

    rest = rest.trim_start();
    let modifier = if rest.starts_with("is") && rest.chars().nth(2).is_some_and(char::is_whitespace) {
        rest = &rest[2..];
        WhereModifier::Is
    } else if rest.starts_with(':') {
        rest = &rest[1..];
        WhereModifier::Colon
    } else {
        return Err(ParseError::InvalidWhereTag(
            span,
            "Expected 'is' or ':' after template parameter name".to_string(),
        ));
    };

    let consumed_len = content.len() - rest.len();
    let type_part_start_pos = span.start.forward(consumed_len as u32);
    let type_part_span = Span::new(span.file_id, type_part_start_pos, span.end);

    let (type_string, _rest) = split_tag_content(rest, type_part_span)
        .ok_or_else(|| ParseError::InvalidWhereTag(span, "Failed to parse type constraint".to_string()))?;

    Ok(WhereTag { span, name: name_part.to_owned(), modifier, type_string })
}

/// Parses the content string of a `@param` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@param`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the param tag syntax is invalid.
pub fn parse_param_tag(content: &str, span: Span) -> Result<ParameterTag, ParseError> {
    let trimmed = content.trim_start();

    // Check if content starts with a variable (no type specified)
    if trimmed.starts_with('$') {
        // No type specified, just parse variable and description
        let mut parts = trimmed.split_whitespace();
        let raw_name =
            parts.next().ok_or_else(|| ParseError::InvalidParameterTag(span, "Expected parameter name".to_string()))?;

        let variable = parse_var_ident(raw_name, false)
            .ok_or_else(|| ParseError::InvalidParameterTag(span, format!("Invalid parameter name: '{raw_name}'")))?;

        let desc_start = trimmed.find(&variable.name).map_or(0, |i| i + variable.name.len());
        let description = trimmed[desc_start..].trim().to_owned();

        return Ok(ParameterTag { span, variable, type_string: None, description });
    }

    // Type is specified, parse it
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidParameterTag(span, "Failed to parse parameter type".to_string()))?;

    // Type must be valid (not empty, not starting with { or $)
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return Err(ParseError::InvalidParameterTag(span, format!("Invalid parameter type: '{}'", type_string.value)));
    }

    if rest_slice.is_empty() {
        // Variable name is mandatory
        return Err(ParseError::InvalidParameterTag(span, "Missing parameter name".to_string()));
    }

    let mut rest_parts = rest_slice.split_whitespace();
    let raw_name = rest_parts
        .next()
        .ok_or_else(|| ParseError::InvalidParameterTag(span, "Expected parameter name".to_string()))?;
    let variable = parse_var_ident(raw_name, false)
        .ok_or_else(|| ParseError::InvalidParameterTag(span, format!("Invalid parameter name: '{raw_name}'")))?;

    let desc_start = rest_slice.find(&variable.name).map_or(0, |i| i + variable.name.len());
    let description = rest_slice[desc_start..].trim_start().to_owned();

    Ok(ParameterTag { span, variable, type_string: Some(type_string), description })
}

/// Parses the content string of a `@param-out` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@param-out`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the param-out tag syntax is invalid.
pub fn parse_param_out_tag(content: &str, span: Span) -> Result<ParameterOutTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidParameterOutTag(span, "Failed to parse parameter type".to_string()))?;

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return Err(ParseError::InvalidParameterOutTag(
            span,
            format!("Invalid parameter type: '{}'", type_string.value),
        ));
    }

    if rest_slice.is_empty() {
        return Err(ParseError::InvalidParameterOutTag(span, "Missing parameter name".to_string()));
    }

    let raw_name = rest_slice
        .split_whitespace()
        .next()
        .ok_or_else(|| ParseError::InvalidParameterOutTag(span, "Expected parameter name".to_string()))?;
    let variable = parse_var_ident(raw_name, false)
        .ok_or_else(|| ParseError::InvalidParameterOutTag(span, format!("Invalid parameter name: '{raw_name}'")))?;

    Ok(ParameterOutTag { span, variable, type_string })
}

/// Parses the content string of a `@return` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@return`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the return tag syntax is invalid.
pub fn parse_return_tag(content: &str, span: Span) -> Result<ReturnTypeTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidReturnTag(span, "Failed to parse return type".to_string()))?;

    // Type cannot start with '{'
    if type_string.value.starts_with('{') {
        return Err(ParseError::InvalidReturnTag(span, format!("Invalid return type: '{}'", type_string.value)));
    }

    let description = rest_slice.to_owned();

    Ok(ReturnTypeTag { span, type_string, description })
}

/// Parses the content string of a `@throws` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@throws`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the throws tag syntax is invalid.
pub fn parse_throws_tag(content: &str, span: Span) -> Result<ThrowsTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidThrowsTag(span, "Failed to parse exception type".to_string()))?;

    // Type cannot start with '{'
    if type_string.value.starts_with('{') {
        return Err(ParseError::InvalidThrowsTag(span, format!("Invalid exception type: '{}'", type_string.value)));
    }

    // Type cannot start with '$' unless it is "$this"
    if type_string.value.starts_with('$') && type_string.value != "$this" {
        return Err(ParseError::InvalidThrowsTag(span, format!("Invalid exception type: '{}'", type_string.value)));
    }

    let description = rest_slice.to_owned();

    Ok(ThrowsTag { span, type_string, description })
}

/// Parses the content string of an `@assert`, `@assert-if-true`, or `@assert-if-false` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the assertion tag syntax is invalid.
pub fn parse_assertion_tag(content: &str, span: Span) -> Result<AssertionTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidAssertionTag(span, "Failed to parse assertion type".to_string()))?;

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return Err(ParseError::InvalidAssertionTag(span, format!("Invalid assertion type: '{}'", type_string.value)));
    }

    if rest_slice.is_empty() {
        // Variable name is mandatory
        return Err(ParseError::InvalidAssertionTag(span, "Missing variable name".to_string()));
    }

    let mut rest_parts = rest_slice.split_whitespace();

    let raw_name =
        rest_parts.next().ok_or_else(|| ParseError::InvalidAssertionTag(span, "Expected variable name".to_string()))?;
    let variable = parse_var_ident(raw_name, true)
        .ok_or_else(|| ParseError::InvalidAssertionTag(span, format!("Invalid variable name: '{raw_name}'")))?;

    Ok(AssertionTag { span, type_string, variable })
}

/// Parses the content string of a `@var` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the var tag syntax is invalid.
pub fn parse_var_tag(content: &str, span: Span) -> Result<VarTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidVarTag(span, "Failed to parse variable type".to_string()))?;

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return Err(ParseError::InvalidVarTag(span, format!("Invalid variable type: '{}'", type_string.value)));
    }

    let variable = if rest_slice.is_empty() {
        None
    } else {
        let var_part = rest_slice
            .split_whitespace()
            .next()
            .ok_or_else(|| ParseError::InvalidVarTag(span, "Expected variable name".to_string()))?;
        parse_var_ident(var_part, true)
    };

    Ok(VarTag { span, type_string, variable })
}

/// Parses the content string of a `@type` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the type tag syntax is invalid.
pub fn parse_type_tag(content: &str, span: Span) -> Result<TypeTag, ParseError> {
    let leading_ws = (content.len() - content.trim_start().len()) as u32;
    let content = content.trim_start();

    if content.is_empty() {
        return Err(ParseError::InvalidTypeTag(span, "Type alias declaration is empty".to_string()));
    }

    let (potential_name, _) = content.split_once(char::is_whitespace).ok_or_else(|| {
        let trimmed = content.trim();
        ParseError::InvalidTypeTag(span, format!("Type alias name '{trimmed}' must be followed by a type definition"))
    })?;

    let name_len = potential_name.len();
    let after_name = &content[name_len..];
    let trimmed_after_name = after_name.trim_start();

    let (name, type_part, type_offset) = if let Some(after_equals) = trimmed_after_name.strip_prefix('=') {
        // Format: @type Name = Type
        let name = potential_name.trim();

        if !is_valid_identifier_start(name, false) {
            return Err(ParseError::InvalidTypeTag(span, format!("Invalid type alias name: '{name}'")));
        }

        let type_start_offset = name_len + (after_name.len() - trimmed_after_name.len()) + 1;

        (name, after_equals, leading_ws + type_start_offset as u32)
    } else {
        let name = potential_name.trim();

        if !is_valid_identifier_start(name, false) {
            return Err(ParseError::InvalidTypeTag(span, format!("Invalid type alias name: '{name}'")));
        }

        let rest = after_name.trim_start();
        let type_start_offset = name_len + (after_name.len() - rest.len());

        (name, rest, leading_ws + type_start_offset as u32)
    };

    let (type_string, _) = split_tag_content(type_part, span.subspan(type_offset, 0))
        .ok_or_else(|| ParseError::InvalidTypeTag(span, "Failed to parse type definition".to_string()))?;

    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return Err(ParseError::InvalidTypeTag(span, format!("Invalid type definition: '{}'", type_string.value)));
    }

    Ok(TypeTag { span, name: name.to_owned(), type_string })
}

/// Parses the content string of an `@import-type` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following the tag.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the import-type tag syntax is invalid.
pub fn parse_import_type_tag(content: &str, span: Span) -> Result<ImportTypeTag, ParseError> {
    let (name, rest) = content.trim_start().split_once(' ').ok_or_else(|| {
        ParseError::InvalidImportTypeTag(span, "Expected type alias name and 'from' clause".to_string())
    })?;
    let name = name.trim();
    let rest = rest.trim();

    if !is_valid_identifier_start(name, false) {
        return Err(ParseError::InvalidImportTypeTag(span, format!("Invalid type alias name: '{name}'")));
    }

    if rest.is_empty() {
        return Err(ParseError::InvalidImportTypeTag(span, "Missing 'from' clause".to_string()));
    }

    let (from, rest) = rest.split_once(' ').ok_or_else(|| {
        ParseError::InvalidImportTypeTag(span, "Expected 'from' keyword followed by class name".to_string())
    })?;

    if !from.eq_ignore_ascii_case("from") {
        return Err(ParseError::InvalidImportTypeTag(span, format!("Expected 'from' keyword, found '{from}'")));
    }

    if rest.is_empty() {
        return Err(ParseError::InvalidImportTypeTag(span, "Missing class name after 'from'".to_string()));
    }

    let (imported_from, rest) = if let Some((imp_from, rest)) = rest.split_once(' ') {
        (imp_from.trim(), rest.trim())
    } else {
        (rest.trim(), "")
    };

    if !is_valid_identifier_start(imported_from, true) {
        return Err(ParseError::InvalidImportTypeTag(span, format!("Invalid class name: '{imported_from}'")));
    }

    let mut alias = None;

    if let Some((r#as, rest)) = rest.split_once(' ')
        && r#as.trim().eq_ignore_ascii_case("as")
        && !rest.is_empty()
    {
        let alias_name = rest
            .split_whitespace()
            .next()
            .ok_or_else(|| ParseError::InvalidImportTypeTag(span, "Expected alias name after 'as'".to_string()))?
            .trim()
            .to_owned();
        alias = Some(alias_name);
    }

    Ok(ImportTypeTag { span, name: name.to_owned(), from: imported_from.to_owned(), alias })
}

/// Parses the content string of a `@property` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] if the property tag syntax is invalid.
pub fn parse_property_tag(content: &str, span: Span, is_read: bool, is_write: bool) -> Result<PropertyTag, ParseError> {
    // If we are at `$` and not `$this`, then no type is present:
    let (type_string, variable) = if content.trim_start().starts_with('$') && !content.trim_start().starts_with("$this")
    {
        let var_part = content
            .split_whitespace()
            .next()
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, "Expected variable name".to_string()))?;
        let variable = parse_var_ident(var_part, false)
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, format!("Invalid variable name: '{var_part}'")))?;

        (None, variable)
    } else {
        let (type_string, rest_slice) = split_tag_content(content, span)
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, "Failed to parse type definition".to_string()))?;

        // Type must exist and be valid
        if type_string.value.is_empty()
            || type_string.value.starts_with('{')
            || (type_string.value.starts_with('$') && type_string.value != "$this")
        {
            return Err(ParseError::InvalidPropertyTag(
                span,
                format!("Invalid type definition: '{}'", type_string.value),
            ));
        }

        if rest_slice.is_empty() {
            return Err(ParseError::InvalidPropertyTag(span, "Missing variable name after type".to_string()));
        }

        let var_part = rest_slice
            .split_whitespace()
            .next()
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, "Expected variable name".to_string()))?;
        let variable = parse_var_ident(var_part, false)
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, format!("Invalid variable name: '{var_part}'")))?;

        (Some(type_string), variable)
    };

    Ok(PropertyTag { span, type_string, variable, is_read, is_write })
}

/// Splits tag content into the type string part and the rest, respecting brackets/quotes.
/// Calculates the absolute span of the identified type string.
///
/// Returns None if parsing fails or input is empty.
///
/// Output: `Some((TypeString, rest_slice))` or `None`
#[inline]
#[must_use]
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
                    Some(opening) if brackets_match(opening, char) => {}
                    _ => return None, // Mismatch or unbalanced
                }
            }
            _ => {}
        }

        // if we are at `:`, `|`, or `&` then consider it significant and consume following
        // whitespaces, and continue processing
        // This allows union/intersection types like `int | string` or `Foo & Bar`
        // as well as callable return types like `callable(): int`
        if char == ':' || char == '|' || char == '&' {
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
                while let Some(&(_, next_char)) = iter.peek() {
                    if next_char == '\n' {
                        break;
                    }

                    iter.next();
                }
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
                let mut temp_iter = iter.clone();
                let mut found_continuation = false;

                while let Some(&(_, next_char)) = temp_iter.peek() {
                    if next_char.is_whitespace() {
                        temp_iter.next();
                    } else {
                        found_continuation = next_char == ':'
                            || next_char == '|'
                            || (next_char == '&' && {
                                temp_iter.next(); // consume '&'
                                !temp_iter.peek().is_some_and(|&(_, c)| c == '$' || c == '.')
                            });
                        break;
                    }
                }

                if found_continuation {
                    while let Some(&(_, next_char)) = iter.peek() {
                        if next_char.is_whitespace() {
                            iter.next();
                        } else {
                            break;
                        }
                    }

                    last_char_was_significant = true;
                } else {
                    split_point_rel = Some(i);
                    break;
                }
            } else {
                last_char_was_significant = false;
            }
        } else if char == '.' {
            // Only treat '.' as a split point if it's NOT part of a numeric literal
            // Check if this is a numeric literal by looking at surrounding chars
            let prev_is_digit = i > 0 && trimmed_content.as_bytes()[i - 1].is_ascii_digit();
            let next_is_digit = iter.peek().is_some_and(|&(_, c)| c.is_ascii_digit());

            if prev_is_digit && next_is_digit {
                // This is part of a numeric literal like "24.0"
                last_char_was_significant = true;
            } else {
                // This is a description separator like "string[]. something"
                if bracket_stack.is_empty() && last_char_was_significant {
                    split_point_rel = Some(i);
                    break;
                }
                last_char_was_significant = false;
            }
        } else {
            last_char_was_significant = true;
        }
    }

    // After loop checks
    if !bracket_stack.is_empty() || quote_char.is_some() {
        return None;
    }

    if let Some(split_idx_rel) = split_point_rel {
        // Split occurred
        let type_part_slice = trimmed_content[..split_idx_rel].trim_end();
        let rest_part_slice = trimmed_content[split_idx_rel..].trim_start();

        // Calculate span relative to the *start* of the trimmed content
        let type_span =
            Span::new(input_span.file_id, trimmed_start_pos, trimmed_start_pos.forward(type_part_slice.len() as u32));

        Some((TypeString { value: type_part_slice.to_owned(), span: type_span }, rest_part_slice))
    } else {
        // No split, entire trimmed content is the type
        let type_part_slice = trimmed_content;
        let type_span =
            Span::new(input_span.file_id, trimmed_start_pos, trimmed_start_pos.forward(type_part_slice.len() as u32));

        Some((TypeString { value: type_part_slice.to_owned(), span: type_span }, ""))
    }
}

/// Parses the content string of a `@method` tag.
///
/// # Arguments
///
/// * `content` - The string slice content following `@method`.
/// * `span` - The original `Span` of the `content` slice.
///
/// # Errors
///
/// Returns a [`ParseError`] if the method tag syntax is invalid.
pub fn parse_method_tag(mut content: &str, mut span: Span) -> Result<MethodTag, ParseError> {
    let (trimmed_content, leading_ws) = consume_whitespace(content);
    content = trimmed_content;
    span = span.subspan(leading_ws as u32, span.length());

    let mut is_static = false;
    let mut visibility = None;

    let mut acc_len = 0;

    // Track the position and length of the static modifier, in case we need to treat it as return type
    let mut static_modifier_start = 0u32;
    let mut static_modifier_len = 0u32;

    loop {
        if let Some((new_content, char_count)) = try_consume(content, "static ") {
            if is_static {
                return Err(ParseError::InvalidMethodTag(span, "Duplicate 'static' modifier".to_string()));
            }

            is_static = true;
            static_modifier_start = acc_len as u32;
            static_modifier_len = 6; // "static" without the space
            acc_len += char_count;
            content = new_content;
        } else if let Some((new_content, char_count)) = try_consume(content, "public ") {
            if visibility.is_some() {
                return Err(ParseError::InvalidMethodTag(span, "Duplicate visibility modifier".to_string()));
            }

            visibility = Some(Visibility::Public);
            acc_len += char_count;
            content = new_content;
        } else if let Some((new_content, char_count)) = try_consume(content, "protected ") {
            if visibility.is_some() {
                return Err(ParseError::InvalidMethodTag(span, "Duplicate visibility modifier".to_string()));
            }

            visibility = Some(Visibility::Protected);
            acc_len += char_count;
            content = new_content;
        } else if let Some((new_content, char_count)) = try_consume(content, "private ") {
            if visibility.is_some() {
                return Err(ParseError::InvalidMethodTag(span, "Duplicate visibility modifier".to_string()));
            }

            visibility = Some(Visibility::Private);
            acc_len += char_count;
            content = new_content;
        } else {
            break;
        }
    }

    let rest_span = span.subspan(acc_len as u32, span.length());

    let (type_string, rest_slice, rest_slice_span) = if is_static && looks_like_method_signature_only(content) {
        is_static = false;
        let static_span = span.subspan(static_modifier_start, static_modifier_start + static_modifier_len);
        let type_string = TypeString { value: "static".into(), span: static_span };
        let (rest_slice, whitespace_count) = consume_whitespace(content);
        let rest_slice_span = rest_span.subspan(whitespace_count as u32, rest_span.length());
        (type_string, rest_slice, rest_slice_span)
    } else {
        let type_string = split_tag_content(content, rest_span)
            .ok_or_else(|| ParseError::InvalidMethodTag(span, "Failed to parse return type".to_string()))?
            .0;
        let (rest_slice, whitespace_count) = consume_whitespace(&content[type_string.span.length() as usize..]);
        let rest_slice_span =
            rest_span.subspan(type_string.span.length() + whitespace_count as u32, rest_span.length());
        (type_string, rest_slice, rest_slice_span)
    };

    // Type must exist and be valid
    if type_string.value.is_empty()
        || type_string.value.starts_with('{')
        || (type_string.value.starts_with('$') && type_string.value != "$this")
    {
        return Err(ParseError::InvalidMethodTag(span, format!("Invalid return type: '{}'", type_string.value)));
    }

    if rest_slice.is_empty() {
        // Method definition is mandatory
        return Err(ParseError::InvalidMethodTag(span, "Missing method signature".to_string()));
    }

    let mut chars = rest_slice.char_indices().peekable();

    let mut name_end = None;

    for (i, ch) in &mut chars {
        if ch == '(' {
            name_end = Some(i);
            break;
        }
    }

    let name_end = name_end.ok_or_else(|| {
        ParseError::InvalidMethodTag(span, "Missing opening parenthesis '(' for method arguments".to_string())
    })?;

    let name = rest_slice[..name_end].trim();

    if name.is_empty() {
        return Err(ParseError::InvalidMethodTag(span, "Missing method name".to_string()));
    }

    let mut depth = 1;
    let mut args_end = None;

    for (i, ch) in &mut chars {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    args_end = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let args_end = args_end.ok_or_else(|| {
        ParseError::InvalidMethodTag(span, "Missing closing parenthesis ')' for method arguments".to_string())
    })?;
    let (args_str, whitespace_count) = consume_whitespace(&rest_slice[name_end + 1..args_end]);
    let args_span = rest_slice_span.subspan((whitespace_count + name_end) as u32 + 1, args_end as u32);

    let description = rest_slice[args_end..].trim();
    let arguments_split = split_args(args_str, args_span);
    let arguments = arguments_split.iter().filter_map(|(arg, span)| parse_argument(arg, span)).collect::<Vec<_>>();

    let method = Method {
        name: name.into(),
        argument_list: arguments,
        visibility: visibility.unwrap_or(Visibility::Public),
        is_static,
    };

    Ok(MethodTag { span, type_string, method, description: description.into() })
}

fn consume_whitespace(input: &str) -> (&str, usize) {
    let mut iter = input.chars().peekable();
    let mut count = 0;

    while let Some(ch) = iter.peek() {
        if ch.is_whitespace() {
            iter.next();
            count += 1;
        } else {
            break;
        }
    }

    (&input[count..], count)
}

fn try_consume<'a>(input: &'a str, token: &str) -> Option<(&'a str, usize)> {
    let (input, whitespace_count) = consume_whitespace(input);

    if !input.starts_with(token) {
        return None;
    }

    let len = token.len() + whitespace_count;
    let input = &input[len..];

    let (input, whitespace_count) = consume_whitespace(input);

    Some((input, len + whitespace_count))
}

/// Checks if the given content looks like only a method signature (no return type).
/// e.g., "`foo()`" or "foo($arg)" returns true
/// e.g., "Money `foo()`" or "int bar($x)" returns false
fn looks_like_method_signature_only(content: &str) -> bool {
    let trimmed = content.trim();
    if let Some(paren_pos) = trimmed.find('(') {
        let before_paren = trimmed[..paren_pos].trim();
        !before_paren.is_empty() && !before_paren.contains(' ')
    } else {
        false
    }
}

fn split_args(args_str: &str, span: Span) -> Vec<(&str, Span)> {
    let mut args = Vec::new();

    let mut start = 0;
    let mut depth = 0;
    for (i, ch) in args_str.char_indices() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                let (arg, whitespace_count) = consume_whitespace(&args_str[start..i]);
                if !arg.is_empty() {
                    args.push((arg, span.subspan((whitespace_count + start) as u32, i as u32)));
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < args_str.len() {
        let (arg, whitespace_count) = consume_whitespace(&args_str[start..]);
        let arg_trimmed = arg.trim_end();
        if !arg.is_empty() {
            args.push((
                arg_trimmed,
                span.subspan(
                    (whitespace_count + start) as u32,
                    (args_str.len() - arg.len() + arg_trimmed.len()) as u32,
                ),
            ));
        }
    }

    args
}

fn parse_argument(arg_str: &str, span: &Span) -> Option<Argument> {
    let default_value_split = arg_str.rsplit_once('=');

    let ((arg_type, raw_name), default_value): ((_, _), Option<&str>) =
        if let Some((variable_definition, default_value)) = default_value_split {
            let arg = variable_definition.trim();
            if let Some((arg_type, raw_name)) = arg.rsplit_once(' ') {
                ((Some(arg_type), raw_name), Some(default_value.trim()))
            } else {
                ((None, arg), Some(default_value))
            }
        } else {
            let arg = arg_str.trim();
            if let Some((arg_type, raw_name)) = arg.rsplit_once(' ') {
                ((Some(arg_type), raw_name), None)
            } else {
                ((None, arg), None)
            }
        };

    let type_string =
        arg_type.map(|arg_type| TypeString { value: arg_type.into(), span: span.subspan(0, arg_type.len() as u32) });

    let variable_span = span.subspan(arg_type.map_or(0, |t| 1 + t.len() as u32), span.length());

    let variable = parse_var_ident(raw_name, false)?;

    Some(Argument {
        type_hint: type_string,
        variable,
        has_default: default_value.is_some(),
        argument_span: *span,
        variable_span,
    })
}

/// Checks if an opening bracket matches a closing one.
#[inline]
const fn brackets_match(open: char, close: char) -> bool {
    matches!((open, close), ('<', '>') | ('(', ')') | ('[', ']') | ('{', '}'))
}

/// Checks if the identifier is valid
#[inline]
fn is_valid_identifier_start(mut identifier: &str, allow_qualified: bool) -> bool {
    if allow_qualified && identifier.starts_with('\\') {
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
            let got = parse_var_ident(input, false);
            match (got, expected) {
                (None, None) => {}
                (Some(v), Some(e)) => {
                    assert_eq!(v.name, e.s, "input={input}");
                    assert_eq!(v.is_variadic, e.variadic, "input={input}");
                    assert_eq!(v.is_by_reference, e.by_ref, "input={input}");
                }
                _ => panic!("mismatch for input={input}"),
            }
        }
    }

    #[test]
    fn test_variable_display_and_raw() {
        let cases = vec![("$x", "$x"), ("&$x", "&$x"), ("...$x", "...$x"), ("...$x)", "...$x"), ("...$x,", "...$x")];

        for (input, expected_raw) in cases {
            let v = parse_var_ident(input, false).expect("should parse variable");
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

        assert_eq!(result.type_string.as_ref().unwrap().value, "string|int"); // Check owned string value
        assert_eq!(result.type_string.as_ref().unwrap().span.start.offset, offset + 1); // Span of type part
        assert_eq!(result.type_string.as_ref().unwrap().span.end.offset, offset + 1 + "string|int".len() as u32);
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
        assert_eq!(result.type_string.as_ref().unwrap().value, "array<int, string>"); // Check owned string
        assert_eq!(result.type_string.as_ref().unwrap().span.start.offset, offset + 1);
        assert_eq!(
            result.type_string.as_ref().unwrap().span.end.offset,
            offset + 1 + "array<int, string>".len() as u32
        );
        assert_eq!(result.variable.name, "$param");
        assert_eq!(result.description, "");
    }

    #[test]
    fn test_param_type_with_comment() {
        let offset = 20;
        let content = " (string // comment \n | int) $var desc";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.as_ref().unwrap().value, "(string // comment \n | int)");
        assert_eq!(result.type_string.as_ref().unwrap().span.start.offset, offset + 1);
        assert_eq!(
            result.type_string.as_ref().unwrap().span.end.offset,
            offset + 1 + "(string // comment \n | int)".len() as u32
        );
        assert_eq!(result.variable.name, "$var");
        assert_eq!(result.description, "desc");
    }

    #[test]
    fn test_param_no_type() {
        let content = " $param Description here ";
        let span = test_span(content, 0);
        let result = parse_param_tag(content, span).unwrap();
        assert!(result.type_string.is_none()); // No type specified
        assert_eq!(result.variable.name, "$param");
        assert_eq!(result.description, "Description here");
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
        assert!(parse_param_out_tag(content, span).is_err());
    }

    #[test]
    fn test_param_out_no_var() {
        let content = " string ";
        let span = test_span(content, 0);
        assert!(parse_param_out_tag(content, span).is_err());
    }

    #[test]
    fn test_type() {
        let content = "MyType = string";
        let span = test_span_for(content);
        let result = parse_type_tag(content, span).unwrap();
        assert_eq!(result.name, "MyType");
        assert_eq!(result.type_string.value, "string");
        assert_eq!(result.type_string.span.start.offset, 9);
        assert_eq!(result.type_string.span.end.offset, 9 + "string".len() as u32);
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

    #[test]
    fn test_param_with_numeric_literals_in_union() {
        let content = "-1|-24.0|string $a";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.as_ref().unwrap().value, "-1|-24.0|string");
        assert_eq!(result.variable.name, "$a");
        assert_eq!(result.description, "");
    }

    #[test]
    fn test_param_with_float_literals() {
        let content = "1.5|2.0|3.14 $value";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.as_ref().unwrap().value, "1.5|2.0|3.14");
        assert_eq!(result.variable.name, "$value");
    }

    #[test]
    fn test_splitter_with_dot_still_works_as_separator() {
        // Ensure we didn't break the original use case where . separates description
        let input = "string[]. something else";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "string[]");
        assert_eq!(rest, ". something else");
    }

    #[test]
    fn test_splitter_with_colon_after_whitespace() {
        let input = "callable(string)    :         string     $callback";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, "callable(string)    :         string");
        assert_eq!(rest, "$callback");

        let input2 = "callable(string) : string $callback";
        let span2 = test_span_for(input2);
        let (ts2, rest2) = split_tag_content(input2, span2).unwrap();
        assert_eq!(ts2.value, "callable(string) : string");
        assert_eq!(rest2, "$callback");

        let input3 = "callable(string): string $callback";
        let span3 = test_span_for(input3);
        let (ts3, rest3) = split_tag_content(input3, span3).unwrap();
        assert_eq!(ts3.value, "callable(string): string");
        assert_eq!(rest3, "$callback");
    }
}
