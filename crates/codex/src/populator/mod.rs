use foldhash::HashSet;
use rayon::prelude::*;

use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;

use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::reference::ReferenceSource;
use crate::reference::SymbolReferences;
use crate::symbol::SymbolIdentifier;
use crate::symbol::Symbols;
use crate::ttype::union::populate_union_type;

mod closures;
mod descendants;
mod docblock;
mod hierarchy;
mod merge;
mod methods;
mod properties;
mod signatures;
mod sorter;
mod templates;

#[cfg(test)]
mod ancestry_tests;

/// Populates the codebase metadata, resolving types and inheritance.
///
/// This function processes class-likes, function-likes, and constants to:
///
/// - Resolve type signatures (populating `TUnion` and `TAtomic` types).
/// - Calculate inheritance hierarchies (parent classes, interfaces, traits).
/// - Determine method and property origins (declaring vs. appearing).
/// - Build descendant maps for efficient lookup.
#[allow(clippy::implicit_hasher)]
pub fn populate_codebase(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: WordSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
) {
    populate_codebase_inner(codebase, symbol_references, safe_symbols, safe_symbol_members, None, false, true, true)
}

/// Populates independent type signatures in the current Rayon pool.
#[allow(clippy::implicit_hasher)]
pub fn populate_codebase_parallel(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: WordSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
) {
    populate_codebase_inner(
        codebase,
        symbol_references,
        safe_symbols,
        safe_symbol_members,
        None,
        !cfg!(target_arch = "wasm32"),
        true,
        true,
    )
}

/// Populates like [`populate_codebase_parallel`], discarding the symbol references it finds.
#[allow(clippy::implicit_hasher)]
pub fn populate_codebase_parallel_without_references(
    codebase: &mut CodebaseMetadata,
    safe_symbols: WordSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
) {
    populate_codebase_inner(
        codebase,
        &mut SymbolReferences::new(),
        safe_symbols,
        safe_symbol_members,
        None,
        !cfg!(target_arch = "wasm32"),
        true,
        false,
    )
}

/// Populates the codebase with an optional set of dirty (invalidated) symbols for targeted iteration.
///
/// When `dirty_symbols` is provided, function-like, class-type, and constant repopulation
/// uses targeted `get_mut()` lookups instead of scanning the entire HashMap — O(dirty) instead of O(all).
/// This is critical for incremental mode where only a few symbols change per cycle.
#[allow(clippy::implicit_hasher)]
pub fn populate_codebase_targeted(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: WordSet,
    safe_symbol_members: HashSet<SymbolIdentifier>,
    dirty_symbols: HashSet<SymbolIdentifier>,
) {
    populate_codebase_inner(
        codebase,
        symbol_references,
        safe_symbols,
        safe_symbol_members,
        Some(dirty_symbols),
        false,
        true,
        true,
    )
}

/// Populates fresh scans after edits that leave all declaration signatures unchanged.
#[allow(clippy::implicit_hasher)]
pub fn populate_codebase_after_body_edit(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    safe_symbols: WordSet,
    dirty_symbols: HashSet<SymbolIdentifier>,
) {
    populate_codebase_inner(
        codebase,
        symbol_references,
        safe_symbols,
        HashSet::default(),
        Some(dirty_symbols),
        false,
        false,
        true,
    )
}

