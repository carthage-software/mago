use mago_allocator::prelude::*;
use mago_collector::pragma::Pragma;
use mago_database::file::File;
use mago_twig_syntax::comments::CommentScopes;
use mago_twig_syntax::comments::comment_lines;
use mago_twig_syntax::cst::Template;

pub(crate) fn collect<'arena, A>(
    arena: &'arena A,
    file: &File,
    template: &Template<'arena>,
) -> Vec<'arena, Pragma<'arena>, A>
where
    A: Arena,
{
    let mut pragmas = Vec::new_in(arena);
    for trivia in &template.trivia {
        if trivia.kind.is_comment() && trivia.value.windows(6).any(|text| text == b"@mago-") {
            pragmas.extend(Pragma::parse_comment(arena, file, trivia.span, comment_lines(trivia), &["lint", "linter"]));
        }
    }
    if pragmas.is_empty() {
        return pragmas;
    }

    let scopes = CommentScopes::new(template);
    for pragma in pragmas.iter_mut().filter(|pragma| pragma.own_line) {
        pragma.scope_span = Some(
            scopes
                .inline_scope(pragma.trivia_span)
                .or_else(|| scopes.following_statement(pragma.trivia_span))
                .unwrap_or(pragma.trivia_span),
        );
    }

    pragmas
}
