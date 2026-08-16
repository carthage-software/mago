use std::ops::Range;
use std::sync::Arc;
use std::sync::OnceLock;

use foldhash::HashMap;
use foldhash::HashSet;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::ReferenceOrigin;
use mago_codex::reference::SymbolReferenceKind;
use mago_codex::reference::SymbolReferences;
use mago_codex::symbol::SymbolIdentifier;
use mago_codex::ttype::union::TUnion;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_extension::PayloadReader;
use mago_extension::PayloadWriter;
use mago_extension::source::SourceSnapshot;
use mago_names::ResolvedNames;
use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::cst::Program;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;
use mago_text_edit::TextRange;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::ascii_lowercase_word;
use mago_word::empty_word;
use mago_word::word;

use crate::analysis_result::AnalysisResult;
use crate::artifacts::AnalysisArtifacts;

use super::ExternalAnalysisSession;
use super::ExternalPlugin;
use super::error::ExternalAnalyzerError;
use super::error::protocol;
use super::protocol;

pub(super) const BEFORE_ANALYSIS_REQUEST: u16 = 5;
pub(super) const AFTER_FILE_ANALYSIS_REQUEST: u16 = 6;
pub(super) const AFTER_ANALYSIS_REQUEST: u16 = 7;
const ANALYSIS_QUERY_REQUEST: u16 = 8;
pub(super) const AFTER_FILE_ANALYSIS_BATCH_REQUEST: u16 = 9;
const SYMBOL_REFERENCE_QUERY_REQUEST: u16 = 10;
const BEFORE_ANALYSIS_RESPONSE: u16 = 0x8005;
const AFTER_FILE_ANALYSIS_RESPONSE: u16 = 0x8006;
const AFTER_ANALYSIS_RESPONSE: u16 = 0x8007;
const ANALYSIS_QUERY_RESPONSE: u16 = 0x8008;
const AFTER_FILE_ANALYSIS_BATCH_RESPONSE: u16 = 0x8009;
const SYMBOL_REFERENCE_QUERY_RESPONSE: u16 = 0x800A;

const GET_EXPRESSION_TYPES: u8 = 1;
const GET_ALL_EXPRESSION_TYPES: u8 = 2;
const GET_INFERRED_RETURN_TYPES: u8 = 3;
const GET_INFERRED_YIELD_KEY_TYPES: u8 = 4;
const GET_INFERRED_YIELD_VALUE_TYPES: u8 = 5;
const GET_SOURCE_FILE: u8 = 6;
const GET_REFERENCES_TO: u8 = 1;
const GET_REFERENCES_FROM: u8 = 2;
const MAXIMUM_ISSUES: usize = 1_000_000;
const MAXIMUM_ANNOTATIONS: usize = 0x0001_0000;
const MAXIMUM_EDITS: usize = 0x0001_0000;
const MAXIMUM_NOTES: usize = 0x0001_0000;
const MAXIMUM_TYPE_QUERIES: usize = 1_000_000;
const MAXIMUM_REFERENCE_QUERIES: usize = 1_000_000;
const MAXIMUM_REFERENCES: usize = 10_000_000;
const EXPRESSION_TYPE_PREFETCH: usize = 16;

#[derive(Debug, Default)]
pub(super) struct LifecycleEffects {
    pub issues: IssueCollection,
    pub references: SymbolReferences,
    pub references_by_file: HashMap<FileId, SymbolReferences>,
}

pub(super) struct SymbolReferenceStore<'analysis> {
    references: &'analysis SymbolReferences,
    by_target: OnceLock<HashMap<SymbolIdentifier, Vec<RecordedReference>>>,
    by_source: OnceLock<HashMap<ReferenceOrigin, Vec<RecordedReference>>>,
}

impl<'analysis> SymbolReferenceStore<'analysis> {
    pub fn new(references: &'analysis SymbolReferences) -> Self {
        Self { references, by_target: OnceLock::new(), by_source: OnceLock::new() }
    }

    fn references_to(&self, target: SymbolIdentifier) -> &[RecordedReference] {
        self.by_target
            .get_or_init(|| {
                let mut index = HashMap::<SymbolIdentifier, Vec<RecordedReference>>::default();
                self.references.for_each_reference(|source, target, kind| {
                    index.entry(target).or_default().push(RecordedReference { source, target, kind });
                });
                for references in index.values_mut() {
                    references.sort_unstable();
                }
                index
            })
            .get(&target)
            .map_or(&[], Vec::as_slice)
    }

    fn references_from(&self, source: ReferenceOrigin) -> &[RecordedReference] {
        self.by_source
            .get_or_init(|| {
                let mut index = HashMap::<ReferenceOrigin, Vec<RecordedReference>>::default();
                self.references.for_each_reference(|source, target, kind| {
                    index.entry(source).or_default().push(RecordedReference { source, target, kind });
                });
                for references in index.values_mut() {
                    references.sort_unstable();
                }
                index
            })
            .get(&source)
            .map_or(&[], Vec::as_slice)
    }
}

#[derive(Debug)]
pub struct FileAnalysisSnapshot {
    file_id: mago_database::file::FileId,
    name: Arc<[u8]>,
    size: u32,
    encoded_source: Box<[u8]>,
    encoded_types: Box<[u8]>,
    expression_types: HashMap<(u32, u32), Range<usize>>,
    inferred_return_types: Vec<Range<usize>>,
    inferred_yield_key_types: Vec<Range<usize>>,
    inferred_yield_value_types: Vec<Range<usize>>,
    references: ReferenceSummary,
    node_analysis_targets: usize,
}

#[derive(Debug, Clone, Copy)]
struct ReferenceSummary {
    body: u64,
    signature: u64,
    maps: u64,
}

/// Number of completed files sent through one external after-file request.
pub const AFTER_FILE_ANALYSIS_BATCH_SIZE: usize = 32;

