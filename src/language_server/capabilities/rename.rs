//! Convert core rename ranges, edits and refusals to the client's source URIs, positions and messages.

use std::fmt;
use std::fmt::Write;
use std::sync::Arc;

use foldhash::HashMap;
use tower_lsp_server::gen_lsp_types::{PrepareRenamePlaceholder, PrepareRenameResult, TextEdit, Uri, WorkspaceEdit};

use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_server::{DocProblem, NameProblem, Range, RefusalReason, RenameRefusal};

use crate::language_server::codec;
use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;

const SNIPPET_CHARS: usize = 60;

pub fn prepare(
    workspace: &mut WorkspaceState,
    file: &MagoFile,
    offset: u32,
) -> Option<Result<PrepareRenameResult, String>> {
    let range = match workspace.core.server.get_prepare_rename(file.id, offset) {
        Ok(range) => range,
        Err(refusal) if refusal.reason == RefusalReason::NoSymbol => return None,
        Err(refusal) => return Some(Err(message(workspace, &refusal, None))),
    };
    Some(Ok(PrepareRenamePlaceholder {
        range: range_at_offsets(file, range.start, range.end),
        placeholder: String::from_utf8_lossy(&file.contents[range.start as usize..range.end as usize]).into_owned(),
    }
    .into()))
}

pub fn compute(
    workspace: &mut WorkspaceState,
    file: &MagoFile,
    offset: u32,
    new_name: String,
) -> Option<Result<WorkspaceEdit, String>> {
    let edits = match workspace.core.server.get_rename(file.id, offset, &new_name) {
        Ok(edits) => edits,
        Err(refusal) => return Some(Err(message(workspace, &refusal, Some(&new_name)))),
    };
    let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::default();
    for (file, edits) in edits {
        let file = workspace.database().get(&file).ok()?;
        let uri = workspace.document_uri(&file)?;
        changes.entry(uri).or_default().extend(edits.into_iter().map(|edit| codec::text_edit(&file, edit)));
    }
    Some(Ok(WorkspaceEdit { changes: Some(changes.into_iter().collect()), ..WorkspaceEdit::default() }))
}

