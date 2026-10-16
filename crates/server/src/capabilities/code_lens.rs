//! Code-lens anchors for named class, function, and constant declarations.

use std::sync::Arc;

use foldhash::HashMap;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_database::file::FileType;

use crate::Server;
use crate::domain::{CodeLensItem, CodeLensKind, SymbolLocation};
use crate::file_analysis::FileAnalysis;
use crate::semantic_index::{SemanticIndex, SymbolKey, SymbolKind};
use crate::symbol_resolution::{may_refer_to, resolved_symbol_name, short_name};

/// Each lens of a file with the references found for it.
pub(crate) type LensReferences = Vec<(CodeLensItem, Arc<[SymbolLocation]>)>;

impl Server {
    /// Find declarations without collecting references until the client resolves a lens.
    pub fn get_lens(&mut self, file_id: FileId) -> Vec<CodeLensItem> {
        let Some((analysis, length)) = self.lens_file(file_id) else { return Vec::new() };
        lenses(analysis.semantic(), length).map(|(item, _)| item).collect()
    }

    /// Find where a lens listed before its file changed is anchored now.
    ///
    /// A lens keeps its rank among the declarations of its name while their number is unchanged;
    /// otherwise the declaration nearest its listed offset, counted from either end of the file, wins.
    /// Returns `None` once the file no longer declares the name.
    pub fn locate_lens(&mut self, file_id: FileId, item: CodeLensItem) -> Option<CodeLensItem> {
        let (analysis, length) = self.lens_file(file_id)?;
        let index = analysis.semantic();
        let symbol = index.symbols.iter().position(|symbol| {
            *symbol.name == *item.name && lens_kind(symbol.kind).is_some_and(|kind| kind == item.kind)
        })?;
        let declarations = || {
            index
                .occurrences
                .iter()
                .filter(move |entry| entry.declaration && entry.symbol as usize == symbol)
                .map(|entry| entry.selection_range.start)
                .enumerate()
        };
        let siblings = declarations().count() as u32;
        let ranked = if siblings == item.siblings { declarations().nth(item.ordinal as usize) } else { None };
        let (ordinal, offset) = ranked.or_else(|| {
            declarations()
                .min_by_key(|(_, offset)| offset.abs_diff(item.offset).min((length - offset).abs_diff(item.from_end)))
        })?;
        Some(CodeLensItem { offset, from_end: length - offset, ordinal: ordinal as u32, siblings, ..item })
    }

    /// Count the references of every lens in one pass over the host files.
    ///
    /// Each count equals the number of [`get_references`](Self::get_references) results
    /// for that lens without its declaration.
    pub fn get_lens_counts(&mut self, file_id: FileId) -> Vec<(CodeLensItem, usize)> {
        self.lens_references(file_id).iter().map(|(item, locations)| (item.clone(), locations.len())).collect()
    }

    /// The references of a lens listed before its file changed, at its current declaration.
    pub fn get_lens_references(
        &mut self,
        file_id: FileId,
        item: CodeLensItem,
    ) -> Option<(CodeLensItem, Arc<[SymbolLocation]>)> {
        let item = self.locate_lens(file_id, item)?;
        let (_, locations) = self.lens_references(file_id).iter().find(|(lens, _)| lens.offset == item.offset)?;
        Some((item, Arc::clone(locations)))
    }

    /// Every lens of a file with its references, kept until the workspace changes.
    fn lens_references(&mut self, file_id: FileId) -> &[(CodeLensItem, Arc<[SymbolLocation]>)] {
        if !self.lens_references.contains_key(&file_id) {
            let lenses = self.find_lens_references(file_id);
            self.lens_references.insert(file_id, lenses);
        }

        &self.lens_references[&file_id]
    }

