use mago_allocator::Arena;
use mago_database::file::File;
use mago_reporting::IssueCollection;
use mago_twig_linter::Linter;
use mago_twig_linter::settings::Settings;
use mago_twig_syntax::parser::Parser;
use mago_twig_syntax::settings::ParserSettings;

/// Parses `file` and lints it with the rules in `only`, or the enabled rules when `only` is empty.
#[must_use]
pub fn lint<A>(arena: &A, file: &File, settings: &Settings, only: &[&str]) -> IssueCollection
where
    A: Arena,
{
    let template =
        Parser::new(arena, file.id, &file.contents, ParserSettings::default()).parse(&file.contents, file.id);
    assert!(!template.has_errors(), "{:?}", template.errors);

    let only: Vec<String> = only.iter().map(|code| (*code).to_owned()).collect();
    Linter::new(arena, settings, (!only.is_empty()).then_some(only.as_slice()), false).lint(file, template)
}
