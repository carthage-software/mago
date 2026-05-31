//! `get_semantic_tokens`: token-level highlighting.
//!
//! Lexes the file and classifies each token to a [`SemanticTokenKind`], skipping
//! tokens that span multiple lines (LSP semantic tokens are single-line). Emits
//! absolute byte offsets; the protocol layer delta-encodes them.

use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;

use crate::Server;
use crate::domain::SemanticTokenItem;
use crate::domain::SemanticTokenKind;
use crate::lookup;

impl Server {
    /// Semantic-highlighting tokens for `file_id`, as absolute byte spans.
    pub fn get_semantic_tokens(&mut self, file_id: FileId) -> Vec<SemanticTokenItem> {
        let Ok(file) = self.database().get(&file_id) else {
            return Vec::new();
        };

        let tokens = lookup::lex(&file);

        let mut out: Vec<SemanticTokenItem> = Vec::with_capacity(tokens.len() / 2);
        let mut prev_token: Option<TokenKind> = None;
        for token in &tokens {
            let classified = classify(token, prev_token);
            let Some(kind) = classified else {
                if !lookup::is_trivia(token.kind) {
                    prev_token = Some(token.kind);
                }

                continue;
            };

            let length = token.value.len() as u32;
            let line = file.line_number(token.start.offset);
            let last_byte = token.start.offset + length;
            // LSP semantic tokens are single-line; skip tokens that wrap.
            if file.line_number(last_byte.saturating_sub(1)) != line {
                if !lookup::is_trivia(token.kind) {
                    prev_token = Some(token.kind);
                }

                continue;
            }

            out.push(SemanticTokenItem { offset: token.start.offset, length, kind });

            if !lookup::is_trivia(token.kind) {
                prev_token = Some(token.kind);
            }
        }

        out
    }
}

#[allow(clippy::too_many_lines, clippy::enum_glob_use)]
fn classify(token: &Token<'_>, prev: Option<TokenKind>) -> Option<SemanticTokenKind> {
    use SemanticTokenKind as K;
    use TokenKind::*;

    Some(match token.kind {
        SingleLineComment | HashComment | MultiLineComment | DocBlockComment => K::Comment,
        LiteralString | PartialLiteralString | StringPart => K::String,
        LiteralInteger | LiteralFloat => K::Number,
        Variable => K::Variable,
        Identifier => match prev {
            Some(Function | Class | Interface | Trait | Enum | Const | New | Use | Namespace) => {
                match token.value.first() {
                    Some(b) if b.is_ascii_uppercase() => K::Type,
                    _ => K::Function,
                }
            }
            Some(MinusGreaterThan | QuestionMinusGreaterThan | ColonColon) => K::Function,
            _ => match token.value.first() {
                Some(b) if b.is_ascii_uppercase() => K::Type,
                _ => K::Function,
            },
        },
        QualifiedIdentifier | FullyQualifiedIdentifier => K::Namespace,
        Abstract | And | Array | As | Break | Callable | Case | Catch | Class | ClassConstant | Clone | Const
        | Continue | Declare | Default | Do | Echo | Else | ElseIf | Empty | EndDeclare | EndFor | EndForeach
        | EndIf | EndSwitch | EndWhile | Enum | Eval | Exit | Extends | False | Final | Finally | Fn | For
        | Foreach | From | Function | Global | Goto | HaltCompiler | If | Implements | Include | IncludeOnce
        | Instanceof | Insteadof | Interface | Isset | List | Match | Namespace | New | Null | Or | Parent | Print
        | Private | Protected | Public | Readonly | Require | RequireOnce | Return | Self_ | Static | Switch
        | Throw | Trait | Try | True | Unset | Use | Var | While | Xor | Yield => K::Keyword,
        TraitConstant | FunctionConstant | MethodConstant | LineConstant | FileConstant | DirConstant
        | NamespaceConstant => K::Keyword,
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
        | At => K::Operator,
        _ => return None,
    })
}