impl FileAnalysisSnapshot {
    /// Builds a compact, thread-safe snapshot of one file's lazy analysis data.
    ///
    /// # Errors
    ///
    /// Returns an error when the source snapshot or an inferred type cannot be represented by the extension protocol.
    pub fn new(
        file: &File,
        program: &Program<'_>,
        resolved_names: &ResolvedNames<'_>,
        artifacts: &AnalysisArtifacts,
        node_analysis_targets: Option<&[bool; u8::MAX as usize + 1]>,
    ) -> Result<Self, ExternalAnalyzerError> {
        let source =
            SourceSnapshot::complete_with_targets(program, resolved_names, node_analysis_targets).map_err(|error| {
                protocol(format!("failed to retain syntax for `{}`: {error}", String::from_utf8_lossy(&file.name)))
            })?;
        let node_analysis_targets = source.target_count();

        let mut source_writer = PayloadWriter::with_capacity(source.encoded_len());
        source.write_to(&mut source_writer).map_err(|error| {
            protocol(format!("failed to encode retained syntax for `{}`: {error}", String::from_utf8_lossy(&file.name)))
        })?;

        let mut writer = PayloadWriter::new();
        let mut type_handles = Vec::new();
        let mut expression_types =
            HashMap::with_capacity_and_hasher(artifacts.expression_types.len(), foldhash::fast::RandomState::default());
        for (span, union) in &artifacts.expression_types {
            expression_types.insert(
                *span,
                encode_snapshot_type(&mut writer, union, &mut type_handles).map_err(|error| {
                    protocol(format!(
                        "failed to retain expression type at {}:{} in `{}`: {error}",
                        span.0,
                        span.1,
                        String::from_utf8_lossy(&file.name)
                    ))
                })?,
            );
        }

        let inferred_return_types = encode_snapshot_types(
            &mut writer,
            artifacts.inferred_return_types.iter().map(AsRef::as_ref),
            &mut type_handles,
        )?;
        let inferred_yield_key_types =
            encode_snapshot_types(&mut writer, &artifacts.inferred_yield_key_types, &mut type_handles)?;
        let inferred_yield_value_types =
            encode_snapshot_types(&mut writer, &artifacts.inferred_yield_value_types, &mut type_handles)?;

        Ok(Self {
            file_id: file.id,
            name: Arc::from(file.name.as_ref()),
            size: file.size,
            encoded_source: source_writer.finish().into_boxed_slice(),
            encoded_types: writer.finish().into_boxed_slice(),
            expression_types,
            inferred_return_types,
            inferred_yield_key_types,
            inferred_yield_value_types,
            references: ReferenceSummary::from(&artifacts.symbol_references),
            node_analysis_targets,
        })
    }

    #[must_use]
    pub const fn file_id(&self) -> mago_database::file::FileId {
        self.file_id
    }

    #[must_use]
    pub const fn has_node_analysis_targets(&self) -> bool {
        self.node_analysis_targets != 0
    }
}

impl From<&SymbolReferences> for ReferenceSummary {
    fn from(references: &SymbolReferences) -> Self {
        Self {
            body: references.count_body_references() as u64,
            signature: references.count_signature_references() as u64,
            maps: references.total_map_entries() as u64,
        }
    }
}

impl ReferenceSummary {
    fn write_to(self, writer: &mut PayloadWriter) {
        writer.write_u64(self.body);
        writer.write_u64(self.signature);
        writer.write_u64(self.maps);
    }
}

fn encode_snapshot_types<'type_info>(
    writer: &mut PayloadWriter,
    types: impl IntoIterator<Item = &'type_info TUnion>,
    handles: &mut Vec<&'type_info TUnion>,
) -> Result<Vec<Range<usize>>, ExternalAnalyzerError> {
    types.into_iter().map(|ty| encode_snapshot_type(writer, ty, handles)).collect()
}

fn encode_snapshot_type<'type_info>(
    writer: &mut PayloadWriter,
    ty: &'type_info TUnion,
    handles: &mut Vec<&'type_info TUnion>,
) -> Result<Range<usize>, ExternalAnalyzerError> {
    handles.clear();
    let start = writer.len();
    protocol::encode_union_snapshot(writer, ty, handles, 0)?;
    Ok(start..writer.len())
}

pub(super) enum AnalysisStore<'analysis> {
    File {
        file: &'analysis File,
        program: &'analysis Program<'analysis>,
        resolved_names: &'analysis ResolvedNames<'analysis>,
        artifacts: &'analysis AnalysisArtifacts,
        node_analysis_targets: Option<&'analysis [bool; u8::MAX as usize + 1]>,
    },
    Project(&'analysis [Arc<FileAnalysisSnapshot>]),
}

impl AnalysisStore<'_> {
    fn file(&self, name: &[u8]) -> Option<FileView<'_>> {
        match self {
            Self::File { file, program, resolved_names, artifacts, node_analysis_targets }
                if file.name.as_ref() == name =>
            {
                Some(FileView::Artifacts(file, program, resolved_names, artifacts, *node_analysis_targets))
            }
            Self::File { .. } => None,
            Self::Project(files) => {
                files.iter().find(|file| file.name.as_ref() == name).map(|file| FileView::Snapshot(file.as_ref()))
            }
        }
    }
}

enum FileView<'analysis> {
    Artifacts(
        &'analysis File,
        &'analysis Program<'analysis>,
        &'analysis ResolvedNames<'analysis>,
        &'analysis AnalysisArtifacts,
        Option<&'analysis [bool; u8::MAX as usize + 1]>,
    ),
    Snapshot(&'analysis FileAnalysisSnapshot),
}

