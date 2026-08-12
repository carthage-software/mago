use std::ops::Range;
use std::sync::Arc;

use foldhash::HashMap;
use mago_codex::reference::SymbolReferences;
use mago_codex::ttype::union::TUnion;
use mago_database::file::File;
use mago_extension::PayloadReader;
use mago_extension::PayloadWriter;
use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_span::Position;
use mago_span::Span;

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
const BEFORE_ANALYSIS_RESPONSE: u16 = 0x8005;
const AFTER_FILE_ANALYSIS_RESPONSE: u16 = 0x8006;
const AFTER_ANALYSIS_RESPONSE: u16 = 0x8007;
const ANALYSIS_QUERY_RESPONSE: u16 = 0x8008;
const AFTER_FILE_ANALYSIS_BATCH_RESPONSE: u16 = 0x8009;

const GET_EXPRESSION_TYPES: u8 = 1;
const GET_ALL_EXPRESSION_TYPES: u8 = 2;
const GET_INFERRED_RETURN_TYPES: u8 = 3;
const GET_INFERRED_YIELD_KEY_TYPES: u8 = 4;
const GET_INFERRED_YIELD_VALUE_TYPES: u8 = 5;
const MAXIMUM_ISSUES: usize = 1_000_000;
const MAXIMUM_ANNOTATIONS: usize = 0x0001_0000;
const MAXIMUM_NOTES: usize = 0x0001_0000;
const MAXIMUM_TYPE_QUERIES: usize = 1_000_000;

#[derive(Debug)]
pub struct FileAnalysisSnapshot {
    file_id: mago_database::file::FileId,
    name: Arc<[u8]>,
    size: u32,
    encoded_types: Box<[u8]>,
    expression_types: HashMap<(u32, u32), Range<usize>>,
    inferred_return_types: Vec<Range<usize>>,
    inferred_yield_key_types: Vec<Range<usize>>,
    inferred_yield_value_types: Vec<Range<usize>>,
    references: ReferenceSummary,
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
    /// Returns an error when an inferred type cannot be represented by the extension protocol.
    pub fn new(file: &File, artifacts: &AnalysisArtifacts) -> Result<Self, ExternalAnalyzerError> {
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
            encoded_types: writer.finish().into_boxed_slice(),
            expression_types,
            inferred_return_types,
            inferred_yield_key_types,
            inferred_yield_value_types,
            references: ReferenceSummary::from(&artifacts.symbol_references),
        })
    }

    #[must_use]
    pub const fn file_id(&self) -> mago_database::file::FileId {
        self.file_id
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
    File { file: &'analysis File, artifacts: &'analysis AnalysisArtifacts },
    Project(&'analysis [Arc<FileAnalysisSnapshot>]),
}

impl AnalysisStore<'_> {
    fn file(&self, name: &[u8]) -> Option<FileView<'_>> {
        match self {
            Self::File { file, artifacts } if file.name.as_ref() == name => Some(FileView::Artifacts(file, artifacts)),
            Self::File { .. } => None,
            Self::Project(files) => {
                files.iter().find(|file| file.name.as_ref() == name).map(|file| FileView::Snapshot(file.as_ref()))
            }
        }
    }
}

