//! Stable binary messages carried inside `mago-extension` frames.
//!
//! All integers are unsigned and big-endian. Strings and byte strings are a
//! `u32` byte length followed by that many bytes. The fixed message header is:
//! `MLNT`, protocol major, protocol minor, message kind, reserved zero.

#![allow(clippy::big_endian_bytes, reason = "network byte order is part of the stable linter wire format")]

use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

use mago_database::file::File;
use mago_extension::PayloadReader;
use mago_extension::PayloadWriter;
#[cfg(test)]
use mago_extension::source::NO_NODE;
use mago_extension::source::SourceSnapshot;
use mago_extension::source::write_node_kind_table;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Program;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;
use mago_text_edit::TextRange;
#[cfg(test)]
use strum::IntoEnumIterator;

use super::ExternalLintError;
use super::ExternalRule;

pub const LINTER_PROTOCOL_MAGIC: [u8; 4] = *b"MLNT";
pub const LINTER_PROTOCOL_MAJOR: u16 = 1;
pub const LINTER_PROTOCOL_MINOR: u16 = 0;

const HEADER_LENGTH: usize = 12;
const DESCRIBE_REQUEST: u16 = 1;
const LINT_FILE_REQUEST: u16 = 2;
const DESCRIBE_RESPONSE: u16 = 0x8001;
const LINT_FILE_RESPONSE: u16 = 0x8002;
const MAXIMUM_EXTENSIONS: usize = 0x4000;
const MAXIMUM_EXTENSIONS_RULES: usize = 0x4000;
const MAXIMUM_TARGETS_PER_RULE: usize = 512;
const MAXIMUM_ISSUES_PER_FILE: usize = 1_000_000;
const MAXIMUM_ANNOTATIONS_PER_ISSUE: usize = 0x0001_0000;
const MAXIMUM_EDITS_PER_ISSUE: usize = 0x0001_0000;
const MAXIMUM_NOTES_PER_ISSUE: usize = 0x0001_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Registration {
    pub extensions: Vec<super::ExternalExtension>,
    pub rules: Vec<ExternalRule>,
    pub has_worker_reducer: bool,
}

#[derive(Debug)]
pub(super) struct LintRequest {
    pub payload: Vec<u8>,
    pub targets: usize,
    pub nodes: usize,
    pub names: usize,
    pub trivia: usize,
    pub snapshot_duration: Duration,
    pub serialization_duration: Duration,
}

pub(super) fn encode_describe_request(php_version: PHPVersion) -> Vec<u8> {
    let mut writer = message_writer(DESCRIBE_REQUEST);
    writer.write_u32(php_version.to_version_id());
    write_node_kind_table(&mut writer);

    writer.finish()
}

pub(super) fn decode_registration(payload: &[u8]) -> Result<Registration, ExternalLintError> {
    let mut reader = message_reader(payload, DESCRIBE_RESPONSE)?;
    let extension_count = reader.read_count("extensions", MAXIMUM_EXTENSIONS)?;
    if extension_count == 0 {
        return Err(protocol("worker registration contains no extensions"));
    }

    let mut extensions = Vec::with_capacity(extension_count);
    let mut rules = Vec::new();
    let mut identifiers = HashSet::with_capacity(extension_count);
    let mut codes = HashSet::new();
    let mut has_worker_reducer = false;
    for _ in 0..extension_count {
        let identifier = reader.read_string("extension identifier")?;
        let name = reader.read_string("extension name")?;
        let version = reader.read_string("extension version")?;
        has_worker_reducer |= reader.read_bool("worker reducer flag")?;
        if identifier.is_empty() {
            return Err(protocol("extension identifier cannot be empty"));
        }

        if name.is_empty() {
            return Err(protocol("extension name cannot be empty"));
        }

        if version.is_empty() {
            return Err(protocol(format!("extension `{identifier}` has an empty version")));
        }

        if !identifiers.insert(identifier.clone()) {
            return Err(protocol(format!("worker advertises extension `{identifier}` more than once")));
        }

        let rule_count = reader.read_count("rules", MAXIMUM_EXTENSIONS_RULES)?;
        let mut extension_rules = Vec::with_capacity(rule_count);
        for _ in 0..rule_count {
            let code = reader.read_string("rule code")?;
            let rule_name = reader.read_string("rule name")?;
            let description = reader.read_string("rule description")?;
            let default_level = read_level(&mut reader)?;
            let default_enabled = reader.read_bool("rule default-enabled flag")?;
            let target_count = reader.read_count("rule targets", MAXIMUM_TARGETS_PER_RULE)?;
            let mut targets = Vec::with_capacity(target_count);
            let mut unique_targets = HashSet::with_capacity(target_count);
            for _ in 0..target_count {
                let target_name = reader.read_str("node kind")?;
                let target = target_name
                    .parse::<NodeKind>()
                    .map_err(|_| protocol(format!("rule `{code}` targets unknown node kind `{target_name}`")))?;
                if !unique_targets.insert(target) {
                    return Err(protocol(format!("rule `{code}` lists node kind `{target_name}` more than once")));
                }

                targets.push(target);
            }

            if code.is_empty() {
                return Err(protocol("rule code cannot be empty"));
            }

            if rule_name.is_empty() {
                return Err(protocol(format!("rule `{code}` has an empty name")));
            }

            if description.is_empty() {
                return Err(protocol(format!("rule `{code}` has an empty description")));
            }

            if targets.is_empty() {
                return Err(protocol(format!("rule `{code}` has no target node kinds")));
            }

            if !codes.insert(code.clone()) {
                return Err(protocol(format!("worker advertises linter rule `{code}` more than once")));
            }

            extension_rules.push(ExternalRule {
                code,
                name: rule_name,
                description,
                default_level,
                default_enabled,
                targets,
            });
        }

        rules.extend(extension_rules.iter().cloned());
        extensions.push(super::ExternalExtension { identifier, name, version, rules: extension_rules });
    }

    reader.finish()?;
    Ok(Registration { extensions, rules, has_worker_reducer })
}