enum TypeView<'analysis> {
    Union(&'analysis TUnion),
    Encoded(&'analysis [u8]),
}

impl TypeView<'_> {
    fn write_to(self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Union(ty) => write_type(writer, ty),
            Self::Encoded(bytes) => {
                writer.write_raw(bytes);
                Ok(())
            }
        }
    }
}

impl FileView<'_> {
    fn name(&self) -> &[u8] {
        match self {
            Self::Artifacts(file, ..) => &file.name,
            Self::Snapshot(file) => &file.name,
        }
    }

    fn size(&self) -> u32 {
        match self {
            Self::Artifacts(file, ..) => file.size,
            Self::Snapshot(file) => file.size,
        }
    }

    fn expression_count(&self) -> usize {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => artifacts.expression_types.len(),
            Self::Snapshot(file) => file.expression_types.len(),
        }
    }

    fn expression_type(&self, span: &(u32, u32)) -> Option<TypeView<'_>> {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => {
                artifacts.expression_types.get(span).map(AsRef::as_ref).map(TypeView::Union)
            }
            Self::Snapshot(file) => file
                .expression_types
                .get(span)
                .and_then(|range| file.encoded_types.get(range.clone()))
                .map(TypeView::Encoded),
        }
    }

    fn nearby_expression_spans(&self, requested: &[(u32, u32)]) -> Vec<(u32, u32)> {
        let mut spans = match self {
            Self::Artifacts(_, _, _, artifacts, _) => artifacts.expression_types.keys().copied().collect::<Vec<_>>(),
            Self::Snapshot(file) => file.expression_types.keys().copied().collect::<Vec<_>>(),
        };
        spans.sort_unstable();

        let requested = requested.iter().copied().collect::<HashSet<_>>();
        let mut selected = HashSet::default();
        let mut nearby = Vec::with_capacity(EXPRESSION_TYPE_PREFETCH.min(spans.len()));
        for span in &requested {
            let index = spans.binary_search(span).unwrap_or_else(|index| index);
            let start = index.saturating_sub(2);
            let end = (start + EXPRESSION_TYPE_PREFETCH).min(spans.len());
            for candidate in &spans[start..end] {
                if !requested.contains(candidate) && selected.insert(*candidate) {
                    nearby.push(*candidate);
                }
            }
        }

        nearby
    }

    fn inferred_return_count(&self) -> usize {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => artifacts.inferred_return_types.len(),
            Self::Snapshot(file) => file.inferred_return_types.len(),
        }
    }

    fn write_expression_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        let mut spans = match self {
            Self::Artifacts(_, _, _, artifacts, _) => artifacts.expression_types.keys().copied().collect::<Vec<_>>(),
            Self::Snapshot(file) => file.expression_types.keys().copied().collect::<Vec<_>>(),
        };

        spans.sort_unstable();
        writer.write_u32(u32::try_from(spans.len()).map_err(|_| protocol("too many expression types"))?);
        for span in spans {
            writer.write_u32(span.0);
            writer.write_u32(span.1);
            let ty = self
                .expression_type(&span)
                .ok_or_else(|| protocol("expression type disappeared while encoding analysis artifacts"))?;
            ty.write_to(writer).map_err(|error| {
                protocol(format!(
                    "failed to encode expression type at {}:{} in `{}`: {error}",
                    span.0,
                    span.1,
                    String::from_utf8_lossy(self.name())
                ))
            })?;
        }

        Ok(())
    }

    fn write_expression_type_snapshot(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => {
                let mut spans = artifacts.expression_types.keys().copied().collect::<Vec<_>>();
                spans.sort_unstable();
                let mut encoded = PayloadWriter::new();
                let mut handles = Vec::new();
                let mut records = Vec::with_capacity(spans.len());
                for span in spans {
                    let ty = artifacts
                        .expression_types
                        .get(&span)
                        .ok_or_else(|| protocol("expression type disappeared while snapshotting analysis artifacts"))?;
                    let range = encode_snapshot_type(&mut encoded, ty, &mut handles)?;
                    records.push((span, range));
                }
                write_expression_type_records(writer, &records, &encoded.finish())
            }
            Self::Snapshot(file) => {
                let mut records =
                    file.expression_types.iter().map(|(span, range)| (*span, range.clone())).collect::<Vec<_>>();
                records.sort_unstable_by_key(|(span, _)| *span);
                let length = records.iter().map(|(_, range)| range.end).max().unwrap_or(0);
                let encoded = file
                    .encoded_types
                    .get(..length)
                    .ok_or_else(|| protocol("expression type snapshot lies outside retained analysis artifacts"))?;
                write_expression_type_records(writer, &records, encoded)
            }
        }
    }

    fn write_inferred_return_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => {
                writer.write_u32(
                    u32::try_from(artifacts.inferred_return_types.len())
                        .map_err(|_| protocol("too many inferred types"))?,
                );
                for ty in &artifacts.inferred_return_types {
                    write_type(writer, ty)?;
                }
            }
            Self::Snapshot(file) => {
                write_encoded_types(writer, &file.encoded_types, &file.inferred_return_types)?;
            }
        }
        Ok(())
    }

    fn inferred_yield_key_count(&self) -> usize {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => artifacts.inferred_yield_key_types.len(),
            Self::Snapshot(file) => file.inferred_yield_key_types.len(),
        }
    }

    fn inferred_yield_value_count(&self) -> usize {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => artifacts.inferred_yield_value_types.len(),
            Self::Snapshot(file) => file.inferred_yield_value_types.len(),
        }
    }

    fn write_inferred_yield_key_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => write_types(writer, &artifacts.inferred_yield_key_types),
            Self::Snapshot(file) => write_encoded_types(writer, &file.encoded_types, &file.inferred_yield_key_types),
        }
    }

    fn write_inferred_yield_value_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => write_types(writer, &artifacts.inferred_yield_value_types),
            Self::Snapshot(file) => write_encoded_types(writer, &file.encoded_types, &file.inferred_yield_value_types),
        }
    }

    fn write_reference_summary(&self, writer: &mut PayloadWriter) {
        match self {
            Self::Artifacts(_, _, _, artifacts, _) => write_reference_summary(writer, &artifacts.symbol_references),
            Self::Snapshot(file) => file.references.write_to(writer),
        }
    }

    fn file_id(&self) -> mago_database::file::FileId {
        match self {
            Self::Artifacts(file, ..) => file.id,
            Self::Snapshot(file) => file.file_id,
        }
    }

    fn write_source_snapshot(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(file, program, resolved_names, _, targets) => {
                let snapshot =
                    SourceSnapshot::complete_with_targets(program, resolved_names, *targets).map_err(|error| {
                        protocol(format!(
                            "failed to snapshot syntax for `{}`: {error}",
                            String::from_utf8_lossy(&file.name)
                        ))
                    })?;

                snapshot.write_to(writer).map_err(|error| {
                    protocol(format!("failed to encode syntax for `{}`: {error}", String::from_utf8_lossy(&file.name)))
                })
            }
            Self::Snapshot(file) => {
                writer.write_raw(&file.encoded_source);
                Ok(())
            }
        }
    }
}