    fn find_lens_references(&mut self, file_id: FileId) -> LensReferences {
        let Some((analysis, length)) = self.lens_file(file_id) else { return Vec::new() };
        let mut slots: HashMap<(SymbolKind, &[u8]), usize> = HashMap::default();
        let lenses: Vec<_> = lenses(analysis.semantic(), length)
            .map(|(item, symbol)| {
                let next = slots.len();
                (item, *slots.entry((symbol.kind, &symbol.name)).or_insert(next))
            })
            .collect();
        let mut locations = vec![Vec::new(); slots.len()];
        if !slots.is_empty() {
            let hosts: Vec<_> =
                self.database().files().filter(|file| file.file_type == FileType::Host).map(|file| file.id).collect();
            let mut candidates = Vec::new();
            let mut lowered = Vec::new();
            for id in hosts {
                let Some(analysis) = self.file_analysis_for(id) else { continue };
                let index = analysis.semantic();
                candidates.clear();
                candidates.extend(index.symbols.iter().map(|symbol| {
                    lens_kind(symbol.kind).is_some()
                        && (slots.contains_key(&(symbol.kind, &*symbol.name))
                            || (matches!(symbol.kind, SymbolKind::Function | SymbolKind::Constant)
                                && slots.contains_key(&(symbol.kind, short_name(&symbol.name)))))
                }));
                if !candidates.contains(&true) {
                    continue;
                }
                let Ok(file) = self.database().get(&id) else { continue };
                for entry in
                    index.occurrences.iter().filter(|entry| !entry.declaration && candidates[entry.symbol as usize])
                {
                    let symbol = &index.symbols[entry.symbol as usize];
                    let mut name = resolved_symbol_name(self.codebase(), &file.contents, index, entry);
                    if symbol.kind != SymbolKind::Constant && name.iter().any(u8::is_ascii_uppercase) {
                        lowered.clear();
                        lowered.extend(name.iter().map(u8::to_ascii_lowercase));
                        name = &lowered;
                    }
                    if let Some(&slot) = slots.get(&(symbol.kind, name))
                        && may_refer_to(symbol.kind, &symbol.name, name)
                    {
                        locations[slot].push(SymbolLocation { file: id, range: entry.range });
                    }
                }
            }
        }

        let locations: Vec<Arc<[SymbolLocation]>> = locations
            .into_iter()
            .map(|mut found: Vec<SymbolLocation>| {
                found.sort_unstable_by_key(|location| (location.file, location.range.start, location.range.end));
                Arc::from(found)
            })
            .collect();
        lenses.into_iter().map(|(item, slot)| (item, Arc::clone(&locations[slot]))).collect()
    }

    fn lens_file(&mut self, file_id: FileId) -> Option<(Arc<FileAnalysis>, u32)> {
        let length = self.database().get(&file_id).ok()?.contents.len() as u32;
        Some((self.file_analysis_for(file_id)?, length))
    }
}

fn lenses(index: &SemanticIndex, length: u32) -> impl Iterator<Item = (CodeLensItem, &SymbolKey)> {
    let declarations = || index.occurrences.iter().filter(|entry| entry.declaration);
    let mut ranks = vec![(0, 0); index.symbols.len()];
    for entry in declarations() {
        ranks[entry.symbol as usize].1 += 1;
    }
    declarations().filter_map(move |entry| {
        let symbol = &index.symbols[entry.symbol as usize];
        let kind = lens_kind(symbol.kind)?;
        let (ordinal, siblings) = &mut ranks[entry.symbol as usize];
        let offset = entry.selection_range.start;
        let item = CodeLensItem {
            offset,
            from_end: length - offset,
            kind,
            name: symbol.name.to_vec(),
            ordinal: *ordinal,
            siblings: *siblings,
        };
        *ordinal += 1;
        Some((item, symbol))
    })
}