pub(super) fn encode_lint_request<'arena>(
    file: &File,
    program: &Program<'arena>,
    resolved_names: &ResolvedNames<'arena>,
    active_rules: &[u16],
    target_kinds: &[bool; u8::MAX as usize + 1],
    trace_enabled: bool,
) -> Result<Option<LintRequest>, ExternalLintError> {
    if file.contents.len() > u32::MAX as usize {
        return Err(ExternalLintError::FileTooLarge(file.contents.len()));
    }

    let snapshot_start = trace_enabled.then(Instant::now);
    let Some(snapshot) = SourceSnapshot::filtered(program, resolved_names, target_kinds)? else {
        return Ok(None);
    };
    let snapshot_duration = snapshot_start.map_or(Duration::ZERO, |start| start.elapsed());
    let serialization_start = trace_enabled.then(Instant::now);
    let target_count = snapshot.target_count();
    let node_count = snapshot.node_count();
    let name_count = snapshot.name_count();
    let trivia_count = snapshot.trivia_count();
    let payload_capacity = HEADER_LENGTH
        + 4
        + file.name.len()
        + 4
        + file.contents.len()
        + 2
        + (active_rules.len() * 2)
        + snapshot.encoded_len();
    let mut writer = message_writer_with_capacity(LINT_FILE_REQUEST, payload_capacity);
    writer.write_bytes(file.name.as_ref())?;
    writer.write_bytes(file.contents.as_ref())?;
    writer.write_u16(
        u16::try_from(active_rules.len()).map_err(|_| protocol("more than u16::MAX linter rules are active"))?,
    );
    for index in active_rules {
        writer.write_u16(*index);
    }
    snapshot.write_to(&mut writer)?;

    Ok(Some(LintRequest {
        payload: writer.finish(),
        targets: target_count,
        nodes: node_count,
        names: name_count,
        trivia: trivia_count,
        snapshot_duration,
        serialization_duration: serialization_start.map_or(Duration::ZERO, |start| start.elapsed()),
    }))
}