fn write_expression_type_records(
    writer: &mut PayloadWriter,
    records: &[((u32, u32), Range<usize>)],
    encoded: &[u8],
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(u32::try_from(records.len()).map_err(|_| protocol("too many expression types"))?);
    for (span, range) in records {
        writer.write_u32(span.0);
        writer.write_u32(span.1);
        writer.write_u32(u32::try_from(range.start).map_err(|_| protocol("expression type offset exceeds u32::MAX"))?);
        writer.write_u32(u32::try_from(range.len()).map_err(|_| protocol("expression type length exceeds u32::MAX"))?);
    }
    writer.write_bytes(encoded)?;
    Ok(())
}

pub(super) fn encode_before_analysis_request(
    generation: u64,
    plugins: &[u16],
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let writer = lifecycle_writer(BEFORE_ANALYSIS_REQUEST, generation, plugins)?;
    Ok(writer.finish())
}

pub(super) fn encode_after_file_analysis_request(
    generation: u64,
    plugins: &[u16],
    file: &File,
    program: &Program<'_>,
    resolved_names: &ResolvedNames<'_>,
    artifacts: &AnalysisArtifacts,
    include_expression_types: bool,
    node_analysis_targets: Option<&[bool; u8::MAX as usize + 1]>,
    embed_source: bool,
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut writer = lifecycle_writer(AFTER_FILE_ANALYSIS_REQUEST, generation, plugins)?;
    write_file_summary(
        &mut writer,
        FileView::Artifacts(file, program, resolved_names, artifacts, node_analysis_targets),
        include_expression_types,
        embed_source.then_some(file.contents.as_ref()),
    )?;
    Ok(writer.finish())
}

pub(super) fn encode_after_file_analysis_batch_request(
    generation: u64,
    plugins: &[u16],
    files: &[Arc<FileAnalysisSnapshot>],
    include_expression_types: bool,
    embed_source: bool,
    session: &ExternalAnalysisSession,
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut writer = lifecycle_writer(AFTER_FILE_ANALYSIS_BATCH_REQUEST, generation, plugins)?;
    writer.write_u32(u32::try_from(files.len()).map_err(|_| protocol("after-file batch exceeds u32::MAX files"))?);
    for file in files {
        let contents = if embed_source {
            Some(
                session
                    .source_file(file.file_id())
                    .filter(|source| source.name.as_ref() == file.name.as_ref() && source.size == file.size)
                    .ok_or_else(|| {
                        protocol(format!(
                            "analysis source `{}` is unavailable in generation {}",
                            String::from_utf8_lossy(&file.name),
                            session.generation()
                        ))
                    })?
                    .contents
                    .as_ref(),
            )
        } else {
            None
        };
        write_file_summary(&mut writer, FileView::Snapshot(file), include_expression_types, contents)?;
    }

    Ok(writer.finish())
}

pub(super) fn encode_after_analysis_request(
    generation: u64,
    plugins: &[u16],
    result: &AnalysisResult,
    files: &[Arc<FileAnalysisSnapshot>],
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut writer = lifecycle_writer(AFTER_ANALYSIS_REQUEST, generation, plugins)?;
    writer
        .write_u32(u32::try_from(result.issues.len()).map_err(|_| protocol("analysis has more than u32::MAX issues"))?);
    write_reference_summary(&mut writer, &result.symbol_references);
    writer.write_u32(u32::try_from(files.len()).map_err(|_| protocol("analysis has more than u32::MAX files"))?);
    for file in files {
        write_file_summary(&mut writer, FileView::Snapshot(file), false, None)?;
    }

    Ok(writer.finish())
}

fn lifecycle_writer(kind: u16, generation: u64, plugins: &[u16]) -> Result<PayloadWriter, ExternalAnalyzerError> {
    let mut writer = protocol::message_writer(kind);
    writer.write_u64(generation);
    writer
        .write_u16(u16::try_from(plugins.len()).map_err(|_| protocol("more than u16::MAX lifecycle plugins matched"))?);
    for plugin in plugins {
        writer.write_u16(*plugin);
    }

    Ok(writer)
}