fn message(workspace: &WorkspaceState, refusal: &RenameRefusal, new_name: Option<&str>) -> String {
    let site = refusal.location.and_then(|location| {
        Some(Site { file: workspace.database().get(&location.file).ok()?, range: location.range })
    });
    let site = site.as_ref();
    let snippet = Snippet(site);
    let place = Place(site);
    let mut message = String::with_capacity(160);
    let _ = match new_name {
        Some(name) => write!(message, "Cannot rename to \"{name}\": "),
        None => write!(message, "Cannot rename: "),
    };
    let _ = match refusal.reason {
        RefusalReason::NoSymbol => write!(message, "no renameable symbol at the cursor."),
        RefusalReason::Namespace { import: false } => {
            write!(message, "{snippet} is a namespace; namespace renames are not supported.")
        }
        RefusalReason::Namespace { import: true } => {
            write!(message, "{snippet} is a namespace import at {place}; namespace renames are not supported.")
        }
        RefusalReason::NamespaceSegment => {
            write!(message, "the cursor is on the namespace part of {snippet}; select its last segment.")
        }
        RefusalReason::InvalidName(problem) => message.write_str(match problem {
            NameProblem::Empty => "the new name is empty.",
            NameProblem::NotIdentifier => "it is not a valid PHP identifier.",
            NameProblem::Keyword => "it is a reserved PHP keyword.",
            NameProblem::ReservedType => "it is a reserved type name.",
            NameProblem::ReservedVariable => "it is a reserved variable name.",
            NameProblem::ReservedMethodName => "method names starting with __ are reserved for magic methods.",
            NameProblem::ReservedConstant => "class is reserved for the ::class constant.",
        }),
        RefusalReason::ReadOnly if site.is_some_and(Site::builtin) => {
            write!(message, "PHP's builtin stubs are read-only.")
        }
        RefusalReason::ReadOnly => write!(message, "{place} is outside the project sources and cannot be edited."),
        RefusalReason::Builtin => write!(message, "{snippet} is declared in PHP's builtin stubs."),
        RefusalReason::EnumMethod => write!(message, "PHP generates this method for the enum {snippet} at {place}."),
        RefusalReason::External => write!(message, "{snippet} is declared outside the project sources, at {place}."),
        RefusalReason::Undeclared => write!(message, "{snippet} has no declaration in the project."),
        RefusalReason::MagicMethod => write!(message, "{snippet} is a magic method."),
        RefusalReason::Magic => {
            write!(message, "{snippet} at {place} goes through a magic method or virtual property.")
        }
        RefusalReason::Dynamic => write!(message, "{snippet} at {place} is a dynamic member access."),
        RefusalReason::Unresolved => write!(message, "the target of {snippet} at {place} cannot be resolved."),
        RefusalReason::SharedTarget => {
            write!(message, "{snippet} at {place} may also refer to a member outside this rename.")
        }
        RefusalReason::IncompleteHierarchy(dependency) => {
            write!(message, "{snippet} at {place} depends on {dependency}, which is not declared.")
        }
        RefusalReason::NameTaken => write!(message, "{snippet} already exists at {place}."),
        RefusalReason::BindingTaken => write!(message, "{snippet} is already defined in the same scope at {place}."),
        RefusalReason::AliasTaken => write!(message, "{snippet} at {place} already uses that name in this namespace."),
        RefusalReason::NamedArgTaken => {
            write!(message, "the named argument {snippet} at {place} already uses that name.")
        }
        RefusalReason::Capture => write!(message, "it would change what {snippet} at {place} refers to."),
        RefusalReason::StringReference { proposed } => write!(
            message,
            "the {} appears in the string {snippet} at {place}, which PHP may resolve at runtime.",
            if proposed { "new name" } else { "name" }
        ),
        RefusalReason::ImportPrefix => {
            write!(message, "the import {snippet} at {place} is also used as a namespace prefix.")
        }
        RefusalReason::RuntimeVariables => write!(message, "{snippet} at {place} uses variables by name at runtime."),
        RefusalReason::ImplicitParameter => {
            write!(message, "{snippet} at {place} is the implicit parameter of a set hook.")
        }
        RefusalReason::RuntimeDeclaration => {
            write!(message, "the name is declared at runtime by {snippet} at {place}.")
        }
        RefusalReason::TraitAdaptation => write!(message, "a trait adaptation at {place} refers to {snippet}."),
        RefusalReason::Doc(DocProblem::Unparsed) => write!(message, "the PHPDoc at {place} cannot be parsed."),
        RefusalReason::Doc(DocProblem::Template) => {
            write!(message, "{snippet} in the PHPDoc at {place} may refer to a template or type alias.")
        }
        RefusalReason::Doc(DocProblem::Capture) => {
            write!(message, "{snippet} in the PHPDoc at {place} would refer to a different class.")
        }
        RefusalReason::AnalysisDisabled => {
            write!(message, "member renames need analysis results for {place}, which are unavailable.")
        }
        RefusalReason::Stale if site.is_some() => {
            write!(message, "the server could not check {place}; try again after the next analysis.")
        }
        RefusalReason::Stale => {
            write!(message, "the server's index is out of date; try again after the next analysis.")
        }
    };
    message
}

struct Site {
    file: Arc<MagoFile>,
    range: Range,
}

impl Site {
    fn builtin(&self) -> bool {
        self.file.file_type.is_builtin()
    }
}

struct Snippet<'a>(Option<&'a Site>);

impl fmt::Display for Snippet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(site) = self.0 else { return f.write_str("the symbol") };
        let bytes = site.file.contents.get(site.range.start as usize..site.range.end as usize).unwrap_or_default();
        let bytes = bytes.trim_ascii();
        let line = bytes.split(|byte| *byte == b'\n').next().unwrap_or_default().trim_ascii_end();
        let text = String::from_utf8_lossy(line);
        match text.char_indices().nth(SNIPPET_CHARS) {
            Some((end, _)) => write!(f, "{}…", &text[..end]),
            None if line.len() < bytes.len() => write!(f, "{text}…"),
            None => f.write_str(&text),
        }
    }
}

struct Place<'a>(Option<&'a Site>);

impl fmt::Display for Place<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => f.write_str("an unknown location"),
            Some(site) if site.builtin() => f.write_str("PHP's builtin stubs"),
            Some(site) => write!(
                f,
                "{}:{}",
                String::from_utf8_lossy(&site.file.name),
                site.file.line_number(site.range.start) + 1
            ),
        }
    }
}