fn lens_kind(kind: SymbolKind) -> Option<CodeLensKind> {
    match kind {
        SymbolKind::ClassLike => Some(CodeLensKind::ClassLike),
        SymbolKind::Function => Some(CodeLensKind::Function),
        SymbolKind::Constant => Some(CodeLensKind::Constant),
        SymbolKind::Namespace | SymbolKind::Variable => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::path::Path;
    use std::sync::Arc;

    use foldhash::HashSet;
    use mago_allocator::LocalArena;
    use mago_codex::metadata::CodebaseMetadata;
    use mago_codex::populator::populate_codebase;
    use mago_codex::reference::SymbolReferences;
    use mago_codex::scanner::scan_program;
    use mago_database::Database;
    use mago_database::DatabaseConfiguration;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_php_version::PHPVersion;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;

    use super::*;

    fn server(files: &[(&str, FileType, &str)]) -> Server {
        let mut database = Database::new(DatabaseConfiguration::new(Path::new("/"), vec![], vec![], vec![], vec![]));
        let mut codebase = CodebaseMetadata::new();
        for (name, file_type, source) in files {
            let file = File::new(name.as_bytes().to_vec().into(), *file_type, None, source.as_bytes().to_vec().into());
            let arena = LocalArena::new();
            let program = parse_file(&arena, &file);
            assert!(!program.has_errors(), "{source}");
            let names = NameResolver::new(&arena).resolve(program);
            codebase.extend(scan_program(&arena, &file, program, &names, PHPVersion::PHP85));
            database.add(file);
        }
        populate_codebase(&mut codebase, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());
        Server::new(
            database,
            codebase,
            SymbolReferences::new(),
            crate::Settings {
                php_version: PHPVersion::PHP85,
                features: crate::Features { analyzer: false, linter: false, formatter: false },
                parser: mago_syntax::settings::ParserSettings::default(),
                analyzer: mago_analyzer::settings::Settings::default(),
                linter: mago_linter::settings::Settings::default(),
                formatter: mago_formatter::settings::FormatSettings::default(),
                plugin_registry: Arc::new(mago_analyzer::plugin::PluginRegistry::new()),
                excludes: crate::Excludes::default(),
            },
        )
    }

    #[test]
    fn lens_counts_match_references_for_each_lens() {
        let global = "<?php
            class Shared {} function Shared(): void {} const Shared = 1; const SHARED = 2;
            function work(): void {} const FLAG = 1; const flag = 2; function idle(): void {}
            if (true) { function twice(): void {} } else { function twice(): void {} }
            new Shared(); Shared(); echo SHARED;";
        let library = "<?php namespace Lib; class Thing {} function work(): void {} const FLAG = 4;
            work(); echo FLAG; new Thing(); echo \\FLAG;";
        let caller = "<?php namespace App; use Lib\\Thing;
            new \\Shared(); SHARED(); Shared(); echo Shared, SHARED, \\FLAG, flag; WORK(); twice(); vendored();
            new Thing(); \\Lib\\work(); echo \\Lib\\FLAG; new shared();";
        let imports = "<?php namespace Imports; use function work as go; use const FLAG as VALUE; use Shared as Alias;
            go(); echo VALUE; new Alias();";
        let shadow = "<?php namespace Shadow; function work(): void {} const FLAG = 3;
            work(); echo FLAG; \\work(); echo \\FLAG; new Shared();";
        let vendor = "<?php function vendored(): void {} work(); vendored(); new Shared(); echo FLAG;";
        let mut server = server(&[
            ("global.php", FileType::Host, global),
            ("library.php", FileType::Host, library),
            ("caller.php", FileType::Host, caller),
            ("imports.php", FileType::Host, imports),
            ("shadow.php", FileType::Host, shadow),
            ("vendor.php", FileType::Vendored, vendor),
        ]);
        for (name, expected) in [
            ("global.php", [4, 3, 1, 2, 4, 5, 1, 0, 1, 1].as_slice()),
            ("library.php", &[3, 2, 2]),
            ("caller.php", &[]),
            ("imports.php", &[]),
            ("shadow.php", &[1, 1]),
            ("vendor.php", &[1]),
        ] {
            let file = server.database().get_id(name.as_bytes()).unwrap();
            let counts = server.get_lens_counts(file);
            let lenses = server.get_lens(file);
            let offsets: Vec<_> = lenses.iter().map(|item| item.offset).collect();
            assert_eq!(counts.iter().map(|(item, _)| item.offset).collect::<Vec<_>>(), offsets, "{name}");
            let references: Vec<_> = offsets.iter().map(|offset| server.get_references(file, *offset, false)).collect();
            assert_eq!(
                counts.iter().map(|(_, count)| *count).collect::<Vec<_>>(),
                references.iter().map(Vec::len).collect::<Vec<_>>(),
                "{name}"
            );

            assert_eq!(references.iter().map(Vec::len).collect::<Vec<_>>(), expected, "{name}");
            server.invalidate_artifacts(&[]);
            for (item, references) in lenses.into_iter().zip(&references) {
                let (_, locations) = server.get_lens_references(file, item).unwrap();
                assert_eq!(&*locations, references.as_slice(), "{name}");
            }
        }
    }
}
