use mago_ast::*;
use mago_interner::ThreadedInterner;

/// A utility function to get the content of a comment trivia.
///
/// This function will return the content of a comment trivia, without the comment markers.
#[inline]
pub fn comment_lines<'a>(trivia: &Trivia, interner: &'a ThreadedInterner) -> Vec<&'a str> {
    match trivia.kind {
        TriviaKind::MultiLineComment => {
            let content = interner.lookup(&trivia.value);
            let content = &content[2..content.len() - 2];

            remove_star_prefix(content)
        }
        TriviaKind::DocBlockComment => {
            let content = interner.lookup(&trivia.value);
            let content = &content[3..content.len() - 2];

            remove_star_prefix(content)
        }
        TriviaKind::SingleLineComment => {
            let content = interner.lookup(&trivia.value);

            vec![&content[2..]]
        }
        TriviaKind::HashComment => {
            let content = interner.lookup(&trivia.value);

            vec![&content[1..]]
        }
        TriviaKind::WhiteSpace => vec![],
    }
}

#[inline]
fn remove_star_prefix(content: &str) -> Vec<&str> {
    content.lines().map(remove_stared_line_prefix).collect::<Vec<_>>()
}

#[inline]
fn remove_stared_line_prefix(line: &str) -> &str {
    if let Some(stripped) = line.trim_start().strip_prefix('*') {
        stripped.trim_start()
    } else {
        line
    }
}