fn populate_codebase_inner(
    codebase: &mut CodebaseMetadata,
    symbol_references: &mut SymbolReferences,
    mut safe_symbols: WordSet,
    mut safe_symbol_members: HashSet<SymbolIdentifier>,
    mut dirty_symbols: Option<HashSet<SymbolIdentifier>>,
    parallel: bool,
    mut repopulate_unchanged_signatures: bool,
    retain_references: bool,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let mut phase_start = tracing::enabled!(tracing::Level::TRACE).then(std::time::Instant::now);

    macro_rules! trace_phase {
        ($phase:literal) => {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(start) = &mut phase_start {
                tracing::trace!(phase = $phase, elapsed = ?start.elapsed(), "Codebase population phase completed.");
                *start = std::time::Instant::now();
            }
        };
    }

    if codebase.populate_class_like_aliases() {
        safe_symbols.clear();
        safe_symbol_members.clear();
        dirty_symbols = None;
        repopulate_unchanged_signatures = true;
    }

    let mut class_likes_to_repopulate = WordSet::default();
    if let Some(dirty) = &dirty_symbols {
        let mut dirty_class_names = WordSet::default();
        for (name, _) in dirty {
            dirty_class_names.insert(*name);
        }

        for class_name in &dirty_class_names {
            if let Some(metadata) = codebase.class_likes.get(class_name)
                && (!metadata.flags.is_populated()
                    || (!metadata.flags.is_built_in() && !safe_symbols.contains(class_name)))
            {
                class_likes_to_repopulate.insert(*class_name);
            }
        }

        for (class_name, metadata) in &codebase.class_likes {
            if !metadata.flags.is_built_in()
                && !safe_symbols.contains(class_name)
                && !class_likes_to_repopulate.contains(class_name)
            {
                class_likes_to_repopulate.insert(*class_name);
            }
        }

        for class_like_name in &class_likes_to_repopulate {
            if let Some(classlike_info) = codebase.class_likes.get_mut(class_like_name) {
                if classlike_info.flags.is_populated() {
                    for method_name in &classlike_info.methods {
                        if let Some(method) = codebase.function_likes.get_mut(&(*class_like_name, *method_name)) {
                            method.thrown_types.retain(|thrown| !thrown.inferred);
                        }
                    }
                }

                classlike_info.reset_for_population();
            }
        }
    } else {
        let mut pending = Vec::new();
        for (name, metadata) in &mut codebase.class_likes {
            if metadata.flags.is_populated() && (metadata.flags.is_built_in() || safe_symbols.contains(name)) {
                continue;
            }

            class_likes_to_repopulate.insert(*name);
            if metadata.flags.is_populated() {
                for method_name in &metadata.methods {
                    if let Some(method) = codebase.function_likes.get_mut(&(*name, *method_name)) {
                        method.thrown_types.retain(|thrown| !thrown.inferred);
                    }
                }
            }

            pending.push(metadata);
        }

        if parallel && pending.len() >= 128 && rayon::current_num_threads() > 1 {
            pending.into_par_iter().with_min_len(64).for_each(|metadata| metadata.reset_for_population());
        } else {
            for metadata in pending {
                metadata.reset_for_population();
            }
        }
    }

    let sorted_classes = sorter::sort_class_likes(codebase, &class_likes_to_repopulate);
    if parallel {
        hierarchy::populate_class_likes_parallel(&sorted_classes, codebase, symbol_references, retain_references);
    } else {
        let mut seeds = methods::OverrideSeeds::default();
        for class_name in sorted_classes {
            hierarchy::populate_class_like_metadata_iterative(class_name, codebase, symbol_references, &mut seeds);
        }
    }
    trace_phase!("hierarchy");

    let incremental = !safe_symbols.is_empty() || !safe_symbol_members.is_empty();
    let closure_sources = closures::reference_sources(codebase, repopulate_unchanged_signatures);
    let reference_source = |name: SymbolIdentifier| {
        closure_sources.get(&name).copied().unwrap_or_else(|| {
            if name.0.is_empty() {
                ReferenceSource::Symbol(true, name.1)
            } else {
                ReferenceSource::ClassLikeMember(true, name.0, name.1)
            }
        })
    };
    let is_safe = |name: SymbolIdentifier, is_closure: bool| {
        if is_closure && !closure_sources.contains_key(&name) {
            return false;
        }
        match reference_source(name) {
            ReferenceSource::Symbol(_, name) => safe_symbols.contains(&name),
            ReferenceSource::ClassLikeMember(_, class, member) => {
                safe_symbols.contains(&class) || safe_symbol_members.contains(&(class, member))
            }
            ReferenceSource::File(_, _) => false,
        }
    };

    if let Some(dirty) = &dirty_symbols {
        for dirty_key in dirty {
            if let Some(function_like_metadata) = codebase.function_likes.get_mut(dirty_key) {
                let force_repopulation = function_like_metadata.flags.is_user_defined();
                if function_like_metadata.flags.is_populated() && !force_repopulation {
                    continue;
                }

                let reference_source = reference_source(*dirty_key);

                signatures::populate_function_like_metadata(
                    function_like_metadata,
                    &codebase.symbols,
                    &reference_source,
                    symbol_references,
                    force_repopulation,
                );
            }
        }

        // Also repopulate non-dirty function_likes that are not safe but need repopulation.
        // This handles e.g. a child method when the parent class was re-added:
        // the child method isn't dirty (file didn't change) but it's not safe
        // (parent class changed), so it needs type signature repopulation.
        for (name, function_like_metadata) in &mut codebase.function_likes {
            if dirty.contains(name) || (!repopulate_unchanged_signatures && function_like_metadata.flags.is_populated())
            {
                continue;
            }

            let is_closure_or_arrow =
                function_like_metadata.get_kind().is_closure() || function_like_metadata.get_kind().is_arrow_function();

            let force_repopulation = (function_like_metadata.flags.is_user_defined()
                || (is_closure_or_arrow && !function_like_metadata.flags.is_built_in()))
                && !is_safe(*name, is_closure_or_arrow);
            if function_like_metadata.flags.is_populated() && !force_repopulation {
                continue;
            }

            let reference_source = reference_source(*name);

            signatures::populate_function_like_metadata(
                function_like_metadata,
                &codebase.symbols,
                &reference_source,
                symbol_references,
                force_repopulation,
            );
        }
        trace_phase!("function_signatures");
        drop(closure_sources);

        for class_name in &class_likes_to_repopulate {
            if let Some(metadata) = codebase.class_likes.get_mut(class_name) {
                hierarchy::populate_class_like_types(
                    *class_name,
                    metadata,
                    &codebase.symbols,
                    symbol_references,
                    true, // force: these are in the repopulate set
                );
            }
        }
        trace_phase!("class_types");

        let mut dirty_const_names: WordSet = WordSet::default();
        for (name, member) in dirty {
            if member.is_empty() {
                dirty_const_names.insert(*name);
            }
        }

        for const_name in dirty_const_names {
            if let Some(constant) = codebase.constants.get_mut(&const_name) {
                let force_repopulation = constant.flags.is_user_defined();
                if constant.flags.is_populated() && !force_repopulation {
                    continue;
                }

                populate_constant(const_name, constant, &codebase.symbols, symbol_references, force_repopulation);
            }
        }
        trace_phase!("constants");

        if incremental {
            descendants::update(codebase, &class_likes_to_repopulate);
        } else {
            let (direct, all) = build_descendants(&mut codebase.class_likes);
            codebase.direct_classlike_descendants = direct;
            codebase.all_class_like_descendants = all;
        }
        trace_phase!("descendants");

        if !incremental || !class_likes_to_repopulate.is_empty() {
            docblock::inherit_method_docblocks(codebase, &safe_symbols, Some(&class_likes_to_repopulate), parallel);
            docblock::inherit_property_docblocks(codebase, &safe_symbols, Some(&class_likes_to_repopulate), parallel);
        }
        trace_phase!("docblocks");
    } else if parallel && rayon::current_num_threads() > 1 {
        let (function_chunks, (class_chunks, constant_chunks)) = rayon::join(
            || {
                populate_chunks(codebase.function_likes.iter_mut(), |(name, function_like_metadata), references| {
                    let is_closure_or_arrow = function_like_metadata.get_kind().is_closure()
                        || function_like_metadata.get_kind().is_arrow_function();

                    let force_repopulation = (function_like_metadata.flags.is_user_defined()
                        || (is_closure_or_arrow && !function_like_metadata.flags.is_built_in()))
                        && !is_safe(*name, is_closure_or_arrow);
                    if incremental && function_like_metadata.flags.is_populated() && !force_repopulation {
                        return;
                    }

                    let reference_source = reference_source(*name);

                    signatures::populate_function_like_metadata(
                        function_like_metadata,
                        &codebase.symbols,
                        &reference_source,
                        references,
                        force_repopulation,
                    );
                })
            },
            || {
                if !incremental || !class_likes_to_repopulate.is_empty() {
                    let (direct, all) = build_descendants(&mut codebase.class_likes);
                    codebase.direct_classlike_descendants = direct;
                    codebase.all_class_like_descendants = all;
                }

                let class_chunks = populate_chunks(codebase.class_likes.iter_mut(), |(name, metadata), references| {
                    let force_repopulation =
                        if incremental { !safe_symbols.contains(name) } else { metadata.flags.is_user_defined() };
                    if incremental && metadata.flags.is_populated() && !force_repopulation {
                        return;
                    }

                    hierarchy::populate_class_like_types(
                        *name,
                        metadata,
                        &codebase.symbols,
                        references,
                        force_repopulation,
                    );
                });

                let constant_chunks = populate_chunks(codebase.constants.iter_mut(), |(name, constant), references| {
                    let force_repopulation = constant.flags.is_user_defined() && !safe_symbols.contains(name);
                    if incremental && constant.flags.is_populated() && !force_repopulation {
                        return;
                    }

                    populate_constant(*name, constant, &codebase.symbols, references, force_repopulation);
                });

                (class_chunks, constant_chunks)
            },
        );
        trace_phase!("types");
        drop(closure_sources);

        rayon::join(
            || {
                let chunks = function_chunks.into_iter().chain(class_chunks).chain(constant_chunks).collect();
                keep_references(symbol_references, chunks, retain_references);
            },
            || {
                if !incremental || !class_likes_to_repopulate.is_empty() {
                    docblock::inherit_method_docblocks(codebase, &safe_symbols, None, true);
                    docblock::inherit_property_docblocks(codebase, &safe_symbols, None, true);
                }
            },
        );
        trace_phase!("references_and_docblocks");
    } else {
        let entries = codebase.function_likes.iter_mut().filter_map(|(name, function_like_metadata)| {
            let is_closure_or_arrow =
                function_like_metadata.get_kind().is_closure() || function_like_metadata.get_kind().is_arrow_function();

            let force_repopulation = (function_like_metadata.flags.is_user_defined()
                || (is_closure_or_arrow && !function_like_metadata.flags.is_built_in()))
                && !is_safe(*name, is_closure_or_arrow);
            if incremental && function_like_metadata.flags.is_populated() && !force_repopulation {
                return None;
            }

            Some((*name, function_like_metadata, force_repopulation))
        });
        populate_entries(
            entries,
            symbol_references,
            parallel,
            retain_references,
            |(name, function_like_metadata, force_repopulation), references| {
                let reference_source = reference_source(name);

                signatures::populate_function_like_metadata(
                    function_like_metadata,
                    &codebase.symbols,
                    &reference_source,
                    references,
                    force_repopulation,
                );
            },
        );
        trace_phase!("function_signatures");
        drop(closure_sources);

        let entries = codebase.class_likes.iter_mut().filter_map(|(name, metadata)| {
            let force_repopulation =
                if incremental { !safe_symbols.contains(name) } else { metadata.flags.is_user_defined() };

            if incremental && metadata.flags.is_populated() && !force_repopulation {
                return None;
            }

            Some((*name, metadata, force_repopulation))
        });
        populate_entries(
            entries,
            symbol_references,
            parallel,
            retain_references,
            |(name, metadata, force_repopulation), references| {
                hierarchy::populate_class_like_types(name, metadata, &codebase.symbols, references, force_repopulation);
            },
        );
        trace_phase!("class_types");

        let entries = codebase.constants.iter_mut().filter_map(|(name, constant)| {
            let force_repopulation = constant.flags.is_user_defined() && !safe_symbols.contains(name);
            if incremental && constant.flags.is_populated() && !force_repopulation {
                return None;
            }

            Some((*name, constant, force_repopulation))
        });
        populate_entries(
            entries,
            symbol_references,
            parallel,
            retain_references,
            |(name, constant, force_repopulation), references| {
                populate_constant(name, constant, &codebase.symbols, references, force_repopulation);
            },
        );
        trace_phase!("constants");

        if !incremental || !class_likes_to_repopulate.is_empty() {
            let (direct, all) = build_descendants(&mut codebase.class_likes);
            codebase.direct_classlike_descendants = direct;
            codebase.all_class_like_descendants = all;
        }
        trace_phase!("descendants");

        if !incremental || !class_likes_to_repopulate.is_empty() {
            docblock::inherit_method_docblocks(codebase, &safe_symbols, None, parallel);
            docblock::inherit_property_docblocks(codebase, &safe_symbols, None, parallel);
        }
        trace_phase!("docblocks");
    }

    codebase.safe_symbols = safe_symbols;
    codebase.safe_symbol_members = safe_symbol_members;
}

