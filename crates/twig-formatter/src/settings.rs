use schemars::JsonSchema;

pub use mago_formatter_core::settings::EndOfLine;
use mago_formatter_core::settings::PrinterSettings;
use mago_twig_syntax::cst::Template;

use crate::tag::Tag;
use crate::tag::TagIterator;

/// Settings for Twig syntax. Template text and string contents stay unchanged.
#[derive(Debug, Clone, Copy, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct FormatSettings {
    pub print_width: usize,
    pub tab_width: usize,
    pub use_tabs: bool,
    pub end_of_line: EndOfLine,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self { print_width: 120, tab_width: 4, use_tabs: false, end_of_line: EndOfLine::Auto }
    }
}

impl FormatSettings {
    pub(super) fn resolve(mut self, template: &Template<'_>, tags: &[Tag<'_>]) -> Self {
        if self.end_of_line != EndOfLine::Auto {
            return self;
        }

        let mut offset = 0;
        for tag in TagIterator::new(tags, template.trivia.as_slice()) {
            let span = tag.span();
            if let Some(newline) = newline(&template.source_text[offset..span.start.offset as usize]) {
                self.end_of_line = newline;
                return self;
            }
            offset = span.end.offset as usize;
        }
        self.end_of_line = newline(&template.source_text[offset..]).unwrap_or(EndOfLine::Lf);

        self
    }
}

fn newline(source: &[u8]) -> Option<EndOfLine> {
    memchr::memchr2(b'\r', b'\n', source).map(|offset| {
        if source[offset..].starts_with(b"\r\n") {
            EndOfLine::Crlf
        } else if source[offset] == b'\r' {
            EndOfLine::Cr
        } else {
            EndOfLine::Lf
        }
    })
}

impl From<FormatSettings> for PrinterSettings {
    fn from(settings: FormatSettings) -> Self {
        Self {
            print_width: settings.print_width,
            tab_width: settings.tab_width,
            use_tabs: settings.use_tabs,
            end_of_line: settings.end_of_line,
        }
    }
}
