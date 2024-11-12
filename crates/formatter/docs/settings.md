# PHP Formatter Configuration

This document outlines all configuration options available for the PHP formatter. Each option is explained with its possible values, aliases (if applicable), and default settings.

---

## Configurations

### `print_width`

- **Description**: Specifies the maximum line length before wrapping.
- **Possible Values**: Any positive integer.
- **Default**: `120`.

---

### `tab_width`

- **Description**: Specifies the number of spaces per indentation level.
- **Possible Values**: Any positive integer.
- **Default**: `4`.

---

### `use_tabs`

- **Description**: Indent lines using tabs instead of spaces.
- **Possible Values**: `true` (tabs) or `false` (spaces).
- **Default**: `false`.

---

### `end_of_line`

- **Description**: Specifies the line ending style.
- **Possible Values**:
  - `auto` (default): Use the system's line ending.
  - `lf`: Line Feed (`\n`).
  - `crlf`: Carriage Return + Line Feed (`\r\n`).
  - `cr`: Carriage Return (`\r`).
- **Aliases**: `lf`, `crlf`, `cr`, `auto`.
- **Default**: `lf`.

---

### `single_quote`

- **Description**: Use single quotes for strings instead of double quotes.
- **Possible Values**: `true` (single quotes) or `false` (double quotes).
- **Default**: `false`.

---

### `trailing_comma`

- **Description**: Include trailing commas in multi-line structures.
- **Possible Values**: `true` or `false`.
- **Default**: `true`.

---

### `space_around_declare_equals`

- **Description**: Add spaces around the `=` in `declare` statements.
- **Possible Values**: `true` or `false`.
- **Default**: `false`.

---

### `strict_types_semicolon`

- **Description**: Include a semicolon after `declare(strict_types=1)`.
- **Possible Values**: `true` or `false`.
- **Default**: `true`.

---

### `keyword_case`

- **Description**: Specifies the casing style for keywords.
- **Possible Values**:
  - `lowercase` (default): `if`, `else`.
  - `uppercase`: `IF`, `ELSE`.
- **Aliases**: `lower`, `upper`.
- **Default**: `lowercase`.

---

### `string_cast`

- **Description**: Specifies the operator used for string casting.
- **Possible Values**:
  - `(string)` (default): Standard cast.
  - `(binary)`: Binary cast.
- **Aliases**: `string`, `binary`.
- **Default**: `(string)`.

---

### `float_cast`

- **Description**: Specifies the operator used for float casting.
- **Possible Values**:
  - `(float)` (default).
  - `(double)`.
  - `(real)`.
- **Aliases**: `float`, `double`, `real`.
- **Default**: `(float)`.

---

### `bool_cast`

- **Description**: Specifies the operator used for boolean casting.
- **Possible Values**:
  - `(bool)` (default).
  - `(boolean)`.
- **Aliases**: `bool`, `boolean`.
- **Default**: `(bool)`.

---

### `int_cast`

- **Description**: Specifies the operator used for integer casting.
- **Possible Values**:
  - `(int)` (default).
  - `(integer)`.
- **Aliases**: `int`, `integer`.
- **Default**: `(int)`.

---

### `leave_casts_as_is`

- **Description**: Retain casting operators as-is without formatting.
- **Possible Values**: `true` or `false`.
- **Default**: `false`.

---

### `include_closing_tag`

- **Description**: Include the closing `?>` tag in files containing only PHP.
- **Possible Values**: `true` or `false`.
- **Default**: `false`.

---

### `blank_line_after_open_tag`

- **Description**: Add a blank line after the opening PHP tag.
- **Possible Values**: `true` or `false`.
- **Default**: `true`.

---

### `elseif_style`

- **Description**: Determines the style for `elseif` or `else if`.
- **Possible Values**:
  - `else if` (default).
  - `elseif`.
- **Aliases**: `else-if`, `compact`, `spaced`.
- **Default**: `else if`.

---

### `array_style`

- **Description**: Specifies the array syntax style.
- **Possible Values**:
  - `short` (default): `[a, b]`.
  - `long`: `array(a, b)`.
- **Aliases**: `short`, `long`, `legacy`.
- **Default**: `short`.

---

### `list_style`

- **Description**: Specifies the list syntax style.
- **Possible Values**:
  - `short` (default): `[a, b]`.
  - `long`: `list(a, b)`.
- **Aliases**: `short`, `long`, `legacy`.
- **Default**: `short`.

---

### `attr_parens`

- **Description**: Specifies whether PHP attributes without arguments include parentheses.
- **Possible Values**:
  - `with_parens` (default): Include parentheses.
  - `without_parens`: Exclude parentheses.
- **Aliases**: `with`, `without`.
- **Default**: `with_parens`.

---

### `null_type_hint`

- **Description**: Specifies the style for null type hints.
- **Possible Values**:
  - `null_pipe` (default): `null|foo`.
  - `question`: `?foo`.
- **Aliases**: `pipe`, `long`, `question`, `short`.
- **Default**: `null_pipe`.

---

### `binary_op_spacing`

- **Description**: Sets the number of spaces around binary operators.
- **Possible Values**: Any non-negative integer.
- **Default**: `1`.

---

### `type_spacing`

- **Description**: Sets the spacing in union/intersection types.
- **Possible Values**: Any non-negative integer.
- **Default**: `0`.

---

### `split_multi_declare`

- **Description**: Split constants and properties into separate statements.
- **Possible Values**: `true` or `false`.
- **Default**: `true`.

---
