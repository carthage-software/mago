use fennec_ast::*;

use crate::context::LintContext;

/// A utility function to get the content of a comment trivia.
///
/// This function will return the content of a comment trivia, without the comment markers.
pub fn comment_content<'ast>(trivia: &'ast Trivia, context: &LintContext<'_>) -> Option<String> {
    match trivia.kind {
        TriviaKind::MultiLineComment => {
            let content = context.lookup(&trivia.value);
            let content = &content[2..content.len() - 2];

            Some(remove_star_prefix(content))
        }
        TriviaKind::DocBlockComment => {
            let content = context.lookup(&trivia.value);
            let content = &content[3..content.len() - 2];

            Some(remove_star_prefix(content))
        }
        TriviaKind::SingleLineComment => {
            let content = context.lookup(&trivia.value);

            Some(content[2..].to_string())
        }
        TriviaKind::HashComment => {
            let content = context.lookup(&trivia.value);

            Some(content[1..].to_string())
        }
        TriviaKind::WhiteSpace => None,
    }
}

fn remove_star_prefix(content: &str) -> String {
    let mut lines = content.lines().map(remove_stared_line_prefix);

    let mut result = String::new();
    if let Some(first) = lines.next() {
        result.push_str(first);
    }

    for line in lines {
        result.push('\n');
        result.push_str(line);
    }

    result
}

fn remove_stared_line_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();

    if trimmed.starts_with('*') {
        trimmed[1..].trim_start()
    } else {
        line
    }
}