fn build_descendants(class_likes: &mut WordMap<Box<ClassLikeMetadata>>) -> (WordMap<WordSet>, WordMap<WordSet>) {
    let mut direct_classlike_descendants: WordMap<WordSet> = WordMap::default();
    let mut all_classlike_descendants: WordMap<WordSet> = WordMap::default();

    for (class_like_name, class_like_metadata) in class_likes.iter() {
        for parent_interface in &class_like_metadata.all_parent_interfaces {
            all_classlike_descendants.entry(*parent_interface).or_default().insert(*class_like_name);
        }

        for parent_interface in &class_like_metadata.direct_parent_interfaces {
            direct_classlike_descendants.entry(*parent_interface).or_default().insert(*class_like_name);
        }

        for parent_class in &class_like_metadata.all_parent_classes {
            all_classlike_descendants.entry(*parent_class).or_default().insert(*class_like_name);
        }

        for used_trait in &class_like_metadata.used_traits {
            all_classlike_descendants.entry(*used_trait).or_default().insert(*class_like_name);
        }

        if let Some(parent_class) = &class_like_metadata.direct_parent_class {
            direct_classlike_descendants.entry(*parent_class).or_default().insert(*class_like_name);
        }
    }

    for (parent_name, children) in &direct_classlike_descendants {
        if let Some(parent_metadata) = class_likes.get_mut(parent_name) {
            parent_metadata.child_class_likes = Some(children.clone());
        }
    }

    (direct_classlike_descendants, all_classlike_descendants)
}

