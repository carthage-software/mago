use std::borrow::Cow;

use bumpalo::Bump;
use rust_embed::RustEmbed;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

use crate::Prelude;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets"]
#[prefix = "mago_prelude_"]
#[include = "*.php"]
struct PHPAssets;

pub(crate) fn build_prelude_internal() -> Prelude {
    let arena = Bump::new();
    let database = get_prelude_database();
    let read_db = database.read_only();
    let mut metadata = get_prelude_metadata(&arena, &read_db);
    let mut symbol_references = SymbolReferences::default();

    populate_codebase(&mut metadata, &mut symbol_references, Default::default(), Default::default());

    Prelude { database, metadata, symbol_references }
}

fn get_prelude_database() -> mago_database::Database<'static> {
    let configuration =
        mago_database::DatabaseConfiguration::new(std::path::Path::new("/"), vec![], vec![], vec![], vec![])
            .into_static();
    let mut db = mago_database::Database::new(configuration);
    for filename in PHPAssets::iter() {
        let Some(embedded) = PHPAssets::get(&filename) else { continue };

        let content = match embedded.data {
            Cow::Borrowed(slice) => {
                let Ok(string) = std::str::from_utf8(slice) else { continue };

                Cow::Borrowed(string)
            }
            Cow::Owned(vector) => {
                let Ok(string) = String::from_utf8(vector) else { continue };

                Cow::Owned(string)
            }
        };

        db.add(File::new(filename, FileType::Builtin, None, content));
    }

    db
}

fn get_prelude_metadata(arena: &Bump, database: &ReadDatabase) -> CodebaseMetadata {
    let mut metadata = CodebaseMetadata::default();
    for file in database.files() {
        let file_metadata = scan_file_for_metadata(&file, arena);

        metadata.extend(file_metadata);
    }

    metadata
}

fn scan_file_for_metadata(source_file: &File, arena: &Bump) -> CodebaseMetadata {
    let (program, _) = parse_file(arena, source_file);
    let resolver = NameResolver::new(arena);
    let resolved_names = resolver.resolve(program);

    scan_program(arena, source_file, program, &resolved_names)
}
