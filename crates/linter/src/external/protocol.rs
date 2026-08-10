//! Stable binary messages carried inside `mago-extension` frames.
//!
//! All integers are unsigned and big-endian. Strings and byte strings are a
//! `u32` byte length followed by that many bytes. The fixed message header is:
//! `MLNT`, protocol major, protocol minor, message kind, reserved zero.

#![allow(clippy::big_endian_bytes, reason = "network byte order is part of the stable linter wire format")]

use std::collections::HashSet;

use mago_database::file::File;
use mago_extension::PayloadReader;
use mago_extension::PayloadWriter;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Program;

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
const NO_PARENT: u32 = u32::MAX;
const MAXIMUM_EXTENSIONS_RULES: usize = 0x4000;
const MAXIMUM_TARGETS_PER_RULE: usize = 512;
const MAXIMUM_ISSUES_PER_FILE: usize = 1_000_000;
const MAXIMUM_ANNOTATIONS_PER_ISSUE: usize = 0x0001_0000;
const MAXIMUM_NOTES_PER_ISSUE: usize = 0x0001_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Registration {
    pub identifier: String,
    pub name: String,
    pub version: String,
    pub rules: Vec<ExternalRule>,
}

#[derive(Debug)]
struct SnapshotNode {
    kind: NodeKind,
    start: u32,
    end: u32,
    parent: Option<u32>,
    children: Vec<u32>,
}