pub(super) fn decode_lint_response(
    payload: &[u8],
    file: &File,
    rules: &[ExternalRule],
    active_rules: &[u16],
) -> Result<IssueCollection, ExternalLintError> {
    let mut reader = message_reader(payload, LINT_FILE_RESPONSE)?;
    let issue_count = reader.read_count("issues", MAXIMUM_ISSUES_PER_FILE)?;
    let mut issues = IssueCollection::new();
    issues.reserve(issue_count);
    for _ in 0..issue_count {
        let rule_index = reader.read_u16("issue rule index")?;
        if !active_rules.contains(&rule_index) {
            return Err(protocol(format!("worker reported inactive rule index `{rule_index}`")));
        }

        let rule = rules
            .get(rule_index as usize)
            .ok_or_else(|| protocol(format!("worker reported unregistered rule index `{rule_index}`")))?;
        let code = &rule.code;
        let level = rule.default_level;
        let message = reader.read_string("issue message")?;
        if message.is_empty() {
            return Err(protocol(format!("rule `{code}` reported an empty issue message")));
        }

        let note_count = reader.read_count("issue notes", MAXIMUM_NOTES_PER_ISSUE)?;
        let mut notes = Vec::with_capacity(note_count);
        for _ in 0..note_count {
            notes.push(reader.read_string("issue note")?);
        }

        let help = reader.read_optional_string("issue help")?;
        let link = reader.read_optional_string("issue link")?;
        let annotation_count = reader.read_count("issue annotations", MAXIMUM_ANNOTATIONS_PER_ISSUE)?;
        let mut annotations = Vec::with_capacity(annotation_count);
        let mut has_primary = false;
        for _ in 0..annotation_count {
            let kind = read_annotation_kind(&mut reader)?;
            let start = reader.read_u32("annotation start")?;
            let end = reader.read_u32("annotation end")?;
            if start > end || end > file.size {
                return Err(protocol(format!(
                    "rule `{code}` reported invalid annotation span {start}..{end} for a {}-byte file",
                    file.size
                )));
            }

            has_primary |= kind == AnnotationKind::Primary;
            let mut annotation = Annotation::new(kind, Span::new(file.id, Position::new(start), Position::new(end)));
            if let Some(message) = reader.read_optional_string("annotation message")? {
                annotation = annotation.with_message(message);
            }

            annotations.push(annotation);
        }

        if !has_primary {
            return Err(protocol(format!("rule `{code}` reported an issue without a primary annotation")));
        }

        let edit_count = reader.read_count("issue edits", MAXIMUM_EDITS_PER_ISSUE)?;
        let mut edits = Vec::with_capacity(edit_count);
        for _ in 0..edit_count {
            let start = reader.read_u32("edit start")?;
            let end = reader.read_u32("edit end")?;
            if start > end || end > file.size {
                return Err(protocol(format!(
                    "rule `{code}` reported invalid edit range {start}..{end} for a {}-byte file",
                    file.size
                )));
            }

            let safety = read_safety(&mut reader, "edit safety")?;
            let new_text = reader.read_bytes("edit replacement")?.to_vec();
            edits.push(TextEdit::replace(TextRange::new(start, end), new_text).with_safety(safety));
        }

        let mut issue = Issue::new(level, message)
            .with_code(code.clone())
            .with_annotations(annotations)
            .with_file_edits(file.id, edits);
        issue.notes = notes;
        issue.help = help;
        issue.link = link;
        issues.push(issue);
    }

    reader.finish()?;
    Ok(issues)
}

fn protocol(message: impl Into<String>) -> ExternalLintError {
    ExternalLintError::Protocol(message.into())
}

fn message_writer(kind: u16) -> PayloadWriter {
    message_writer_with_capacity(kind, HEADER_LENGTH)
}

fn message_writer_with_capacity(kind: u16, capacity: usize) -> PayloadWriter {
    let mut writer = PayloadWriter::with_capacity(capacity);
    writer.write_raw(&LINTER_PROTOCOL_MAGIC);
    writer.write_u16(LINTER_PROTOCOL_MAJOR);
    writer.write_u16(LINTER_PROTOCOL_MINOR);
    writer.write_u16(kind);
    writer.write_u16(0);
    writer
}

fn message_reader(payload: &[u8], expected_kind: u16) -> Result<PayloadReader<'_>, ExternalLintError> {
    let mut reader = PayloadReader::new(payload);
    if reader.read_array::<4>("message magic")? != LINTER_PROTOCOL_MAGIC {
        return Err(protocol("invalid linter message magic"));
    }

    let major = reader.read_u16("protocol major version")?;
    let minor = reader.read_u16("protocol minor version")?;
    if major != LINTER_PROTOCOL_MAJOR {
        return Err(protocol(format!("unsupported linter protocol version {major}.{minor}")));
    }

    let kind = reader.read_u16("message kind")?;
    if kind != expected_kind {
        return Err(protocol(format!("expected linter message kind {expected_kind}, received {kind}")));
    }

    let reserved = reader.read_u16("reserved header")?;
    if reserved != 0 {
        return Err(protocol(format!("linter message reserved header is non-zero: {reserved:#06x}")));
    }

    Ok(reader)
}