fn write_file_summary(
    writer: &mut PayloadWriter,
    file: FileView<'_>,
    include_expression_types: bool,
    source_contents: Option<&[u8]>,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(file.name())?;
    writer.write_u32(file.size());
    writer.write_u32(u32::try_from(file.expression_count()).map_err(|_| protocol("too many expression types"))?);
    writer.write_u32(u32::try_from(file.inferred_return_count()).map_err(|_| protocol("too many return types"))?);
    writer.write_u32(u32::try_from(file.inferred_yield_key_count()).map_err(|_| protocol("too many yield key types"))?);

    writer.write_u32(
        u32::try_from(file.inferred_yield_value_count()).map_err(|_| protocol("too many yield value types"))?,
    );

    file.write_reference_summary(writer);
    writer.write_bool(include_expression_types);
    if include_expression_types {
        file.write_expression_type_snapshot(writer)?;
    }
    writer.write_bool(source_contents.is_some());
    if let Some(contents) = source_contents {
        writer.write_bytes(contents)?;
        file.write_source_snapshot(writer)?;
    }
    Ok(())
}

fn write_reference_summary(writer: &mut PayloadWriter, references: &SymbolReferences) {
    writer.write_u64(references.count_body_references() as u64);
    writer.write_u64(references.count_signature_references() as u64);
    writer.write_u64(references.total_map_entries() as u64);
}

pub(super) fn decode_lifecycle_response(
    payload: &[u8],
    request_kind: u16,
    active_plugins: &[u16],
    plugins: &[ExternalPlugin],
    session: &ExternalAnalysisSession,
    default_file: Option<&File>,
    codebase: &CodebaseMetadata,
) -> Result<LifecycleEffects, ExternalAnalyzerError> {
    let response_kind = match request_kind {
        BEFORE_ANALYSIS_REQUEST => BEFORE_ANALYSIS_RESPONSE,
        AFTER_FILE_ANALYSIS_REQUEST => AFTER_FILE_ANALYSIS_RESPONSE,
        AFTER_ANALYSIS_REQUEST => AFTER_ANALYSIS_RESPONSE,
        AFTER_FILE_ANALYSIS_BATCH_REQUEST => AFTER_FILE_ANALYSIS_BATCH_RESPONSE,
        _ => return Err(protocol(format!("unknown lifecycle request kind {request_kind}"))),
    };

    let mut reader = protocol::message_reader(payload, response_kind)?;
    let count = reader.read_count("lifecycle issues", MAXIMUM_ISSUES)?;
    let mut issues = IssueCollection::new();
    issues.reserve(count);
    for _ in 0..count {
        let plugin_index = reader.read_u16("lifecycle issue plugin index")?;
        if !active_plugins.contains(&plugin_index) {
            return Err(protocol(format!("worker reported an issue for inactive plugin index {plugin_index}")));
        }

        let plugin = plugins
            .get(plugin_index as usize)
            .ok_or_else(|| protocol(format!("worker reported unknown plugin index {plugin_index}")))?;
        let level = read_level(&mut reader)?;
        let local_code = reader.read_string("lifecycle issue code")?;
        if local_code.is_empty() {
            return Err(protocol(format!("plugin `{}` reported an empty issue code", plugin.identifier)));
        }

        let message = reader.read_string("lifecycle issue message")?;
        if message.is_empty() {
            return Err(protocol(format!("plugin `{}` reported an empty issue message", plugin.identifier)));
        }

        let note_count = reader.read_count("lifecycle issue notes", MAXIMUM_NOTES)?;
        let mut notes = Vec::with_capacity(note_count);
        for _ in 0..note_count {
            notes.push(reader.read_string("lifecycle issue note")?);
        }

        let help = reader.read_optional_string("lifecycle issue help")?;
        let link = reader.read_optional_string("lifecycle issue link")?;
        let annotation_count = reader.read_count("lifecycle issue annotations", MAXIMUM_ANNOTATIONS)?;
        let mut annotations = Vec::with_capacity(annotation_count);
        let mut has_primary = false;
        for _ in 0..annotation_count {
            let kind = match reader.read_u8("lifecycle annotation kind")? {
                1 => AnnotationKind::Primary,
                2 => AnnotationKind::Secondary,
                value => return Err(protocol(format!("invalid lifecycle annotation kind {value}"))),
            };

            let named_file = if reader.read_bool("lifecycle annotation file presence")? {
                Some(reader.read_bytes("lifecycle annotation file")?)
            } else {
                None
            };

            let (file_id, size) = match named_file {
                Some(name) => session.source(name).ok_or_else(|| {
                    protocol(format!("lifecycle annotation names unknown file `{}`", String::from_utf8_lossy(name)))
                })?,
                None => default_file
                    .map(|file| (file.id, file.size))
                    .ok_or_else(|| protocol("a project lifecycle annotation must name its source file"))?,
            };

            let start = reader.read_u32("lifecycle annotation start")?;
            let end = reader.read_u32("lifecycle annotation end")?;
            if start > end || end > size {
                return Err(protocol(format!(
                    "plugin `{}` reported invalid annotation span {start}..{end}",
                    plugin.identifier
                )));
            }

            has_primary |= kind == AnnotationKind::Primary;
            let mut annotation = Annotation::new(kind, Span::new(file_id, Position::new(start), Position::new(end)));
            if let Some(message) = reader.read_optional_string("lifecycle annotation message")? {
                annotation = annotation.with_message(message);
            }

            annotations.push(annotation);
        }

        // FIXME(azjezz): Should we forbid missing primary annotations? Rust-side can create issues without primary annotations,
        // but it messes up the baseline stuff.
        if !has_primary {
            return Err(protocol(format!(
                "plugin `{}` reported an issue without a primary annotation",
                plugin.identifier
            )));
        }

        let edit_count = reader.read_count("lifecycle issue edits", MAXIMUM_EDITS)?;
        let mut edits = HashMap::default();
        for _ in 0..edit_count {
            let named_file = if reader.read_bool("lifecycle edit file presence")? {
                Some(reader.read_bytes("lifecycle edit file")?)
            } else {
                None
            };

            let (file_id, size) = match named_file {
                Some(name) => session.source(name).ok_or_else(|| {
                    protocol(format!("lifecycle edit names unknown file `{}`", String::from_utf8_lossy(name)))
                })?,
                None => default_file
                    .map(|file| (file.id, file.size))
                    .ok_or_else(|| protocol("a project lifecycle edit must name its source file"))?,
            };

            let start = reader.read_u32("lifecycle edit start")?;
            let end = reader.read_u32("lifecycle edit end")?;
            if start > end || end > size {
                return Err(protocol(format!(
                    "plugin `{}` reported invalid edit range {start}..{end}",
                    plugin.identifier
                )));
            }

            let safety = read_safety(&mut reader)?;
            let new_text = reader.read_bytes("lifecycle edit replacement")?.to_vec();
            edits
                .entry(file_id)
                .or_insert_with(Vec::new)
                .push(TextEdit::replace(TextRange::new(start, end), new_text).with_safety(safety));
        }

        let mut issue = Issue::new(level, message)
            .with_code(format!("{}/{}", plugin.identifier, local_code))
            .with_annotations(annotations);
        issue.notes = notes;
        issue.help = help;
        issue.link = link;
        issue.edits = edits;
        issues.push(issue);
    }

    let reference_count = reader.read_count("lifecycle contributed references", MAXIMUM_REFERENCES)?;
    if request_kind == AFTER_ANALYSIS_REQUEST && reference_count != 0 {
        return Err(protocol("after-analysis hooks may not contribute symbol references"));
    }

    let mut references = SymbolReferences::new();
    let mut references_by_file = HashMap::<FileId, SymbolReferences>::default();
    for _ in 0..reference_count {
        let plugin_index = reader.read_u16("reference contribution plugin index")?;
        if !active_plugins.contains(&plugin_index) {
            return Err(protocol(format!("worker contributed a reference for inactive plugin index {plugin_index}")));
        }

        let plugin = plugins.get(plugin_index as usize).ok_or_else(|| {
            protocol(format!("worker contributed a reference for unknown plugin index {plugin_index}"))
        })?;

        let discovery_file = if request_kind == BEFORE_ANALYSIS_REQUEST {
            None
        } else {
            let name = reader.read_bytes("reference contribution discovery file")?;
            Some(
                session
                    .source(name)
                    .ok_or_else(|| {
                        protocol(format!(
                            "reference contribution names unknown discovery file `{}`",
                            String::from_utf8_lossy(name)
                        ))
                    })?
                    .0,
            )
        };

        let source = read_reference_origin(&mut reader, codebase)?;
        let target = read_symbol_identifier(&mut reader, codebase, true)?;
        let kind = read_reference_kind(&mut reader)?;
        validate_reference_target(codebase, target, kind).map_err(|reason| {
            protocol(format!("plugin `{}` contributed an invalid reference: {reason}", plugin.identifier))
        })?;

        let references =
            discovery_file.map_or(&mut references, |file_id| references_by_file.entry(file_id).or_default());
        match kind {
            SymbolReferenceKind::Body => references.add_reference(source, target, false),
            SymbolReferenceKind::Signature => references.add_reference(source, target, true),
            SymbolReferenceKind::OverriddenMember => {
                let ReferenceOrigin::Symbol(source) = source else {
                    return Err(protocol("an overridden-member reference must originate from a symbol"));
                };
                references.add_overridden_member_reference(source, target);
            }
            SymbolReferenceKind::FunctionLikeReturn => {
                let ReferenceOrigin::Symbol(source) = source else {
                    return Err(protocol("a function-like-return reference must originate from a symbol"));
                };
                references.add_functionlike_return_reference(source, target);
            }
            SymbolReferenceKind::PropertyRead => {
                let ReferenceOrigin::Symbol(source) = source else {
                    return Err(protocol("a property-read reference must originate from a symbol"));
                };
                references.add_property_read_reference(source, target);
            }
            SymbolReferenceKind::PropertyWrite => {
                let ReferenceOrigin::Symbol(source) = source else {
                    return Err(protocol("a property-write reference must originate from a symbol"));
                };
                references.add_property_write_reference(source, target);
            }
        }
    }

    reader.finish()?;
    Ok(LifecycleEffects { issues, references, references_by_file })
}