fn populate_entries<I>(
    entries: impl Iterator<Item = I>,
    references: &mut SymbolReferences,
    parallel: bool,
    retain_references: bool,
    populate: impl Fn(I, &mut SymbolReferences) + Sync,
) where
    I: Send,
{
    if !parallel {
        for entry in entries {
            populate(entry, references);
        }
        return;
    }

    keep_references(references, populate_chunks(entries, populate), retain_references);
}

pub(crate) fn keep_references(references: &mut SymbolReferences, chunks: Vec<SymbolReferences>, retain: bool) {
    if retain {
        for chunk in chunks {
            references.extend(chunk);
        }
    } else {
        chunks.into_par_iter().for_each(drop);
    }
}

fn populate_chunks<I>(
    entries: impl Iterator<Item = I>,
    populate: impl Fn(I, &mut SymbolReferences) + Sync,
) -> Vec<SymbolReferences>
where
    I: Send,
{
    let entries = entries.collect::<Vec<_>>();
    if entries.len() < 128 || rayon::current_num_threads() == 1 {
        let mut references = SymbolReferences::new();
        for entry in entries {
            populate(entry, &mut references);
        }
        return vec![references];
    }

    let chunk_size = entries.len().div_ceil(rayon::current_num_threads() * 4).max(64);
    entries
        .into_par_iter()
        .chunks(chunk_size)
        .map(|entries| {
            let mut references = SymbolReferences::new();
            for entry in entries {
                populate(entry, &mut references);
            }
            references
        })
        .collect()
}