enum FileView<'analysis> {
    Artifacts(&'analysis File, &'analysis AnalysisArtifacts),
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
            Self::Artifacts(file, _) => &file.name,
            Self::Snapshot(file) => &file.name,
        }
    }

    fn size(&self) -> u32 {
        match self {
            Self::Artifacts(file, _) => file.size,
            Self::Snapshot(file) => file.size,
        }
    }

    fn expression_count(&self) -> usize {
        match self {
            Self::Artifacts(_, artifacts) => artifacts.expression_types.len(),
            Self::Snapshot(file) => file.expression_types.len(),
        }
    }

    fn expression_type(&self, span: &(u32, u32)) -> Option<TypeView<'_>> {
        match self {
            Self::Artifacts(_, artifacts) => {
                artifacts.expression_types.get(span).map(AsRef::as_ref).map(TypeView::Union)
            }
            Self::Snapshot(file) => file
                .expression_types
                .get(span)
                .and_then(|range| file.encoded_types.get(range.clone()))
                .map(TypeView::Encoded),
        }
    }

    fn inferred_return_count(&self) -> usize {
        match self {
            Self::Artifacts(_, artifacts) => artifacts.inferred_return_types.len(),
            Self::Snapshot(file) => file.inferred_return_types.len(),
        }
    }

    fn write_expression_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        let mut spans = match self {
            Self::Artifacts(_, artifacts) => artifacts.expression_types.keys().copied().collect::<Vec<_>>(),
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

    fn write_inferred_return_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(_, artifacts) => {
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
            Self::Artifacts(_, artifacts) => artifacts.inferred_yield_key_types.len(),
            Self::Snapshot(file) => file.inferred_yield_key_types.len(),
        }
    }

    fn inferred_yield_value_count(&self) -> usize {
        match self {
            Self::Artifacts(_, artifacts) => artifacts.inferred_yield_value_types.len(),
            Self::Snapshot(file) => file.inferred_yield_value_types.len(),
        }
    }

    fn write_inferred_yield_key_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(_, artifacts) => write_types(writer, &artifacts.inferred_yield_key_types),
            Self::Snapshot(file) => write_encoded_types(writer, &file.encoded_types, &file.inferred_yield_key_types),
        }
    }

    fn write_inferred_yield_value_types(&self, writer: &mut PayloadWriter) -> Result<(), ExternalAnalyzerError> {
        match self {
            Self::Artifacts(_, artifacts) => write_types(writer, &artifacts.inferred_yield_value_types),
            Self::Snapshot(file) => write_encoded_types(writer, &file.encoded_types, &file.inferred_yield_value_types),
        }
    }

    fn write_reference_summary(&self, writer: &mut PayloadWriter) {
        match self {
            Self::Artifacts(_, artifacts) => write_reference_summary(writer, &artifacts.symbol_references),
            Self::Snapshot(file) => file.references.write_to(writer),
        }
    }
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
    artifacts: &AnalysisArtifacts,
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut writer = lifecycle_writer(AFTER_FILE_ANALYSIS_REQUEST, generation, plugins)?;
    write_file_summary(&mut writer, FileView::Artifacts(file, artifacts))?;
    Ok(writer.finish())
}

pub(super) fn encode_after_file_analysis_batch_request(
    generation: u64,
    plugins: &[u16],
    files: &[Arc<FileAnalysisSnapshot>],
) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut writer = lifecycle_writer(AFTER_FILE_ANALYSIS_BATCH_REQUEST, generation, plugins)?;
    writer.write_u32(u32::try_from(files.len()).map_err(|_| protocol("after-file batch exceeds u32::MAX files"))?);
    for file in files {
        write_file_summary(&mut writer, FileView::Snapshot(file))?;
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
        write_file_summary(&mut writer, FileView::Snapshot(file))?;
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

fn write_file_summary(writer: &mut PayloadWriter, file: FileView<'_>) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(file.name())?;
    writer.write_u32(file.size());
    writer.write_u32(u32::try_from(file.expression_count()).map_err(|_| protocol("too many expression types"))?);
    writer.write_u32(u32::try_from(file.inferred_return_count()).map_err(|_| protocol("too many return types"))?);
    writer.write_u32(u32::try_from(file.inferred_yield_key_count()).map_err(|_| protocol("too many yield key types"))?);

    writer.write_u32(
        u32::try_from(file.inferred_yield_value_count()).map_err(|_| protocol("too many yield value types"))?,
    );

    file.write_reference_summary(writer);
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
) -> Result<IssueCollection, ExternalAnalyzerError> {
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

        let mut issue = Issue::new(level, message)
            .with_code(format!("{}/{}", plugin.identifier, local_code))
            .with_annotations(annotations);
        issue.notes = notes;
        issue.help = help;
        issue.link = link;
        issues.push(issue);
    }

    reader.finish()?;
    Ok(issues)
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
            writer.write_u32(count as u32);
            for _ in 0..count {
                let span = (reader.read_u32("expression start")?, reader.read_u32("expression end")?);
                let ty = file.expression_type(&span);
                writer.write_bool(ty.is_some());
                if let Some(ty) = ty {
                    ty.write_to(&mut writer)?;
                }
            }
        }
        GET_ALL_EXPRESSION_TYPES => file.write_expression_types(&mut writer)?,
        GET_INFERRED_RETURN_TYPES => file.write_inferred_return_types(&mut writer)?,
        GET_INFERRED_YIELD_KEY_TYPES => file.write_inferred_yield_key_types(&mut writer)?,
        GET_INFERRED_YIELD_VALUE_TYPES => file.write_inferred_yield_value_types(&mut writer)?,
        value => return Err(protocol(format!("unknown analysis query operation {value}"))),
    }

    reader.finish()?;
    Ok(writer.finish())
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
