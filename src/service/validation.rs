use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_syntax::settings::ParserSettings;
use mago_twig_syntax::parser::Parser;
use mago_twig_syntax::settings::ParserSettings as TwigParserSettings;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum EditLanguage {
    #[default]
    Php,
    Twig,
}

impl EditLanguage {
    pub fn accepts(self, arena: &LocalArena, file: FileId, source: &[u8], parser: ParserSettings) -> bool {
        match self {
            Self::Php => super::check_php_code(arena, file, source, parser),
            Self::Twig => !Parser::new(arena, file, source, TwigParserSettings::default())
                .with_recursion_limit(32)
                .parse(source, file)
                .has_errors(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Php => "PHP",
            Self::Twig => "Twig",
        }
    }
}