/// Populates a single constant's type metadata.
fn populate_constant(
    name: Word,
    constant: &mut ConstantMetadata,
    symbols: &Symbols,
    symbol_references: &mut SymbolReferences,
    force_repopulation: bool,
) {
    for attribute_metadata in &constant.attributes {
        symbol_references.add_symbol_reference_to_symbol(name, attribute_metadata.name, true);
    }

    if let Some(type_metadata) = &mut constant.type_metadata {
        populate_union_type(
            &mut type_metadata.type_union,
            symbols,
            Some(&ReferenceSource::Symbol(true, name)),
            symbol_references,
            force_repopulation,
        );
    }

    if let Some(inferred_type) = &mut constant.inferred_type {
        populate_union_type(
            inferred_type,
            symbols,
            Some(&ReferenceSource::Symbol(true, name)),
            symbol_references,
            force_repopulation,
        );
    }

    constant.flags |= MetadataFlags::POPULATED;
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::fmt::Write;

    use foldhash::HashSet;
    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_php_version::PHPVersion;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;
    use mago_word::empty_word;
    use mago_word::word;

    use super::populate_codebase;
    use super::populate_codebase_parallel;
    use super::populate_entries;
    use crate::reference::ReferenceOrigin;
    use crate::reference::SymbolReferences;
    use crate::scanner::scan_program;

    #[test]
    fn parallel_docblock_inheritance_matches_serial() -> Result<(), Box<dyn std::error::Error>> {
        let mut source = String::from(
            "<?php
interface Traversable {}
interface Iterator extends Traversable { public function current(): mixed; public function key(): mixed; }
interface IteratorAggregate extends Traversable { public function getIterator(): Iterator; }
/** @template T */
interface Source {
    /** @return list<T> */
    public function all(): array;
    /** @param T $value */
    public function add(mixed $value): void;
}
class Strings implements IteratorAggregate {
    /** @return Iterator<int, string> */
    public function getIterator(): Iterator { return $this->getIterator(); }
}
class Invokable {
    /** @return list<int> */
    public function __invoke(): array { return []; }
}
class Consumer {
    /** @param callable(): list<int> $callback */
    public function take(Invokable $callback): void {}
    /** @param iterable<int, string> $values */
    public function read(Strings $values): void {}
}
",
        );
        for index in 0..600 {
            writeln!(
                source,
                "
/** @implements Source<Strings> */
class Store{index} implements Source {{
    public function all(): array {{ return []; }}
    public function add(mixed $value): void {{}}
}}
class Wrapped{index} extends Strings {{
    public function getIterator(): Iterator {{ return $this->getIterator(); }}
}}
class Called{index} extends Invokable {{
    public function __invoke(): array {{ return []; }}
}}
class Reader{index} extends Consumer {{
    public function take(Called{index} $callback): void {{}}
    public function read(Wrapped{index} $values): void {{}}
}}
"
            )?;
        }

        let file = File::ephemeral(Cow::Borrowed(b"docblocks.php"), Cow::Owned(source.into_bytes()));
        let arena = LocalArena::new();
        let program = parse_file(&arena, &file);
        if program.has_errors() {
            return Err(format!("docblock fixture did not parse: {:?}", program.errors).into());
        }
        let names = NameResolver::new(&arena).resolve(program);
        let scanned = scan_program(&arena, &file, program, &names, PHPVersion::LATEST);

        let populate = |threads, parallel| -> Result<_, rayon::ThreadPoolBuildError> {
            let mut codebase = scanned.clone();
            let mut references = SymbolReferences::new();
            rayon::ThreadPoolBuilder::new().num_threads(threads).build()?.install(|| {
                let populate = if parallel { populate_codebase_parallel } else { populate_codebase };
                populate(&mut codebase, &mut references, WordSet::default(), HashSet::default());
            });
            Ok(codebase)
        };

        if populate(1, false)? != populate(4, true)? {
            return Err("parallel docblock inheritance changed metadata".into());
        }

        Ok(())
    }

    #[test]
    fn parallel_population_matches_serial_metadata_and_references() -> Result<(), Box<dyn std::error::Error>> {
        let mut source = String::from("<?php class base { public function method(): base { return $this; } }\n");
        for index in 0..160 {
            writeln!(
                source,
                "
/** @template T of base */
class Child{index} extends base {{
    /** @var array<string, T> */
    public array $values = [];
    public const NAME = 'child';
    /** @inheritDoc */
    public function method(): base {{ return $this; }}
}}
/** @return list<Child{index}> */
function function_{index}(Child{index} $value): array {{ return [$value]; }}
/** @var Child{index}|null */
const CONSTANT_{index} = null;
"
            )?;
        }

        let file = File::ephemeral(Cow::Borrowed(b"population.php"), Cow::Owned(source.into_bytes()));
        let arena = LocalArena::new();
        let program = parse_file(&arena, &file);
        if program.has_errors() {
            return Err(format!("population fixture did not parse: {:?}", program.errors).into());
        }
        let names = NameResolver::new(&arena).resolve(program);
        let scanned = scan_program(&arena, &file, program, &names, PHPVersion::LATEST);

        let collision_file = File::ephemeral(
            Cow::Borrowed(b"collision.php"),
            Cow::Borrowed(b"<?php $closure = function (): base { return new base; };"),
        );
        let collision_program = parse_file(&arena, &collision_file);
        let collision_names = NameResolver::new(&arena).resolve(collision_program);
        let collision = scan_program(&arena, &collision_file, collision_program, &collision_names, PHPVersion::LATEST);
        let mut with_collision = scanned.clone();
        with_collision.extend(collision);

        for scanned in [scanned, with_collision] {
            let populate = |threads, parallel| -> Result<_, rayon::ThreadPoolBuildError> {
                let mut codebase = scanned.clone();
                let mut references = SymbolReferences::new();
                rayon::ThreadPoolBuilder::new().num_threads(threads).build()?.install(|| {
                    let populate = if parallel { populate_codebase_parallel } else { populate_codebase };
                    populate(&mut codebase, &mut references, WordSet::default(), HashSet::default());
                });
                Ok((codebase, references))
            };

            let expected = populate(1, false)?;
            for threads in [1, 4] {
                if expected != populate(threads, true)? {
                    return Err(
                        format!("parallel population changed metadata or references with {threads} threads").into()
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn ordered_chunks_preserve_signature_and_body_insertion_order() {
        let origins = [
            ReferenceOrigin::Symbol((word("function"), empty_word())),
            ReferenceOrigin::Symbol((word("class"), word("method"))),
            ReferenceOrigin::File(word("file.php")),
        ];
        let target = (word("target"), empty_word());
        let Ok(pool) = rayon::ThreadPoolBuilder::new().num_threads(2).build() else {
            panic!("failed to create test thread pool");
        };
        for signature_first in [false, true] {
            let populate = |entry, references: &mut SymbolReferences| {
                if entry == 63 || entry == 64 {
                    let in_signature = (entry == 63) == signature_first;
                    for origin in origins {
                        references.add_reference(origin, target, in_signature);
                    }
                }
            };
            let mut expected = SymbolReferences::new();
            populate_entries(0..256, &mut expected, false, true, populate);
            let mut actual = SymbolReferences::new();
            pool.install(|| populate_entries(0..256, &mut actual, true, true, populate));
            assert_eq!(actual, expected, "cross-chunk order changed with signature_first={signature_first}");
        }
    }
}