fn read_safety(reader: &mut PayloadReader<'_>) -> Result<Safety, ExternalAnalyzerError> {
    match reader.read_u8("lifecycle edit safety")? {
        1 => Ok(Safety::Safe),
        2 => Ok(Safety::PotentiallyUnsafe),
        3 => Ok(Safety::Unsafe),
        value => Err(protocol(format!("invalid lifecycle text edit safety {value}"))),
    }
}

pub(super) fn handle_analysis_query(
    payload: &[u8],
    session: &ExternalAnalysisSession,
    store: &AnalysisStore<'_>,
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut reader = protocol::message_reader(payload, ANALYSIS_QUERY_REQUEST)?;
    let generation = reader.read_u64("analysis query generation")?;
    if generation != session.generation() {
        return Err(protocol(format!(
            "analysis query generation {generation} does not match {}",
            session.generation()
        )));
    }

    let operation = reader.read_u8("analysis query operation")?;
    let file_name = reader.read_bytes("analysis query file")?;
    let file = store.file(file_name).ok_or_else(|| {
        protocol(format!("analysis query names unavailable file `{}`", String::from_utf8_lossy(file_name)))
    })?;

    let mut writer = protocol::message_writer(ANALYSIS_QUERY_RESPONSE);
    writer.write_u64(generation);
    writer.write_u8(operation);
    writer.write_bytes(file_name)?;
    match operation {
        GET_EXPRESSION_TYPES => {
            let count = reader.read_count("expression type queries", MAXIMUM_TYPE_QUERIES)?;
            let mut spans = Vec::with_capacity(count);
            for _ in 0..count {
                spans.push((reader.read_u32("expression start")?, reader.read_u32("expression end")?));
            }
            writer.write_u32(count as u32);
            for span in &spans {
                let ty = file.expression_type(span);
                writer.write_bool(ty.is_some());
                if let Some(ty) = ty {
                    ty.write_to(&mut writer)?;
                }
            }

            let nearby = file.nearby_expression_spans(&spans);
            writer.write_u32(nearby.len() as u32);
            for span in nearby {
                writer.write_u32(span.0);
                writer.write_u32(span.1);
                file.expression_type(&span)
                    .ok_or_else(|| {
                        protocol("prefetched expression type disappeared while encoding analysis artifacts")
                    })?
                    .write_to(&mut writer)?;
            }
        }
        GET_ALL_EXPRESSION_TYPES => file.write_expression_types(&mut writer)?,
        GET_INFERRED_RETURN_TYPES => file.write_inferred_return_types(&mut writer)?,
        GET_INFERRED_YIELD_KEY_TYPES => file.write_inferred_yield_key_types(&mut writer)?,
        GET_INFERRED_YIELD_VALUE_TYPES => file.write_inferred_yield_value_types(&mut writer)?,
        GET_SOURCE_FILE => {
            let source = session
                .source_file(file.file_id())
                .filter(|source| source.name.as_ref() == file.name() && source.size == file.size())
                .ok_or_else(|| {
                    protocol(format!(
                        "analysis source `{}` is unavailable in generation {generation}",
                        String::from_utf8_lossy(file.name())
                    ))
                })?;

            writer.write_bytes(&source.contents)?;
            file.write_source_snapshot(&mut writer)?;
        }
        value => return Err(protocol(format!("unknown analysis query operation {value}"))),
    }

    reader.finish()?;
    Ok(writer.finish())
}