#[derive(Debug)]
struct FileSnapshot<'source> {
    nodes: Vec<SnapshotNode>,
    names: Vec<(u32, u32, &'source [u8], bool)>,
    trivia: Vec<(&'static str, u32, u32)>,
}

impl<'source> FileSnapshot<'source> {
    fn build<'ast, 'arena>(
        file: &'source File,
        program: &'ast Program<'arena>,
        resolved_names: &'source ResolvedNames<'arena>,
    ) -> Result<Self, ExternalLintError> {
        if file.contents.len() > u32::MAX as usize {
            return Err(ExternalLintError::FileTooLarge(file.contents.len()));
        }

        let mut nodes = Vec::new();
        let mut stack = vec![(Node::Program(program), None)];
        while let Some((node, parent)) = stack.pop() {
            let identifier = u32::try_from(nodes.len()).map_err(|_| {
                ExternalLintError::Protocol("syntax tree contains more than u32::MAX nodes".to_string())
            })?;
            let span = node.span();
            nodes.push(SnapshotNode {
                kind: node.kind(),
                start: span.start.offset,
                end: span.end.offset,
                parent,
                children: Vec::new(),
            });

            if let Some(parent) = parent {
                nodes[parent as usize].children.push(identifier);
            }

            let start = stack.len();
            node.visit_children(|child| stack.push((child, Some(identifier))));
            stack[start..].reverse();
        }

        let mut names: Vec<_> = resolved_names.iter().collect();
        names.sort_unstable_by_key(|(start, end, _, _)| (*start, *end));

        let trivia = program
            .trivia
            .iter()
            .filter(|trivia| trivia.kind.is_comment())
            .map(|trivia| {
                let kind = match trivia.kind {
                    mago_syntax::cst::TriviaKind::SingleLineComment => "SingleLineComment",
                    mago_syntax::cst::TriviaKind::MultiLineComment => "MultiLineComment",
                    mago_syntax::cst::TriviaKind::HashComment => "HashComment",
                    mago_syntax::cst::TriviaKind::DocBlockComment => "DocBlockComment",
                    mago_syntax::cst::TriviaKind::WhiteSpace => unreachable!("whitespace was filtered out"),
                };

                (kind, trivia.span.start.offset, trivia.span.end.offset)
            })
            .collect();

        Ok(Self { nodes, names, trivia })
    }
}

pub(super) fn encode_describe_request(php_version: PHPVersion) -> Vec<u8> {
    let mut writer = message_writer(DESCRIBE_REQUEST);
    writer.write_u32(php_version.to_version_id());
    writer.finish()
}

pub(super) fn decode_registration(payload: &[u8]) -> Result<Registration, ExternalLintError> {
    let mut reader = message_reader(payload, DESCRIBE_RESPONSE)?;
    let identifier = reader.read_string("extension identifier")?;
    let name = reader.read_string("extension name")?;
    let version = reader.read_string("extension version")?;
    if identifier.is_empty() {
        return Err(protocol("extension identifier cannot be empty"));
    }

    if name.is_empty() {
        return Err(protocol("extension name cannot be empty"));
    }

    let rule_count = reader.read_count("rules", MAXIMUM_EXTENSIONS_RULES)?;
    let mut rules = Vec::with_capacity(rule_count);
    let mut codes = HashSet::with_capacity(rule_count);
    for _ in 0..rule_count {
        let code = reader.read_string("rule code")?;
        let name = reader.read_string("rule name")?;
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

        if name.is_empty() {
            return Err(protocol(format!("rule `{code}` has an empty name")));
        }

        if targets.is_empty() {
            return Err(protocol(format!("rule `{code}` has no target node kinds")));
        }

        if !codes.insert(code.clone()) {
            return Err(protocol(format!("extension `{identifier}` advertises rule `{code}` more than once")));
        }

        rules.push(ExternalRule {
            extension: identifier.clone(),
            code,
            name,
            description,
            default_level,
            default_enabled,
            targets,
        });
    }

    reader.finish()?;
    Ok(Registration { identifier, name, version, rules })
}

pub(super) fn encode_lint_request<'arena>(
    php_version: PHPVersion,
    file: &File,
    program: &Program<'arena>,
    resolved_names: &ResolvedNames<'arena>,
    active_rules: &[&str],
) -> Result<Vec<u8>, ExternalLintError> {
    let snapshot = FileSnapshot::build(file, program, resolved_names)?;
    let mut writer = message_writer(LINT_FILE_REQUEST);
    writer.write_u32(php_version.to_version_id());
    writer.write_bytes(file.name.as_ref())?;
    writer.write_bytes(file.contents.as_ref())?;
    writer.write_length(active_rules.len())?;
    for code in active_rules {
        writer.write_string(code)?;
    }

    // Node kinds are stable names on the wire, but each name is written only
    // once. Nodes refer to this per-message dictionary with a compact u16.
    let mut kind_indices = [u16::MAX; u8::MAX as usize + 1];
    let mut kinds = Vec::new();
    for node in &snapshot.nodes {
        let slot = &mut kind_indices[node.kind as usize];
        if *slot == u16::MAX {
            *slot =
                u16::try_from(kinds.len()).map_err(|_| protocol("syntax tree uses more than u16::MAX node kinds"))?;
            kinds.push(node.kind);
        }
    }

    writer.write_length(kinds.len())?;
    for kind in kinds {
        writer.write_string(&kind.to_string())?;
    }

    writer.write_length(snapshot.nodes.len())?;
    for node in &snapshot.nodes {
        writer.write_u16(kind_indices[node.kind as usize]);
        writer.write_u32(node.start);
        writer.write_u32(node.end);
        writer.write_u32(node.parent.unwrap_or(NO_PARENT));
        writer.write_length(node.children.len())?;
        for child in &node.children {
            writer.write_u32(*child);
        }
    }

    writer.write_length(snapshot.names.len())?;
    for (start, end, name, imported) in snapshot.names {
        writer.write_u32(start);
        writer.write_u32(end);
        writer.write_bytes(name)?;
        writer.write_bool(imported);
    }

    writer.write_length(snapshot.trivia.len())?;
    for (kind, start, end) in snapshot.trivia {
        writer.write_string(kind)?;
        writer.write_u32(start);
        writer.write_u32(end);
    }

    Ok(writer.finish())
}

