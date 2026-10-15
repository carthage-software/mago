//! Validate and format the body of a generated class or enum.

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_formatter::Formatter;
use mago_names::resolver::NameResolver;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;

use crate::Server;

pub(super) fn format_body(
    server: &Server,
    arena: &LocalArena,
    file: &File,
    settings: ParserSettings,
) -> Option<String> {
    let program = parse_file_borrowed_with_settings(arena, file, settings);
    let names = NameResolver::new(arena).resolve(program);
    if program.has_errors() || !SemanticsChecker::new(server.php_version).check(file, program, &names).is_empty() {
        return None;
    }
    let formatted = Formatter::new(arena, server.php_version, server.formatter).format_file(file).ok()?;
    let start = formatted.iter().position(|byte| *byte == b'{')? + 1;
    let end = formatted.iter().rposition(|byte| *byte == b'}')?;
    Some(std::str::from_utf8(&formatted[start..end]).ok()?.trim_matches(['\r', '\n']).to_owned())
}