pub(super) fn is_symbol_reference_query(payload: &[u8]) -> Result<bool, ExternalAnalyzerError> {
    Ok(protocol::message_kind(payload)? == SYMBOL_REFERENCE_QUERY_REQUEST)
}

pub(super) fn handle_symbol_reference_query(
    payload: &[u8],
    session: &ExternalAnalysisSession,
    codebase: &CodebaseMetadata,
    store: &SymbolReferenceStore<'_>,
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut reader = protocol::message_reader(payload, SYMBOL_REFERENCE_QUERY_REQUEST)?;
    let generation = reader.read_u64("symbol-reference query generation")?;
    if generation != session.generation() {
        return Err(protocol(format!(
            "symbol-reference query generation {generation} does not match {}",
            session.generation()
        )));
    }

    let operation = reader.read_u8("symbol-reference query operation")?;
    let query_count = reader.read_count("symbol-reference queries", MAXIMUM_REFERENCE_QUERIES)?;
    let mut writer = protocol::message_writer(SYMBOL_REFERENCE_QUERY_RESPONSE);
    writer.write_u64(generation);
    writer.write_u8(operation);
    writer.write_u32(query_count as u32);

    match operation {
        GET_REFERENCES_TO => {
            let mut queries = Vec::with_capacity(query_count);
            for _ in 0..query_count {
                let target = read_symbol_identifier(&mut reader, codebase, false)?;
                queries.push(target);
            }

            for query in queries {
                write_recorded_references(&mut writer, store.references_to(query))?;
            }
        }
        GET_REFERENCES_FROM => {
            let mut queries = Vec::with_capacity(query_count);
            for _ in 0..query_count {
                let source = read_reference_origin(&mut reader, codebase)?;
                queries.push(source);
            }

            for query in queries {
                write_recorded_references(&mut writer, store.references_from(query))?;
            }
        }
        unknown => return Err(protocol(format!("unknown symbol-reference query operation {unknown}"))),
    }

    reader.finish()?;
    Ok(writer.finish())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RecordedReference {
    source: ReferenceOrigin,
    target: SymbolIdentifier,
    kind: SymbolReferenceKind,
}

fn write_recorded_references(
    writer: &mut PayloadWriter,
    references: &[RecordedReference],
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(u32::try_from(references.len()).map_err(|_| protocol("too many symbol references"))?);
    for reference in references {
        write_reference_origin(writer, reference.source)?;
        write_symbol_identifier(writer, reference.target)?;
        writer.write_u8(match reference.kind {
            SymbolReferenceKind::Body => 1,
            SymbolReferenceKind::Signature => 2,
            SymbolReferenceKind::OverriddenMember => 3,
            SymbolReferenceKind::FunctionLikeReturn => 4,
            SymbolReferenceKind::PropertyRead => 5,
            SymbolReferenceKind::PropertyWrite => 6,
        });
    }

    Ok(())
}

fn read_reference_origin(
    reader: &mut PayloadReader<'_>,
    codebase: &CodebaseMetadata,
) -> Result<ReferenceOrigin, ExternalAnalyzerError> {
    let endpoint = reader.read_u8("reference origin kind")?;
    if endpoint == 3 {
        let file = reader.read_bytes("reference origin file")?;
        if file.is_empty() {
            return Err(protocol("reference origin file cannot be empty"));
        }

        return Ok(ReferenceOrigin::File(word(file)));
    }

    read_symbol_identifier_with_kind(reader, codebase, false, endpoint).map(ReferenceOrigin::Symbol)
}

fn write_reference_origin(writer: &mut PayloadWriter, source: ReferenceOrigin) -> Result<(), ExternalAnalyzerError> {
    match source {
        ReferenceOrigin::Symbol(symbol) => write_symbol_identifier(writer, symbol),
        ReferenceOrigin::File(file) => {
            writer.write_u8(3);
            writer.write_bytes(file.as_bytes())?;
            Ok(())
        }
    }
}

fn read_symbol_identifier(
    reader: &mut PayloadReader<'_>,
    codebase: &CodebaseMetadata,
    target: bool,
) -> Result<SymbolIdentifier, ExternalAnalyzerError> {
    let kind = reader.read_u8("symbol reference endpoint kind")?;
    read_symbol_identifier_with_kind(reader, codebase, target, kind)
}

fn read_symbol_identifier_with_kind(
    reader: &mut PayloadReader<'_>,
    codebase: &CodebaseMetadata,
    target: bool,
    kind: u8,
) -> Result<SymbolIdentifier, ExternalAnalyzerError> {
    let symbol = reader.read_bytes("symbol reference name")?;
    if symbol.is_empty() {
        return Err(protocol("symbol reference name cannot be empty"));
    }
    let symbol = normalize_symbol(codebase, symbol, target);

    match kind {
        1 => Ok((symbol, empty_word())),
        2 => {
            let member = reader.read_bytes("symbol reference member")?;
            if member.is_empty() {
                return Err(protocol("symbol reference member cannot be empty"));
            }
            let raw_member = word(member);
            let lowercase_member = ascii_lowercase_word(member);
            let member = if codebase.function_likes.contains_key(&(symbol, lowercase_member)) {
                lowercase_member
            } else {
                raw_member
            };
            Ok((symbol, member))
        }
        unknown => Err(protocol(format!("unknown symbol reference endpoint kind {unknown}"))),
    }
}

fn normalize_symbol(codebase: &CodebaseMetadata, bytes: &[u8], target: bool) -> mago_word::Word {
    let lowercase = ascii_lowercase_word(bytes);
    if codebase.class_likes.contains_key(&lowercase) || codebase.function_likes.contains_key(&(empty_word(), lowercase))
    {
        return lowercase;
    }

    let constant = ascii_lowercase_constant_name_word(bytes);
    if codebase.constants.contains_key(&constant) {
        return constant;
    }

    if target { word(bytes) } else { lowercase }
}

fn write_symbol_identifier(writer: &mut PayloadWriter, symbol: SymbolIdentifier) -> Result<(), ExternalAnalyzerError> {
    if symbol.1.is_empty() {
        writer.write_u8(1);
        writer.write_bytes(symbol.0.as_bytes())?;
    } else {
        writer.write_u8(2);
        writer.write_bytes(symbol.0.as_bytes())?;
        writer.write_bytes(symbol.1.as_bytes())?;
    }

    Ok(())
}

fn read_reference_kind(reader: &mut PayloadReader<'_>) -> Result<SymbolReferenceKind, ExternalAnalyzerError> {
    match reader.read_u8("symbol reference kind")? {
        1 => Ok(SymbolReferenceKind::Body),
        2 => Ok(SymbolReferenceKind::Signature),
        3 => Ok(SymbolReferenceKind::OverriddenMember),
        4 => Ok(SymbolReferenceKind::FunctionLikeReturn),
        5 => Ok(SymbolReferenceKind::PropertyRead),
        6 => Ok(SymbolReferenceKind::PropertyWrite),
        unknown => Err(protocol(format!("unknown symbol reference kind {unknown}"))),
    }
}

fn validate_reference_target(
    codebase: &CodebaseMetadata,
    target: SymbolIdentifier,
    kind: SymbolReferenceKind,
) -> Result<(), String> {
    if target.1.is_empty() {
        let exists = if kind == SymbolReferenceKind::FunctionLikeReturn {
            codebase.function_likes.contains_key(&(empty_word(), target.0))
        } else {
            codebase.class_likes.contains_key(&target.0)
                || codebase.function_likes.contains_key(&(empty_word(), target.0))
                || codebase.constants.contains_key(&target.0)
        };
        return exists.then_some(()).ok_or_else(|| format!("target `{}` does not exist", target.0));
    }

    let Some(class_like) = codebase.class_likes.get(&target.0) else {
        return Err(format!("target class-like `{}` does not exist", target.0));
    };
    let exists = match kind {
        SymbolReferenceKind::PropertyRead | SymbolReferenceKind::PropertyWrite => {
            class_like.properties.contains_key(&target.1)
        }
        SymbolReferenceKind::FunctionLikeReturn => codebase.function_likes.contains_key(&target),
        SymbolReferenceKind::OverriddenMember => class_like.methods.contains(&target.1),
        SymbolReferenceKind::Body | SymbolReferenceKind::Signature => {
            class_like.methods.contains(&target.1)
                || class_like.properties.contains_key(&target.1)
                || class_like.constants.contains_key(&target.1)
                || class_like.enum_cases.contains_key(&target.1)
        }
    };

    exists.then_some(()).ok_or_else(|| format!("target member `{}::{}` does not exist", target.0, target.1))
}

fn write_types(writer: &mut PayloadWriter, types: &[TUnion]) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(u32::try_from(types.len()).map_err(|_| protocol("too many inferred types"))?);
    for ty in types {
        write_type(writer, ty)?;
    }

    Ok(())
}

fn write_encoded_types(
    writer: &mut PayloadWriter,
    payload: &[u8],
    ranges: &[Range<usize>],
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(u32::try_from(ranges.len()).map_err(|_| protocol("too many inferred types"))?);
    for range in ranges {
        let bytes = payload
            .get(range.clone())
            .ok_or_else(|| protocol("retained analysis type points outside its encoded payload"))?;
        writer.write_raw(bytes);
    }

    Ok(())
}

fn write_type(writer: &mut PayloadWriter, ty: &TUnion) -> Result<(), ExternalAnalyzerError> {
    protocol::encode_union_snapshot(writer, ty, &mut Vec::new(), 0)
}

fn read_level(reader: &mut PayloadReader<'_>) -> Result<Level, ExternalAnalyzerError> {
    match reader.read_u8("lifecycle issue level")? {
        1 => Ok(Level::Note),
        2 => Ok(Level::Help),
        3 => Ok(Level::Warning),
        4 => Ok(Level::Error),
        value => Err(protocol(format!("invalid lifecycle issue level {value}"))),
    }
}