pub(super) fn decode_lint_response(
    payload: &[u8],
    file: &File,
    active_rules: &HashSet<&str>,
) -> Result<IssueCollection, ExternalLintError> {
    let mut reader = message_reader(payload, LINT_FILE_RESPONSE)?;
    let issue_count = reader.read_count("issues", MAXIMUM_ISSUES_PER_FILE)?;
    let mut issues = IssueCollection::new();
    issues.reserve(issue_count);
    for _ in 0..issue_count {
        let code = reader.read_string("issue code")?;
        if !active_rules.contains(code.as_str()) {
            return Err(protocol(format!("worker reported inactive or unregistered rule `{code}`")));
        }

        let level = read_level(&mut reader)?;
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

        let mut issue = Issue::new(level, message).with_code(code).with_annotations(annotations);
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
    let mut writer = PayloadWriter::with_capacity(HEADER_LENGTH);
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
        pub php_version: u32,
        pub file_name: Vec<u8>,
        pub source: Vec<u8>,
        pub active_rules: Vec<String>,
        pub nodes: Vec<DecodedNode>,
        pub names: Vec<(u32, u32, Vec<u8>, bool)>,
        pub trivia: Vec<(String, u32, u32)>,
    }

    pub fn describe_response(identifier: &str, name: &str, version: &str, rules: &[RuleDescription<'_>]) -> Vec<u8> {
        let mut writer = message_writer(DESCRIBE_RESPONSE);
        writer.write_string(identifier).unwrap();
        writer.write_string(name).unwrap();
        writer.write_string(version).unwrap();
        writer.write_length(rules.len()).unwrap();
        for (code, name, description, level, enabled, targets) in rules {
            writer.write_string(code).unwrap();
            writer.write_string(name).unwrap();
            writer.write_string(description).unwrap();
            writer.write_u8(level_value(*level));
            writer.write_bool(*enabled);
            writer.write_length(targets.len()).unwrap();
            for target in *targets {
                writer.write_string(&target.to_string()).unwrap();
            }
        }

        writer.finish()
    }

    pub fn lint_response(issues: &[Issue]) -> Vec<u8> {
        let mut writer = message_writer(LINT_FILE_RESPONSE);
        writer.write_length(issues.len()).unwrap();
        for issue in issues {
            writer.write_string(issue.code.as_deref().unwrap()).unwrap();
            writer.write_u8(level_value(issue.level));
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
        }

        writer.finish()
    }

    pub fn decode_lint_request(payload: &[u8]) -> Result<DecodedRequest, ExternalLintError> {
        let mut reader = message_reader(payload, LINT_FILE_REQUEST)?;
        let php_version = reader.read_u32("PHP version")?;
        let file_name = reader.read_bytes("file name")?.to_vec();
        let source = reader.read_bytes("source")?.to_vec();
        let active_count = reader.read_u32("active rule count")? as usize;
        let mut active_rules = Vec::with_capacity(active_count);
        for _ in 0..active_count {
            active_rules.push(reader.read_string("active rule")?);
        }

        let kind_count = reader.read_u32("node kind count")? as usize;
        let mut kinds = Vec::with_capacity(kind_count);
        for _ in 0..kind_count {
            kinds.push(reader.read_string("node kind")?);
        }

        let node_count = reader.read_u32("node count")? as usize;
        let mut nodes = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            let kind_index = reader.read_u16("node kind index")? as usize;
            let kind =
                kinds.get(kind_index).ok_or_else(|| protocol(format!("invalid node kind index {kind_index}")))?.clone();
            let start = reader.read_u32("node start")?;
            let end = reader.read_u32("node end")?;
            let parent = match reader.read_u32("node parent")? {
                NO_PARENT => None,
                parent => Some(parent),
            };
            let child_count = reader.read_u32("child count")? as usize;
            let mut children = Vec::with_capacity(child_count);
            for _ in 0..child_count {
                children.push(reader.read_u32("child")?);
            }

            nodes.push(DecodedNode { kind, start, end, parent, children });
        }

        let name_count = reader.read_u32("name count")? as usize;
        let mut names = Vec::with_capacity(name_count);
        for _ in 0..name_count {
            names.push((
                reader.read_u32("name start")?,
                reader.read_u32("name end")?,
                reader.read_bytes("resolved name")?.to_vec(),
                reader.read_bool("imported")?,
            ));
        }

        let trivia_count = reader.read_u32("trivia count")? as usize;
        let mut trivia = Vec::with_capacity(trivia_count);
        for _ in 0..trivia_count {
            trivia.push((
                reader.read_string("trivia kind")?,
                reader.read_u32("trivia start")?,
                reader.read_u32("trivia end")?,
            ));
        }

        reader.finish()?;
        Ok(DecodedRequest { php_version, file_name, source, active_rules, nodes, names, trivia })
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
