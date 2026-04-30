//! `textDocument/semanticTokens/full`; token-level highlighting.
//!
//! We lex the file with mago's lexer and emit one semantic token per
//! syntactic token whose [`TokenKind`] maps to an LSP
//! [`SemanticTokenType`]. The encoding follows the LSP delta-line/delta-
//! start scheme.

use mago_database::file::File as MagoFile;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;
use tower_lsp::lsp_types::SemanticToken;
use tower_lsp::lsp_types::SemanticTokenType;

use crate::language_server::capabilities::lookup;

/// The legend ordering used both in the `initialize` capability advertisement
/// and the encoder. Editors look up `tokenType` indices into this list.
pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::TYPE,
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::PROPERTY,
];

const T_KEYWORD: u32 = 0;
const T_COMMENT: u32 = 1;
const T_STRING: u32 = 2;
const T_NUMBER: u32 = 3;
const T_OPERATOR: u32 = 4;
const T_VARIABLE: u32 = 5;
const T_FUNCTION: u32 = 6;
const T_TYPE: u32 = 7;
const T_NAMESPACE: u32 = 8;
#[allow(dead_code)]
const T_PARAMETER: u32 = 9;
#[allow(dead_code)]
const T_PROPERTY: u32 = 10;

pub fn compute(file: &MagoFile) -> Vec<SemanticToken> {
    let tokens = lookup::lex(file);

    let mut out: Vec<SemanticToken> = Vec::with_capacity(tokens.len() / 2);
    let mut prev_line: u32 = 0;
    let mut prev_start: u32 = 0;
    let mut prev_token: Option<TokenKind> = None;

    for token in &tokens {
        let Some(token_type) = classify(token, prev_token) else {
            if !lookup::is_trivia(token.kind) {
                prev_token = Some(token.kind);
            }
            continue;
        };

        let line = file.line_number(token.start.offset);
        let line_start = file.get_line_start_offset(line).unwrap_or(token.start.offset);
        let column = token.start.offset - line_start;
        let length = token.value.len() as u32;
        let last_byte = token.start.offset + length;
        if file.line_number(last_byte.saturating_sub(1)) != line {
            if !lookup::is_trivia(token.kind) {
                prev_token = Some(token.kind);
            }
            continue;
        }

        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 { column - prev_start } else { column };

        out.push(SemanticToken { delta_line, delta_start, length, token_type, token_modifiers_bitset: 0 });

        prev_line = line;
        prev_start = column;
        if !lookup::is_trivia(token.kind) {
            prev_token = Some(token.kind);
        }
    }

    out
}

#[allow(clippy::too_many_lines)]
fn classify(token: &Token<'_>, prev: Option<TokenKind>) -> Option<u32> {
    use TokenKind::*;

    Some(match token.kind {
        SingleLineComment | HashComment | MultiLineComment | DocBlockComment => T_COMMENT,

        LiteralString | PartialLiteralString | StringPart => T_STRING,

        LiteralInteger | LiteralFloat => T_NUMBER,

        Variable => T_VARIABLE,

        Identifier => match prev {
            Some(Function | Class | Interface | Trait | Enum | Const | New | Use | Namespace) => {
                match token.value.chars().next() {
                    Some(c) if c.is_ascii_uppercase() => T_TYPE,
                    _ => T_FUNCTION,
                }
            }
            Some(MinusGreaterThan | QuestionMinusGreaterThan | ColonColon) => T_FUNCTION,
            _ => match token.value.chars().next() {
                Some(c) if c.is_ascii_uppercase() => T_TYPE,
                _ => T_FUNCTION,
            },
        },

        QualifiedIdentifier | FullyQualifiedIdentifier => T_NAMESPACE,

        Abstract | And | Array | As | Break | Callable | Case | Catch | Class | ClassConstant | Clone | Const
        | Continue | Declare | Default | Do | Echo | Else | ElseIf | Empty | EndDeclare | EndFor | EndForeach
        | EndIf | EndSwitch | EndWhile | Enum | Eval | Exit | Extends | False | Final | Finally | Fn | For
        | Foreach | From | Function | Global | Goto | HaltCompiler | If | Implements | Include | IncludeOnce
        | Instanceof | Insteadof | Interface | Isset | List | Match | Namespace | New | Null | Or | Parent | Print
        | Private | Protected | Public | Readonly | Require | RequireOnce | Return | Self_ | Static | Switch
        | Throw | Trait | Try | True | Unset | Use | Var | While | Xor | Yield => T_KEYWORD,

        TraitConstant | FunctionConstant | MethodConstant | LineConstant | FileConstant | DirConstant
        | NamespaceConstant => T_KEYWORD,

        Ampersand
        | AmpersandEqual
        | AmpersandAmpersand
        | AmpersandAmpersandEqual
        | Asterisk
        | AsteriskEqual
        | Bang
        | BangEqual
        | BangEqualEqual
        | Caret
        | CaretEqual
        | Colon
        | ColonColon
        | Comma
        | Dot
        | DotEqual
        | DotDotDot
        | Equal
        | EqualEqual
        | EqualEqualEqual
        | EqualGreaterThan
        | GreaterThan
        | GreaterThanEqual
        | LessThan
        | LessThanEqual
        | LessThanGreaterThan
        | LessThanEqualGreaterThan
        | Minus
        | MinusEqual
        | MinusMinus
        | MinusGreaterThan
        | Percent
        | PercentEqual
        | Pipe
        | PipeEqual
        | PipePipe
        | Plus
        | PlusEqual
        | PlusPlus
        | Question
        | QuestionQuestion
        | QuestionQuestionEqual
        | QuestionMinusGreaterThan
        | Slash
        | SlashEqual
        | Tilde
        | At => T_OPERATOR,

        _ => return None,
    })
}
