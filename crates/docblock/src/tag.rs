use crate::error::ParseError;
use mago_span::Span;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variable {
    pub name: Vec<u8>,
    pub is_variadic: bool,
    pub is_by_reference: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Method {
    pub visibility: Visibility,
    pub is_static: bool,

    pub name: Vec<u8>,
    pub argument_list: Vec<Argument>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Argument {
    pub type_hint: Option<TypeString>,
    pub variable: Variable,
    pub has_default: bool,
    pub argument_span: Span,
    pub variable_span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        f.write_str(&String::from_utf8_lossy(&self.name))
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.name))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeString {
    pub value: Vec<u8>,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReturnTypeTag {
    pub span: Span,
    pub type_string: TypeString,

    pub description: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeTag {
    pub span: Span,

    pub name: Vec<u8>,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportTypeTag {
    pub span: Span,

    pub name: Vec<u8>,

    pub from: Vec<u8>,
    pub alias: Option<ByteAlias>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ByteAlias(pub Vec<u8>);

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterTag {
    pub span: Span,
    pub variable: Variable,
    pub type_string: Option<TypeString>,

    pub description: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterOutTag {
    pub span: Span,
    pub variable: Variable,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThrowsTag {
    pub span: Span,
    pub type_string: TypeString,

    pub description: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum TemplateModifier {
    Of,
    As,
    Super,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateTag {
    pub span: Span,

    pub name: Vec<u8>,
    pub modifier: Option<TemplateModifier>,
    pub type_string: Option<TypeString>,
    pub default: Option<TypeString>,
    pub covariant: bool,
    pub contravariant: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum WhereModifier {
    Is,
    Colon,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhereTag {
    pub span: Span,

    pub name: Vec<u8>,
    pub modifier: WhereModifier,
    pub type_string: TypeString,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssertionTag {
    pub span: Span,
    pub type_string: TypeString,
    pub variable: Variable,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VarTag {
    pub span: Span,
    pub type_string: TypeString,
    pub variable: Option<Variable>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MethodTag {
    pub span: Span,
    pub method: Method,
    pub type_string: TypeString,

    pub description: Vec<u8>,
}

#[inline]
fn position_of(haystack: &[u8], needle: u8) -> Option<usize> {
    memchr::memchr(needle, haystack)
}

#[inline]
fn split_once_byte(haystack: &[u8], needle: u8) -> Option<(&[u8], &[u8])> {
    memchr::memchr(needle, haystack).map(|i| (&haystack[..i], &haystack[i + 1..]))
}

#[inline]
fn rsplit_once_byte(haystack: &[u8], needle: u8) -> Option<(&[u8], &[u8])> {
    memchr::memrchr(needle, haystack).map(|i| (&haystack[..i], &haystack[i + 1..]))
}

#[inline]
fn find_ascii_whitespace(haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|b| b.is_ascii_whitespace())
}

#[inline]
fn split_once_ascii_whitespace(haystack: &[u8]) -> Option<(&[u8], &[u8])> {
    find_ascii_whitespace(haystack).map(|i| (&haystack[..i], &haystack[i + 1..]))
}

/// First-byte predicate for a PHP variable name, matching PHP's
/// `[a-zA-Z_\x80-\xff]` rule. The `\x80-\xff` range is how PHP admits
/// multi-byte UTF-8 codepoints (e.g. `$café`, `$module🤔`).
#[inline]
const fn is_var_name_start(byte: u8) -> bool {
    matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'_' | 0x80..=0xFF)
}

/// Continuation-byte predicate for a PHP variable name, matching PHP's
/// `[a-zA-Z0-9_\x80-\xff]` rule.
#[inline]
const fn is_var_name_part(byte: u8) -> bool {
    matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | 0x80..=0xFF)
}

/// Parses a `PHPDoc` variable token and returns a structured `Variable`.
#[inline]
fn parse_var_ident(raw: &[u8], allow_property_access: bool) -> Option<Variable> {
    if allow_property_access {
        if raw.starts_with(b"&") || raw.starts_with(b"...") {
            return None;
        }

        if !raw.starts_with(b"$") {
            return None;
        }

        let rest = &raw[1..];

        if rest.is_empty() {
            return None;
        }

        if !is_var_name_start(rest[0]) {
            return None;
        }

        let mut pos = 1;
        while pos < rest.len() && is_var_name_part(rest[pos]) {
            pos += 1;
        }

        while pos < rest.len() {
            if pos + 1 < rest.len() && &rest[pos..pos + 2] == b"->" {
                pos += 2;

                if pos >= rest.len() || !is_var_name_start(rest[pos]) {
                    return None;
                }

                pos += 1;
                while pos < rest.len() && is_var_name_part(rest[pos]) {
                    pos += 1;
                }
            } else if rest[pos] == b'[' {
                pos += 1;
                let mut bracket_depth = 1;

                while pos < rest.len() && bracket_depth > 0 {
                    if rest[pos] == b'[' {
                        bracket_depth += 1;
                    } else if rest[pos] == b']' {
                        bracket_depth -= 1;
                    }
                    pos += 1;
                }

                if bracket_depth != 0 {
                    return None;
                }
            } else if rest[pos] == b'(' && rest.get(pos + 1).is_some_and(|b| *b == b')') {
                pos += 2;
                break;
            } else {
                break;
            }
        }

        let token = &raw[..=pos];

        Some(Variable { name: token.to_vec(), is_variadic: false, is_by_reference: false })
    } else {
        let is_by_reference = raw.starts_with(b"&");
        let raw = raw.strip_prefix(b"&").unwrap_or(raw);
        let (prefix_len, rest, is_variadic) = if let Some(r) = raw.strip_prefix(b"...$") {
            (4usize, r, true)
        } else {
            let r = raw.strip_prefix(b"$")?;
            (1usize, r, false)
        };

        if rest.is_empty() {
            return None;
        }

        if !is_var_name_start(rest[0]) {
            return None;
        }

        let mut len = 1usize;
        while len < rest.len() && is_var_name_part(rest[len]) {
            len += 1;
        }

        let token = &raw[..prefix_len + len];
        let normalized = if is_variadic { &token[3..] } else { token };
        Some(Variable { name: normalized.to_vec(), is_variadic, is_by_reference })
    }
}

/// Parses the content of a `@template` (and variant) tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
#[inline]
pub fn parse_template_tag(
    content: &[u8],
    span: Span,
    mut covariant: bool,
    mut contravariant: bool,
) -> Result<TemplateTag, ParseError> {
    let trim_start_offset_rel = content.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(0);
    let trimmed_content = content.trim_ascii();

    if trimmed_content.is_empty() {
        return Err(ParseError::InvalidTemplateTag(span, "Expected template parameter name".to_string()));
    }

    let mut parts = trimmed_content.split(|b: &u8| b.is_ascii_whitespace()).filter(|s| !s.is_empty());

    let mut name_part = parts
        .next()
        .ok_or_else(|| ParseError::InvalidTemplateTag(span, "Expected template parameter name".to_string()))?;
    if name_part.starts_with(b"+") && !contravariant && !covariant {
        covariant = true;
        name_part = &name_part[1..];
    } else if name_part.starts_with(b"-") && !contravariant && !covariant {
        contravariant = true;
        name_part = &name_part[1..];
    }

    let name = name_part.to_vec();

    let mut modifier: Option<TemplateModifier> = None;
    let mut type_string_opt: Option<TypeString> = None;
    let mut default_opt: Option<TypeString> = None;

    let mut current_offset_rel = trim_start_offset_rel + name_part.len();

    let remaining_after_name = content.get(current_offset_rel..).unwrap_or(b"");
    let whitespace_len1 = remaining_after_name.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(0);
    let after_whitespace1_offset_rel = current_offset_rel + whitespace_len1;
    let potential_modifier_slice = remaining_after_name.trim_ascii_start();

    if let Some(rest) = potential_modifier_slice.strip_prefix(b"=") {
        let after_eq_offset_rel = after_whitespace1_offset_rel + 1;
        if let Some((default_type, _)) = split_tag_content(rest, span.subspan(after_eq_offset_rel as u32, 0)) {
            default_opt = Some(default_type);
        }
    } else if !potential_modifier_slice.is_empty() {
        let mut modifier_parts =
            potential_modifier_slice.split(|b: &u8| b.is_ascii_whitespace()).filter(|s| !s.is_empty()).peekable();
        if let Some(potential_modifier_str) = modifier_parts.peek().copied() {
            let lowered = potential_modifier_str.to_ascii_lowercase();
            let modifier_val = match lowered.as_slice() {
                b"as" => Some(TemplateModifier::As),
                b"of" => Some(TemplateModifier::Of),
                b"super" => Some(TemplateModifier::Super),
                _ => None,
            };

            if modifier_val.is_some() {
                modifier = modifier_val;
                modifier_parts.next();
                current_offset_rel = after_whitespace1_offset_rel + potential_modifier_str.len();

                let remaining_after_modifier = content.get(current_offset_rel..).unwrap_or(b"");
                if let Some((type_string, _)) =
                    split_tag_content(remaining_after_modifier, span.subspan(current_offset_rel as u32, 0))
                {
                    let type_end_rel = (type_string.span.end.offset - span.start.offset) as usize;
                    type_string_opt = Some(type_string);

                    let after_constraint = content.get(type_end_rel..).unwrap_or(b"");
                    let trimmed = after_constraint.trim_ascii_start();
                    if let Some(rest) = trimmed.strip_prefix(b"=") {
                        let leading_ws = after_constraint.len() - trimmed.len();
                        let after_eq_offset_rel = type_end_rel + leading_ws + 1;
                        if let Some((default_type, _)) =
                            split_tag_content(rest, span.subspan(after_eq_offset_rel as u32, 0))
                        {
                            default_opt = Some(default_type);
                        }
                    }
                }
            }
        }
    }

    Ok(TemplateTag {
        span,
        name,
        modifier,
        type_string: type_string_opt,
        default: default_opt,
        covariant,
        contravariant,
    })
}

/// Parses the content of a `@where` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_where_tag(content: &[u8], span: Span) -> Result<WhereTag, ParseError> {
    let name_end_pos = find_ascii_whitespace(content).ok_or_else(|| {
        ParseError::InvalidWhereTag(span, "Expected template parameter name and constraint".to_string())
    })?;
    let (name_part, rest_raw) = content.split_at(name_end_pos);
    let mut rest = rest_raw;

    if !is_valid_identifier_start(name_part, false) {
        return Err(ParseError::InvalidWhereTag(
            span,
            format!("Invalid template parameter name: '{}'", String::from_utf8_lossy(name_part)),
        ));
    }

    rest = rest.trim_ascii_start();
    let modifier = if rest.starts_with(b"is") && rest.get(2).is_some_and(|b| b.is_ascii_whitespace()) {
        rest = &rest[2..];
        WhereModifier::Is
    } else if rest.starts_with(b":") {
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

    Ok(WhereTag { span, name: name_part.to_vec(), modifier, type_string })
}

/// Parses the content of a `@param` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_param_tag(content: &[u8], span: Span) -> Result<ParameterTag, ParseError> {
    let trimmed = content.trim_ascii_start();

    if trimmed.starts_with(b"$") {
        let raw_name = trimmed
            .split(|b: &u8| b.is_ascii_whitespace())
            .find(|s| !s.is_empty())
            .ok_or_else(|| ParseError::InvalidParameterTag(span, "Expected parameter name".to_string()))?;

        let variable = parse_var_ident(raw_name, false).ok_or_else(|| {
            ParseError::InvalidParameterTag(
                span,
                format!("Invalid parameter name: '{}'", String::from_utf8_lossy(raw_name)),
            )
        })?;

        let desc_start = find_subslice(trimmed, &variable.name).map_or(0, |i| i + variable.name.len());
        let description = trimmed[desc_start..].trim_ascii().to_vec();

        return Ok(ParameterTag { span, variable, type_string: None, description });
    }

    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidParameterTag(span, "Failed to parse parameter type".to_string()))?;

    if type_string.value.is_empty()
        || type_string.value.starts_with(b"{")
        || (type_string.value.starts_with(b"$") && type_string.value != b"$this")
    {
        return Err(ParseError::InvalidParameterTag(
            span,
            format!("Invalid parameter type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    if rest_slice.is_empty() {
        return Err(ParseError::InvalidParameterTag(span, "Missing parameter name".to_string()));
    }

    let raw_name = rest_slice
        .split(|b: &u8| b.is_ascii_whitespace())
        .find(|s| !s.is_empty())
        .ok_or_else(|| ParseError::InvalidParameterTag(span, "Expected parameter name".to_string()))?;
    let variable = parse_var_ident(raw_name, false).ok_or_else(|| {
        ParseError::InvalidParameterTag(
            span,
            format!("Invalid parameter name: '{}'", String::from_utf8_lossy(raw_name)),
        )
    })?;

    let desc_start = find_subslice(rest_slice, &variable.name).map_or(0, |i| i + variable.name.len());
    let description = rest_slice[desc_start..].trim_ascii_start().to_vec();

    Ok(ParameterTag { span, variable, type_string: Some(type_string), description })
}

#[inline]
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    memchr::memmem::find(haystack, needle)
}

/// Parses the content of a `@param-out` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_param_out_tag(content: &[u8], span: Span) -> Result<ParameterOutTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidParameterOutTag(span, "Failed to parse parameter type".to_string()))?;

    if type_string.value.is_empty()
        || type_string.value.starts_with(b"{")
        || (type_string.value.starts_with(b"$") && type_string.value != b"$this")
    {
        return Err(ParseError::InvalidParameterOutTag(
            span,
            format!("Invalid parameter type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    if rest_slice.is_empty() {
        return Err(ParseError::InvalidParameterOutTag(span, "Missing parameter name".to_string()));
    }

    let raw_name = rest_slice
        .split(|b: &u8| b.is_ascii_whitespace())
        .find(|s| !s.is_empty())
        .ok_or_else(|| ParseError::InvalidParameterOutTag(span, "Expected parameter name".to_string()))?;
    let variable = parse_var_ident(raw_name, false).ok_or_else(|| {
        ParseError::InvalidParameterOutTag(
            span,
            format!("Invalid parameter name: '{}'", String::from_utf8_lossy(raw_name)),
        )
    })?;

    Ok(ParameterOutTag { span, variable, type_string })
}

/// Parses the content of a `@return` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_return_tag(content: &[u8], span: Span) -> Result<ReturnTypeTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidReturnTag(span, "Failed to parse return type".to_string()))?;

    if type_string.value.starts_with(b"{") {
        return Err(ParseError::InvalidReturnTag(
            span,
            format!("Invalid return type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    let description = rest_slice.to_vec();

    Ok(ReturnTypeTag { span, type_string, description })
}

/// Parses the content of a `@throws` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_throws_tag(content: &[u8], span: Span) -> Result<ThrowsTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidThrowsTag(span, "Failed to parse exception type".to_string()))?;

    if type_string.value.starts_with(b"{") {
        return Err(ParseError::InvalidThrowsTag(
            span,
            format!("Invalid exception type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    if type_string.value.starts_with(b"$") && type_string.value != b"$this" {
        return Err(ParseError::InvalidThrowsTag(
            span,
            format!("Invalid exception type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    let description = rest_slice.to_vec();

    Ok(ThrowsTag { span, type_string, description })
}

/// Parses the content of an `@assert`/`@assert-if-true`/`@assert-if-false` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_assertion_tag(content: &[u8], span: Span) -> Result<AssertionTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidAssertionTag(span, "Failed to parse assertion type".to_string()))?;

    if type_string.value.is_empty()
        || type_string.value.starts_with(b"{")
        || (type_string.value.starts_with(b"$") && type_string.value != b"$this")
    {
        return Err(ParseError::InvalidAssertionTag(
            span,
            format!("Invalid assertion type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    if rest_slice.is_empty() {
        return Err(ParseError::InvalidAssertionTag(span, "Missing variable name".to_string()));
    }

    let raw_name = rest_slice
        .split(|b: &u8| b.is_ascii_whitespace())
        .find(|s| !s.is_empty())
        .ok_or_else(|| ParseError::InvalidAssertionTag(span, "Expected variable name".to_string()))?;
    let variable = parse_var_ident(raw_name, true).ok_or_else(|| {
        ParseError::InvalidAssertionTag(span, format!("Invalid variable name: '{}'", String::from_utf8_lossy(raw_name)))
    })?;

    Ok(AssertionTag { span, type_string, variable })
}

/// Parses the content of a `@var` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_var_tag(content: &[u8], span: Span) -> Result<VarTag, ParseError> {
    let (type_string, rest_slice) = split_tag_content(content, span)
        .ok_or_else(|| ParseError::InvalidVarTag(span, "Failed to parse variable type".to_string()))?;

    if type_string.value.is_empty()
        || type_string.value.starts_with(b"{")
        || (type_string.value.starts_with(b"$") && type_string.value != b"$this")
    {
        return Err(ParseError::InvalidVarTag(
            span,
            format!("Invalid variable type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    let variable = if rest_slice.is_empty() {
        None
    } else {
        let var_part = rest_slice
            .split(|b: &u8| b.is_ascii_whitespace())
            .find(|s| !s.is_empty())
            .ok_or_else(|| ParseError::InvalidVarTag(span, "Expected variable name".to_string()))?;
        parse_var_ident(var_part, true)
    };

    Ok(VarTag { span, type_string, variable })
}

/// Parses the content of a `@type` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_type_tag(content: &[u8], span: Span) -> Result<TypeTag, ParseError> {
    let leading_ws = (content.len() - content.trim_ascii_start().len()) as u32;
    let content = content.trim_ascii_start();

    if content.is_empty() {
        return Err(ParseError::InvalidTypeTag(span, "Type alias declaration is empty".to_string()));
    }

    let (potential_name, _) = split_once_ascii_whitespace(content).ok_or_else(|| {
        let trimmed = content.trim_ascii();
        ParseError::InvalidTypeTag(
            span,
            format!("Type alias name '{}' must be followed by a type definition", String::from_utf8_lossy(trimmed)),
        )
    })?;

    let name_len = potential_name.len();
    let after_name = &content[name_len..];
    let trimmed_after_name = after_name.trim_ascii_start();

    let (name, type_part, type_offset) = if let Some(after_equals) = trimmed_after_name.strip_prefix(b"=") {
        let name = potential_name.trim_ascii();

        if !is_valid_identifier_start(name, false) {
            return Err(ParseError::InvalidTypeTag(
                span,
                format!("Invalid type alias name: '{}'", String::from_utf8_lossy(name)),
            ));
        }

        let type_start_offset = name_len + (after_name.len() - trimmed_after_name.len()) + 1;

        (name, after_equals, leading_ws + type_start_offset as u32)
    } else {
        let name = potential_name.trim_ascii();

        if !is_valid_identifier_start(name, false) {
            return Err(ParseError::InvalidTypeTag(
                span,
                format!("Invalid type alias name: '{}'", String::from_utf8_lossy(name)),
            ));
        }

        let rest = after_name.trim_ascii_start();
        let type_start_offset = name_len + (after_name.len() - rest.len());

        (name, rest, leading_ws + type_start_offset as u32)
    };

    let (type_string, _) = split_tag_content(type_part, span.subspan(type_offset, 0))
        .ok_or_else(|| ParseError::InvalidTypeTag(span, "Failed to parse type definition".to_string()))?;

    if type_string.value.is_empty()
        || type_string.value.starts_with(b"{")
        || (type_string.value.starts_with(b"$") && type_string.value != b"$this")
    {
        return Err(ParseError::InvalidTypeTag(
            span,
            format!("Invalid type definition: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    Ok(TypeTag { span, name: name.to_vec(), type_string })
}

/// Parses the content of an `@import-type` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_import_type_tag(content: &[u8], span: Span) -> Result<ImportTypeTag, ParseError> {
    let trimmed = content.trim_ascii_start();
    let (name, rest) = split_once_byte(trimmed, b' ').ok_or_else(|| {
        ParseError::InvalidImportTypeTag(span, "Expected type alias name and 'from' clause".to_string())
    })?;
    let name = name.trim_ascii();
    let rest = rest.trim_ascii();

    if !is_valid_identifier_start(name, false) {
        return Err(ParseError::InvalidImportTypeTag(
            span,
            format!("Invalid type alias name: '{}'", String::from_utf8_lossy(name)),
        ));
    }

    if rest.is_empty() {
        return Err(ParseError::InvalidImportTypeTag(span, "Missing 'from' clause".to_string()));
    }

    let (from, rest) = split_once_byte(rest, b' ').ok_or_else(|| {
        ParseError::InvalidImportTypeTag(span, "Expected 'from' keyword followed by class name".to_string())
    })?;

    if !from.eq_ignore_ascii_case(b"from") {
        return Err(ParseError::InvalidImportTypeTag(
            span,
            format!("Expected 'from' keyword, found '{}'", String::from_utf8_lossy(from)),
        ));
    }

    if rest.is_empty() {
        return Err(ParseError::InvalidImportTypeTag(span, "Missing class name after 'from'".to_string()));
    }

    let (imported_from, rest) = if let Some((imp_from, rest)) = split_once_byte(rest, b' ') {
        (imp_from.trim_ascii(), rest.trim_ascii())
    } else {
        (rest.trim_ascii(), b"" as &[u8])
    };

    if !is_valid_identifier_start(imported_from, true) {
        return Err(ParseError::InvalidImportTypeTag(
            span,
            format!("Invalid class name: '{}'", String::from_utf8_lossy(imported_from)),
        ));
    }

    let mut alias = None;

    if let Some((r#as, rest)) = split_once_byte(rest, b' ')
        && r#as.trim_ascii().eq_ignore_ascii_case(b"as")
        && !rest.is_empty()
    {
        let alias_name = rest
            .split(|b: &u8| b.is_ascii_whitespace())
            .find(|s| !s.is_empty())
            .ok_or_else(|| ParseError::InvalidImportTypeTag(span, "Expected alias name after 'as'".to_string()))?
            .trim_ascii()
            .to_vec();
        alias = Some(ByteAlias(alias_name));
    }

    Ok(ImportTypeTag { span, name: name.to_vec(), from: imported_from.to_vec(), alias })
}

/// Parses the content of a `@property` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_property_tag(
    content: &[u8],
    span: Span,
    is_read: bool,
    is_write: bool,
) -> Result<PropertyTag, ParseError> {
    let trimmed_start = content.trim_ascii_start();
    let (type_string, variable) = if trimmed_start.starts_with(b"$") && !trimmed_start.starts_with(b"$this") {
        let var_part = content
            .split(|b: &u8| b.is_ascii_whitespace())
            .find(|s| !s.is_empty())
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, "Expected variable name".to_string()))?;
        let variable = parse_var_ident(var_part, false).ok_or_else(|| {
            ParseError::InvalidPropertyTag(
                span,
                format!("Invalid variable name: '{}'", String::from_utf8_lossy(var_part)),
            )
        })?;

        (None, variable)
    } else {
        let (type_string, rest_slice) = split_tag_content(content, span)
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, "Failed to parse type definition".to_string()))?;

        if type_string.value.is_empty()
            || type_string.value.starts_with(b"{")
            || (type_string.value.starts_with(b"$") && type_string.value != b"$this")
        {
            return Err(ParseError::InvalidPropertyTag(
                span,
                format!("Invalid type definition: '{}'", String::from_utf8_lossy(&type_string.value)),
            ));
        }

        if rest_slice.is_empty() {
            return Err(ParseError::InvalidPropertyTag(span, "Missing variable name after type".to_string()));
        }

        let var_part = rest_slice
            .split(|b: &u8| b.is_ascii_whitespace())
            .find(|s| !s.is_empty())
            .ok_or_else(|| ParseError::InvalidPropertyTag(span, "Expected variable name".to_string()))?;
        let variable = parse_var_ident(var_part, false).ok_or_else(|| {
            ParseError::InvalidPropertyTag(
                span,
                format!("Invalid variable name: '{}'", String::from_utf8_lossy(var_part)),
            )
        })?;

        (Some(type_string), variable)
    };

    Ok(PropertyTag { span, type_string, variable, is_read, is_write })
}

/// Splits tag content into the type-string part and the rest, respecting brackets/quotes.
#[inline]
#[must_use]
pub fn split_tag_content(content: &[u8], input_span: Span) -> Option<(TypeString, &[u8])> {
    let trim_start_offset = content.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(0);
    let trimmed_start_pos = input_span.start.forward(trim_start_offset as u32);

    let trimmed_content = content.trim_ascii();
    if trimmed_content.is_empty() {
        return None;
    }

    let mut bracket_stack: Vec<u8> = Vec::with_capacity(8);
    let mut quote_char: Option<u8> = None;
    let mut escaped = false;
    let mut last_char_was_significant = false;
    let mut split_point_rel: Option<usize> = None;

    let bytes = trimmed_content;
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        let ch = bytes[i];

        if let Some(q) = quote_char {
            if ch == q && !escaped {
                quote_char = None;
            } else {
                escaped = ch == b'\\' && !escaped;
            }
            last_char_was_significant = true;
            i += 1;
            continue;
        }
        if ch == b'\'' || ch == b'"' {
            quote_char = Some(ch);
            last_char_was_significant = true;
            i += 1;
            continue;
        }
        match ch {
            b'<' | b'(' | b'[' | b'{' => bracket_stack.push(ch),
            b'>' | b')' | b']' | b'}' => match bracket_stack.pop() {
                Some(opening) if brackets_match(opening, ch) => {}
                _ => return None,
            },
            _ => {}
        }

        if ch == b':' || ch == b'|' || ch == b'&' {
            last_char_was_significant = true;

            let mut peek_i = i + 1;
            let mut has_whitespace_after = false;
            while peek_i < len && bytes[peek_i].is_ascii_whitespace() {
                peek_i += 1;
                has_whitespace_after = true;
            }

            let next_non_ws = bytes.get(peek_i).copied();
            if bracket_stack.is_empty() && matches!(next_non_ws, None | Some(b'$')) {
                split_point_rel = Some(i + 1);
                break;
            }

            if has_whitespace_after {
                while i + 1 < len && bytes[i + 1].is_ascii_whitespace() {
                    i += 1;
                }
            }

            i += 1;
            continue;
        }

        if ch == b'/' && bytes.get(i + 1).is_some_and(|&b| b == b'/') {
            if !bracket_stack.is_empty() {
                while i + 1 < len {
                    if bytes[i + 1] == b'\n' {
                        break;
                    }
                    i += 1;
                }
                last_char_was_significant = true;
                i += 1;
                continue;
            }

            split_point_rel = Some(i);

            break;
        }

        if ch.is_ascii_whitespace() {
            if bracket_stack.is_empty() && last_char_was_significant {
                let mut peek_i = i + 1;
                let mut found_continuation = false;

                while peek_i < len && bytes[peek_i].is_ascii_whitespace() {
                    peek_i += 1;
                }

                if peek_i < len {
                    let next_char = bytes[peek_i];
                    found_continuation = next_char == b':'
                        || next_char == b'|'
                        || (next_char == b'&' && {
                            let after = bytes.get(peek_i + 1).copied();
                            !matches!(after, Some(b'$') | Some(b'.'))
                        });
                }

                if found_continuation {
                    while i + 1 < len && bytes[i + 1].is_ascii_whitespace() {
                        i += 1;
                    }

                    last_char_was_significant = true;
                } else {
                    split_point_rel = Some(i);
                    break;
                }
            } else {
                last_char_was_significant = false;
            }
        } else if ch == b'.' {
            let prev_is_digit = i > 0 && bytes[i - 1].is_ascii_digit();
            let next_is_digit = bytes.get(i + 1).is_some_and(|b| b.is_ascii_digit());

            if prev_is_digit && next_is_digit {
                last_char_was_significant = true;
            } else if bracket_stack.is_empty() && last_char_was_significant {
                split_point_rel = Some(i);
                break;
            } else {
                last_char_was_significant = false;
            }
        } else {
            last_char_was_significant = true;
        }

        i += 1;
    }

    if !bracket_stack.is_empty() || quote_char.is_some() {
        return None;
    }

    if let Some(split_idx_rel) = split_point_rel {
        let type_part_slice = trimmed_content[..split_idx_rel].trim_ascii_end();
        let rest_part_slice = trimmed_content[split_idx_rel..].trim_ascii_start();

        let type_span =
            Span::new(input_span.file_id, trimmed_start_pos, trimmed_start_pos.forward(type_part_slice.len() as u32));

        Some((TypeString { value: type_part_slice.to_vec(), span: type_span }, rest_part_slice))
    } else {
        let type_part_slice = trimmed_content;
        let type_span =
            Span::new(input_span.file_id, trimmed_start_pos, trimmed_start_pos.forward(type_part_slice.len() as u32));

        Some((TypeString { value: type_part_slice.to_vec(), span: type_span }, b"" as &[u8]))
    }
}

/// Parses the content of a `@method` tag.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tag content is malformed.
pub fn parse_method_tag(mut content: &[u8], mut span: Span) -> Result<MethodTag, ParseError> {
    let (trimmed_content, leading_ws) = consume_whitespace(content);
    content = trimmed_content;
    span = span.subspan(leading_ws as u32, span.length());

    let mut is_static = false;
    let mut visibility = None;

    let mut acc_len = 0;

    let mut static_modifier_start = 0u32;
    let mut static_modifier_len = 0u32;

    loop {
        if let Some((new_content, char_count)) = try_consume(content, b"static ") {
            if is_static {
                break;
            }

            is_static = true;
            static_modifier_start = acc_len as u32;
            static_modifier_len = 6;
            acc_len += char_count;
            content = new_content;
        } else if let Some((new_content, char_count)) = try_consume(content, b"public ") {
            if visibility.is_some() {
                return Err(ParseError::InvalidMethodTag(span, "Duplicate visibility modifier".to_string()));
            }

            visibility = Some(Visibility::Public);
            acc_len += char_count;
            content = new_content;
        } else if let Some((new_content, char_count)) = try_consume(content, b"protected ") {
            if visibility.is_some() {
                return Err(ParseError::InvalidMethodTag(span, "Duplicate visibility modifier".to_string()));
            }

            visibility = Some(Visibility::Protected);
            acc_len += char_count;
            content = new_content;
        } else if let Some((new_content, char_count)) = try_consume(content, b"private ") {
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
        let type_string = TypeString { value: b"static".to_vec(), span: static_span };
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

    if type_string.value.is_empty()
        || type_string.value.starts_with(b"{")
        || (type_string.value.starts_with(b"$") && type_string.value != b"$this")
    {
        return Err(ParseError::InvalidMethodTag(
            span,
            format!("Invalid return type: '{}'", String::from_utf8_lossy(&type_string.value)),
        ));
    }

    if rest_slice.is_empty() {
        return Err(ParseError::InvalidMethodTag(span, "Missing method signature".to_string()));
    }

    let name_end = position_of(rest_slice, b'(').ok_or_else(|| {
        ParseError::InvalidMethodTag(span, "Missing opening parenthesis '(' for method arguments".to_string())
    })?;

    let name = rest_slice[..name_end].trim_ascii();

    if name.is_empty() {
        return Err(ParseError::InvalidMethodTag(span, "Missing method name".to_string()));
    }

    let mut depth = 1;
    let mut args_end = None;

    let mut idx = name_end + 1;
    while idx < rest_slice.len() {
        match rest_slice[idx] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    args_end = Some(idx);
                    break;
                }
            }
            _ => {}
        }
        idx += 1;
    }

    let args_end = args_end.ok_or_else(|| {
        ParseError::InvalidMethodTag(span, "Missing closing parenthesis ')' for method arguments".to_string())
    })?;
    let (args_str, whitespace_count) = consume_whitespace(&rest_slice[name_end + 1..args_end]);
    let args_span = rest_slice_span.subspan((whitespace_count + name_end) as u32 + 1, args_end as u32);

    let description = rest_slice[args_end..].trim_ascii();
    let arguments_split = split_args(args_str, args_span);
    let arguments = arguments_split.iter().filter_map(|(arg, span)| parse_argument(arg, span)).collect::<Vec<_>>();

    let method = Method {
        name: name.to_vec(),
        argument_list: arguments,
        visibility: visibility.unwrap_or(Visibility::Public),
        is_static,
    };

    Ok(MethodTag { span, type_string, method, description: description.to_vec() })
}

fn consume_whitespace(input: &[u8]) -> (&[u8], usize) {
    let mut byte_count = 0;
    while byte_count < input.len() && input[byte_count].is_ascii_whitespace() {
        byte_count += 1;
    }
    (&input[byte_count..], byte_count)
}

fn try_consume<'input>(input: &'input [u8], token: &[u8]) -> Option<(&'input [u8], usize)> {
    let (input, whitespace_count) = consume_whitespace(input);

    if !input.starts_with(token) {
        return None;
    }

    let len = token.len() + whitespace_count;
    let input = &input[token.len()..];

    let (input, trailing_whitespace) = consume_whitespace(input);

    Some((input, len + trailing_whitespace))
}

fn looks_like_method_signature_only(content: &[u8]) -> bool {
    let trimmed = content.trim_ascii();
    if let Some(paren_pos) = position_of(trimmed, b'(') {
        let before_paren = trimmed[..paren_pos].trim_ascii();
        !before_paren.is_empty() && !before_paren.contains(&b' ')
    } else {
        false
    }
}

fn split_args(args_str: &[u8], span: Span) -> Vec<(&[u8], Span)> {
    let mut args = Vec::new();

    let mut start = 0;
    let mut depth = 0;
    for (i, &ch) in args_str.iter().enumerate() {
        match ch {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
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
        let arg_trimmed = arg.trim_ascii_end();
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

fn parse_argument(arg_str: &[u8], span: &Span) -> Option<Argument> {
    let default_value_split = rsplit_once_byte(arg_str, b'=');

    let ((arg_type, raw_name), default_value): ((_, _), Option<&[u8]>) =
        if let Some((variable_definition, default_value)) = default_value_split {
            let arg = variable_definition.trim_ascii();
            if let Some((arg_type, raw_name)) = rsplit_once_byte(arg, b' ') {
                ((Some(arg_type), raw_name), Some(default_value.trim_ascii()))
            } else {
                ((None, arg), Some(default_value))
            }
        } else {
            let arg = arg_str.trim_ascii();
            if let Some((arg_type, raw_name)) = rsplit_once_byte(arg, b' ') {
                ((Some(arg_type), raw_name), None)
            } else {
                ((None, arg), None)
            }
        };

    let type_string =
        arg_type.map(|arg_type| TypeString { value: arg_type.to_vec(), span: span.subspan(0, arg_type.len() as u32) });

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

#[inline]
const fn brackets_match(open: u8, close: u8) -> bool {
    matches!((open, close), (b'<', b'>') | (b'(', b')') | (b'[', b']') | (b'{', b'}'))
}

#[inline]
fn is_valid_identifier_start(mut identifier: &[u8], allow_qualified: bool) -> bool {
    if allow_qualified && identifier.starts_with(b"\\") {
        identifier = &identifier[1..];
    }

    if identifier.is_empty() {
        return false;
    }

    let first = identifier[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }

    identifier.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_' || (allow_qualified && b == b'\\'))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::single_char_lifetime_names)]
mod tests {
    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;

    use super::*;

    fn test_span(input: &[u8], start_offset: u32) -> Span {
        let base_start = Position::new(start_offset);
        Span::new(FileId::zero(), base_start, base_start.forward(input.len() as u32))
    }

    fn test_span_for(s: &[u8]) -> Span {
        test_span(s, 0)
    }

    fn make_span(start: u32, end: u32) -> Span {
        Span::new(FileId::zero(), Position::new(start), Position::new(end))
    }

    #[test]
    fn test_parse_var_ident() {
        struct Expect<'a> {
            s: &'a [u8],
            variadic: bool,
            by_ref: bool,
        }
        let cases: &[(&[u8], Option<Expect>)] = &[
            (b"$x", Some(Expect { s: b"$x", variadic: false, by_ref: false })),
            (b"&$refVar", Some(Expect { s: b"$refVar", variadic: false, by_ref: true })),
            (b"$foo,", Some(Expect { s: b"$foo", variadic: false, by_ref: false })),
            (b"...$ids)", Some(Expect { s: b"$ids", variadic: true, by_ref: false })),
            (b"...$items,", Some(Expect { s: b"$items", variadic: true, by_ref: false })),
            (b"$", None),
            (b"...$", None),
            (b"$1x", None),
            (b"foo", None),
        ];

        for (input, expected) in cases {
            let got = parse_var_ident(input, false);
            match (got, expected) {
                (None, None) => {}
                (Some(v), Some(e)) => {
                    assert_eq!(v.name, e.s, "input={input:?}");
                    assert_eq!(v.is_variadic, e.variadic, "input={input:?}");
                    assert_eq!(v.is_by_reference, e.by_ref, "input={input:?}");
                }
                _ => panic!("mismatch for input={input:?}"),
            }
        }
    }

    #[test]
    fn test_parse_var_ident_accepts_non_ascii_bytes() {
        let input = "$module🤔_lorem_ipsum_dolor_sit_amet_consete".as_bytes();
        let parsed = parse_var_ident(input, false).expect("emoji-containing variable should parse");
        assert_eq!(parsed.name.as_slice(), input);
        assert!(!parsed.is_variadic);
        assert!(!parsed.is_by_reference);

        let parsed = parse_var_ident(input, true).expect("emoji-containing variable should parse (path mode)");
        assert_eq!(parsed.name.as_slice(), input);

        let input = "$café".as_bytes();
        let parsed = parse_var_ident(input, false).expect("$café should parse");
        assert_eq!(parsed.name.as_slice(), input);

        let input = "$module🤔->name".as_bytes();
        let parsed = parse_var_ident(input, true).expect("property access on emoji name should parse");
        assert_eq!(parsed.name.as_slice(), input);
    }

    #[test]
    fn test_variable_display_and_raw() {
        let cases: &[(&[u8], &str)] =
            &[(b"$x", "$x"), (b"&$x", "&$x"), (b"...$x", "...$x"), (b"...$x)", "...$x"), (b"...$x,", "...$x")];

        for (input, expected_raw) in cases {
            let v = parse_var_ident(input, false).expect("should parse variable");
            assert_eq!(v.to_string(), *expected_raw);
        }
    }

    #[test]
    fn test_splitter_brackets() {
        let input = b"array<int, (string|bool)> desc";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"array<int, (string|bool)>");
        assert_eq!(ts.span, make_span(0, b"array<int, (string|bool)>".len() as u32));
        assert_eq!(rest, b"desc");

        let input = b"array<int, string> desc";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"array<int, string>");
        assert_eq!(ts.span, make_span(0, b"array<int, string>".len() as u32));
        assert_eq!(rest, b"desc");

        assert!(split_tag_content(b"array<int", test_span_for(b"array<int")).is_none());
        assert!(split_tag_content(b"array<int)", test_span_for(b"array<int)")).is_none());
        assert!(split_tag_content(b"array(int>", test_span_for(b"array(int>")).is_none());
        assert!(split_tag_content(b"string>", test_span_for(b"string>")).is_none());
    }

    #[test]
    fn test_splitter_quotes() {
        let input = b" 'inside quote' outside ";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"'inside quote'");
        assert_eq!(ts.span, make_span(1, "'inside quote'".len() as u32 + 1));
        assert_eq!(rest, b"outside");

        let input = br#""string \" with escape" $var"#;
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, br#""string \" with escape""#);
        assert_eq!(ts.span, make_span(0, r#""string \" with escape""#.len() as u32));
        assert_eq!(rest, b"$var");

        assert!(split_tag_content(b"\"unterminated", test_span_for(b"\"unterminated")).is_none());
    }

    #[test]
    fn test_splitter_comments() {
        let input = b"(string // comment \n | int) $var";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"(string // comment \n | int)");
        assert_eq!(ts.span, make_span(0, "(string // comment \n | int)".len() as u32));
        assert_eq!(rest, b"$var");

        let input = b"string // comment goes to end";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"string");
        assert_eq!(ts.span, make_span(0, "string".len() as u32));
        assert_eq!(rest, b"// comment goes to end");

        let input = b"array<string // comment\n> $var";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"array<string // comment\n>");
        assert_eq!(ts.span, make_span(0, "array<string // comment\n>".len() as u32));
        assert_eq!(rest, b"$var");
    }

    #[test]
    fn test_splitter_whole_string_is_type() {
        let input = b" array<int, string> ";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"array<int, string>");
        assert_eq!(ts.span, make_span(1, "array<int, string>".len() as u32 + 1));
        assert_eq!(rest, b"");
    }

    #[test]
    fn test_splitter_with_dot() {
        let input = b"string[]. something";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"string[]");
        assert_eq!(ts.span, make_span(0, "string[]".len() as u32));
        assert_eq!(rest, b". something");
    }

    #[test]
    fn test_param_basic() {
        let offset = 10;
        let content = b" string|int $myVar Description here ";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();

        assert_eq!(result.type_string.as_ref().unwrap().value, b"string|int");
        assert_eq!(result.type_string.as_ref().unwrap().span.start.offset, offset + 1);
        assert_eq!(result.type_string.as_ref().unwrap().span.end.offset, offset + 1 + "string|int".len() as u32);
        assert_eq!(result.variable.name, b"$myVar");
        assert_eq!(result.description, b"Description here");
        assert_eq!(result.span, span);
    }

    #[test]
    fn test_param_complex_type_no_desc() {
        let offset = 5;
        let content = b" array<int, string> $param ";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.as_ref().unwrap().value, b"array<int, string>");
        assert_eq!(result.type_string.as_ref().unwrap().span.start.offset, offset + 1);
        assert_eq!(
            result.type_string.as_ref().unwrap().span.end.offset,
            offset + 1 + "array<int, string>".len() as u32
        );
        assert_eq!(result.variable.name, b"$param");
        assert_eq!(result.description, b"");
    }

    #[test]
    fn test_param_type_with_comment() {
        let offset = 20;
        let content = b" (string // comment \n | int) $var desc";
        let span = test_span(content, offset);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.as_ref().unwrap().value, b"(string // comment \n | int)");
        assert_eq!(result.type_string.as_ref().unwrap().span.start.offset, offset + 1);
        assert_eq!(
            result.type_string.as_ref().unwrap().span.end.offset,
            offset + 1 + "(string // comment \n | int)".len() as u32
        );
        assert_eq!(result.variable.name, b"$var");
        assert_eq!(result.description, b"desc");
    }

    #[test]
    fn test_param_no_type() {
        let content = b" $param Description here ";
        let span = test_span(content, 0);
        let result = parse_param_tag(content, span).unwrap();
        assert!(result.type_string.is_none());
        assert_eq!(result.variable.name, b"$param");
        assert_eq!(result.description, b"Description here");
    }

    #[test]
    fn test_return_basic() {
        let offset = 10u32;
        let content = b" string Description here ";
        let span = test_span(content, offset);
        let result = parse_return_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, b"string");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "string".len() as u32);
        assert_eq!(result.description, b"Description here");
        assert_eq!(result.span, span);
    }

    #[test]
    fn test_return_complex_type_with_desc() {
        let offset = 0;
        let content = b" array<int, (string|null)> Description ";
        let span = test_span(content, offset);
        let result = parse_return_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, b"array<int, (string|null)>");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, (string|null)>".len() as u32);
        assert_eq!(result.description, b"Description");
    }

    #[test]
    fn test_return_complex_type_no_desc() {
        let offset = 0;
        let content = b" array<int, (string|null)> ";
        let span = test_span(content, offset);
        let result = parse_return_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, b"array<int, (string|null)>");
        assert_eq!(result.type_string.span.start.offset, offset + 1);
        assert_eq!(result.type_string.span.end.offset, offset + 1 + "array<int, (string|null)>".len() as u32);
        assert_eq!(result.description, b"");
    }

    #[test]
    fn test_param_out_no_type() {
        let content = b" $myVar ";
        let span = test_span(content, 0);
        parse_param_out_tag(content, span).unwrap_err();
    }

    #[test]
    fn test_param_out_no_var() {
        let content = b" string ";
        let span = test_span(content, 0);
        parse_param_out_tag(content, span).unwrap_err();
    }

    #[test]
    fn test_type() {
        let content = b"MyType = string";
        let span = test_span_for(content);
        let result = parse_type_tag(content, span).unwrap();
        assert_eq!(result.name, b"MyType");
        assert_eq!(result.type_string.value, b"string");
        assert_eq!(result.type_string.span.start.offset, 9);
        assert_eq!(result.type_string.span.end.offset, 9 + "string".len() as u32);
        assert_eq!(result.span, span);
    }

    #[test]
    fn test_import_type() {
        let content = b"MyType from \\My\\Namespace\\Class as Alias";
        let span = test_span_for(content);
        let result = parse_import_type_tag(content, span).unwrap();
        assert_eq!(result.name, b"MyType");
        assert_eq!(result.from, b"\\My\\Namespace\\Class");
        assert_eq!(result.alias.as_ref().map(|a| a.0.as_slice()), Some(b"Alias" as &[u8]));
        assert_eq!(result.span, span);
    }

    #[test]
    fn test_param_trailing_comma_is_ignored_in_name() {
        let content = b" string $foo, desc";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, b"$foo");
        assert_eq!(result.description, b", desc");
    }

    #[test]
    fn test_param_variadic_trailing_paren_is_ignored_in_name() {
        let content = b" list<int> ...$items) rest";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, b"$items");
        assert_eq!(result.description, b") rest");
    }

    #[test]
    fn test_param_out_trailing_comma() {
        let content = b" int $out,";
        let span = test_span_for(content);
        let result = parse_param_out_tag(content, span).unwrap();
        assert_eq!(result.variable.name, b"$out");
    }

    #[test]
    fn test_assertion_trailing_comma() {
        let content = b" int $x,";
        let span = test_span_for(content);
        let result = parse_assertion_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, b"int");
        assert_eq!(result.variable.name, b"$x");
    }

    #[test]
    fn test_assertion_method_call() {
        let content = b" Statement $this->first()";
        let span = test_span_for(content);
        let result = parse_assertion_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, b"Statement");
        assert_eq!(result.variable.name, b"$this->first()");
    }

    #[test]
    fn test_assertion_property_access() {
        let content = b" Statement $this->property";
        let span = test_span_for(content);
        let result = parse_assertion_tag(content, span).unwrap();
        assert_eq!(result.type_string.value, b"Statement");
        assert_eq!(result.variable.name, b"$this->property");
    }

    #[test]
    fn test_param_trailing_without_space() {
        let content = b" string $foo,desc";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, b"$foo");
        assert_eq!(result.description, b",desc");
    }

    #[test]
    fn test_param_variadic_trailing_paren_without_space() {
        let content = b" list<int> ...$items)more";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.variable.name, b"$items");
        assert_eq!(result.description, b")more");
    }

    #[test]
    fn test_param_with_numeric_literals_in_union() {
        let content = b"-1|-24.0|string $a";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.as_ref().unwrap().value, b"-1|-24.0|string");
        assert_eq!(result.variable.name, b"$a");
        assert_eq!(result.description, b"");
    }

    #[test]
    fn test_param_with_float_literals() {
        let content = b"1.5|2.0|3.14 $value";
        let span = test_span_for(content);
        let result = parse_param_tag(content, span).unwrap();
        assert_eq!(result.type_string.as_ref().unwrap().value, b"1.5|2.0|3.14");
        assert_eq!(result.variable.name, b"$value");
    }

    #[test]
    fn test_splitter_with_dot_still_works_as_separator() {
        let input = b"string[]. something else";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"string[]");
        assert_eq!(rest, b". something else");
    }

    #[test]
    fn test_splitter_with_colon_after_whitespace() {
        let input = b"callable(string)    :         string     $callback";
        let span = test_span_for(input);
        let (ts, rest) = split_tag_content(input, span).unwrap();
        assert_eq!(ts.value, b"callable(string)    :         string");
        assert_eq!(rest, b"$callback");

        let input2 = b"callable(string) : string $callback";
        let span2 = test_span_for(input2);
        let (ts2, rest2) = split_tag_content(input2, span2).unwrap();
        assert_eq!(ts2.value, b"callable(string) : string");
        assert_eq!(rest2, b"$callback");

        let input3 = b"callable(string): string $callback";
        let span3 = test_span_for(input3);
        let (ts3, rest3) = split_tag_content(input3, span3).unwrap();
        assert_eq!(ts3.value, b"callable(string): string");
        assert_eq!(rest3, b"$callback");
    }

    #[test]
    fn test_consume_whitespace_ascii_only() {
        let input = b"   rest";
        let (rest, count) = consume_whitespace(input);
        assert_eq!(rest, b"rest");
        assert_eq!(count, 3);

        let input2 = b"\t \trest";
        let (rest2, count2) = consume_whitespace(input2);
        assert_eq!(rest2, b"rest");
        assert_eq!(count2, 3);
    }

    #[test]
    fn test_parse_method_tag_static_modifier_with_static_return_type() {
        let input: &[u8] = b"static static magicMethod()";
        let tag = parse_method_tag(input, test_span_for(input)).expect("tag should parse");

        assert!(tag.method.is_static);
        assert_eq!(tag.type_string.value, b"static");
        assert_eq!(tag.method.name, b"magicMethod");
        assert!(tag.method.argument_list.is_empty());
    }

    #[test]
    fn test_parse_method_tag_static_modifier_with_concrete_return_type() {
        let input: &[u8] = b"static int count()";
        let tag = parse_method_tag(input, test_span_for(input)).expect("tag should parse");

        assert!(tag.method.is_static);
        assert_eq!(tag.type_string.value, b"int");
        assert_eq!(tag.method.name, b"count");
    }
}