fn read_level(reader: &mut PayloadReader<'_>) -> Result<Level, ExternalLintError> {
    match reader.read_u8("severity level")? {
        1 => Ok(Level::Note),
        2 => Ok(Level::Help),
        3 => Ok(Level::Warning),
        4 => Ok(Level::Error),
        value => Err(protocol(format!("invalid severity level {value}"))),
    }
}

fn read_annotation_kind(reader: &mut PayloadReader<'_>) -> Result<AnnotationKind, ExternalLintError> {
    match reader.read_u8("annotation kind")? {
        1 => Ok(AnnotationKind::Primary),
        2 => Ok(AnnotationKind::Secondary),
        value => Err(protocol(format!("invalid annotation kind {value}"))),
    }
}

fn read_safety(reader: &mut PayloadReader<'_>, description: &'static str) -> Result<Safety, ExternalLintError> {
    match reader.read_u8(description)? {
        1 => Ok(Safety::Safe),
        2 => Ok(Safety::PotentiallyUnsafe),
        3 => Ok(Safety::Unsafe),
        value => Err(protocol(format!("invalid text edit safety {value}"))),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(super) mod testing {
    use super::*;

    type RuleDescription<'rule> = (&'rule str, &'rule str, &'rule str, Level, bool, &'rule [NodeKind]);

    #[derive(Debug, PartialEq, Eq)]
    pub struct DecodedNode {
        pub kind: String,
        pub start: u32,
        pub end: u32,
        pub parent: Option<u32>,
        pub children: Vec<u32>,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct DecodedRequest {
        pub file_name: Vec<u8>,
        pub source: Vec<u8>,
        pub active_rules: Vec<u16>,
        pub targets: Vec<u32>,
        pub nodes: Vec<DecodedNode>,
        pub names: Vec<(u32, u32, Vec<u8>, bool)>,
        pub trivia: Vec<(String, u32, u32)>,
    }

    pub fn describe_response(identifier: &str, name: &str, version: &str, rules: &[RuleDescription<'_>]) -> Vec<u8> {
        describe_extensions_response(&[(identifier, name, version, rules)])
    }

    pub fn describe_extensions_response(extensions: &[(&str, &str, &str, &[RuleDescription<'_>])]) -> Vec<u8> {
        let mut writer = message_writer(DESCRIBE_RESPONSE);
        writer.write_length(extensions.len()).unwrap();
        for (identifier, extension_name, version, rules) in extensions {
            writer.write_string(identifier).unwrap();
            writer.write_string(extension_name).unwrap();
            writer.write_string(version).unwrap();
            writer.write_bool(false);
            writer.write_length(rules.len()).unwrap();
            for (code, rule_name, description, level, enabled, targets) in *rules {
                writer.write_string(code).unwrap();
                writer.write_string(rule_name).unwrap();
                writer.write_string(description).unwrap();
                writer.write_u8(level_value(*level));
                writer.write_bool(*enabled);
                writer.write_length(targets.len()).unwrap();
                for target in *targets {
                    writer.write_string(&target.to_string()).unwrap();
                }
            }
        }

        writer.finish()
    }

    pub fn lint_response(issues: &[(u16, Issue)]) -> Vec<u8> {
        let mut writer = message_writer(LINT_FILE_RESPONSE);
        writer.write_length(issues.len()).unwrap();
        for (rule_index, issue) in issues {
            writer.write_u16(*rule_index);
            writer.write_string(&issue.message).unwrap();
            writer.write_length(issue.notes.len()).unwrap();
            for note in &issue.notes {
                writer.write_string(note).unwrap();
            }

            optional_string(&mut writer, issue.help.as_deref());
            optional_string(&mut writer, issue.link.as_deref());
            writer.write_length(issue.annotations.len()).unwrap();
            for annotation in &issue.annotations {
                writer.write_u8(match annotation.kind {
                    AnnotationKind::Primary => 1,
                    AnnotationKind::Secondary => 2,
                });
                writer.write_u32(annotation.span.start.offset);
                writer.write_u32(annotation.span.end.offset);
                optional_string(&mut writer, annotation.message.as_deref());
            }
            writer.write_length(issue.edits.values().map(Vec::len).sum()).unwrap();
            for edits in issue.edits.values() {
                for edit in edits {
                    writer.write_u32(edit.range.start);
                    writer.write_u32(edit.range.end);
                    writer.write_u8(match edit.safety {
                        Safety::Safe => 1,
                        Safety::PotentiallyUnsafe => 2,
                        Safety::Unsafe => 3,
                        _ => 3,
                    });
                    writer.write_bytes(&edit.new_text).unwrap();
                }
            }
        }

        writer.finish()
    }

    pub fn decode_lint_request(payload: &[u8]) -> Result<DecodedRequest, ExternalLintError> {
        let mut reader = message_reader(payload, LINT_FILE_REQUEST)?;
        let file_name = reader.read_bytes("file name")?.to_vec();
        let source = reader.read_bytes("source")?.to_vec();
        let active_count = reader.read_u16("active rule count")? as usize;
        let mut active_rules = Vec::with_capacity(active_count);
        for _ in 0..active_count {
            active_rules.push(reader.read_u16("active rule index")?);
        }

        let kinds = NodeKind::iter().map(|kind| kind.to_string()).collect::<Vec<_>>();

        let target_count = reader.read_u32("target node count")? as usize;
        let mut targets = Vec::with_capacity(target_count);
        for _ in 0..target_count {
            targets.push(reader.read_u32("target node identifier")?);
        }

        let node_count = reader.read_u32("node count")? as usize;
        let mut raw_nodes = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            let kind_index = reader.read_u8("node kind index")? as usize;
            let kind =
                kinds.get(kind_index).ok_or_else(|| protocol(format!("invalid node kind index {kind_index}")))?.clone();
            let start = reader.read_u32("node start")?;
            let end = reader.read_u32("node end")?;
            let parent = match reader.read_u32("node parent")? {
                NO_NODE => None,
                parent => Some(parent),
            };
            let first_child = reader.read_u32("first child")?;
            let next_sibling = reader.read_u32("next sibling")?;

            raw_nodes.push((kind, start, end, parent, first_child, next_sibling));
        }

        let mut nodes = Vec::with_capacity(node_count);
        for (kind, start, end, parent, first_child, _) in &raw_nodes {
            let mut children = Vec::new();
            let mut child = *first_child;
            while child != NO_NODE {
                children.push(child);
                if children.len() > node_count {
                    return Err(protocol("cycle in node sibling list"));
                }

                child = raw_nodes
                    .get(child as usize)
                    .ok_or_else(|| protocol(format!("invalid child node identifier {child}")))?
                    .5;
            }

            nodes.push(DecodedNode { kind: kind.clone(), start: *start, end: *end, parent: *parent, children });
        }

        let name_count = reader.read_u32("name count")? as usize;
        let mut name_starts = Vec::with_capacity(name_count);
        for _ in 0..name_count {
            name_starts.push(reader.read_u32("name start")?);
        }

        let mut name_records = Vec::with_capacity(name_count);
        for _ in 0..name_count {
            name_records.push((
                reader.read_u32("name end")?,
                reader.read_u32("name offset")? as usize,
                reader.read_u32("name length")? as usize,
                reader.read_bool("imported")?,
            ));
        }

        let names_buffer = reader.read_bytes("resolved names")?;
        let mut names = Vec::with_capacity(name_count);
        for (start, (end, offset, length, imported)) in name_starts.into_iter().zip(name_records) {
            let name = names_buffer
                .get(offset..offset + length)
                .ok_or_else(|| protocol("resolved name points outside the name buffer"))?;
            names.push((start, end, name.to_vec(), imported));
        }

        let trivia_count = reader.read_u32("trivia count")? as usize;
        let mut trivia = Vec::with_capacity(trivia_count);
        for _ in 0..trivia_count {
            let kind = match reader.read_u8("trivia kind")? {
                1 => "SingleLineComment",
                2 => "MultiLineComment",
                3 => "HashComment",
                4 => "DocBlockComment",
                value => return Err(protocol(format!("invalid trivia kind {value}"))),
            };
            trivia.push((kind.to_owned(), reader.read_u32("trivia start")?, reader.read_u32("trivia end")?));
        }

        reader.finish()?;
        Ok(DecodedRequest { file_name, source, active_rules, targets, nodes, names, trivia })
    }

    fn optional_string(writer: &mut PayloadWriter, value: Option<&str>) {
        writer.write_optional_string(value).unwrap();
    }

    fn level_value(level: Level) -> u8 {
        match level {
            Level::Note => 1,
            Level::Help => 2,
            Level::Warning => 3,
            Level::Error => 4,
        }
    }
}
