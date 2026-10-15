//! Rename member declarations and their checked uses as one family.

use mago_analyzer::artifacts::editor::MemberKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_span::Span;
use mago_word::Word;
use mago_word::word;

use crate::capabilities::rename::{NO_SYMBOL, STALE, declared_outside, invalid};
use crate::symbol_resolution::{declared_member_at, members_at, methods_at};
use crate::{NameProblem, Range, RefusalReason, RenameRefusal, Server, SymbolLocation};

mod callables;
mod family;
mod references;

pub(crate) use family::Family;

#[derive(Debug)]
pub struct MemberRename {
    pub range: Range,
    pub locations: Vec<SymbolLocation>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Method,
    Property,
    Constant,
}

struct Selection {
    kind: Kind,
    name: Word,
    file: FileId,
    range: Range,
    declaration: Span,
}

impl Server {
    pub(crate) fn method_family(&self, method: &FunctionLikeMetadata) -> Result<Family, RenameRefusal> {
        let span = method.name_span.ok_or(STALE)?;
        let selected = Selection {
            kind: Kind::Method,
            name: method.name,
            file: span.file_id,
            range: Range::new(span.start.offset, span.end.offset),
            declaration: span,
        };
        Family::resolve(self, &selected)
    }

    /// Return complete member edits, or decline when a use cannot be checked.
    ///
    /// # Errors
    ///
    /// Returns a [`RenameRefusal`] with the check that declined the rename.
    pub fn get_member_rename(
        &mut self,
        file: FileId,
        offset: u32,
        new_name: Option<&[u8]>,
    ) -> Result<MemberRename, RenameRefusal> {
        let source = self.rename_source(file, offset)?;
        let selected = select(self, file, offset)?.ok_or(NO_SYMBOL)?;
        let selection_range =
            checked_range(&source, selected.range, selected.name.as_bytes(), selected.kind).ok_or(STALE)?;
        if selected.kind == Kind::Method {
            if selected.name.as_bytes().starts_with(b"__") {
                return Err(RenameRefusal::at(RefusalReason::MagicMethod, file, selection_range));
            }
            if new_name.is_some_and(|name| name.starts_with(b"__")) {
                return Err(invalid(NameProblem::ReservedMethodName));
            }
        }
        if selected.kind == Kind::Constant && new_name.is_some_and(|name| name.eq_ignore_ascii_case(b"class")) {
            return Err(invalid(NameProblem::ReservedConstant));
        }
        let declaration = self.database().get(&selected.declaration.file_id).map_err(|_| STALE)?;
        if declaration.file_type != FileType::Host {
            return Err(declared_outside(&declaration, selected.declaration));
        }
        let replacement = new_name.map(|name| {
            if selected.kind == Kind::Property { word([b"$".as_slice(), name].concat()) } else { word(name) }
        });
        let family = Family::resolve(self, &selected)?;
        family.refuse_outside(self, family.declarations.iter().copied(), |span| family.owner(self.codebase(), span))?;
        family.check(self, replacement)?;
        let mut locations = Vec::new();
        for span in &family.declarations {
            let file = self.database().get(&span.file_id).map_err(|_| STALE)?;
            let range = Range::new(span.start.offset, span.end.offset);
            locations.push(SymbolLocation {
                file: file.id,
                range: checked_range(&file, range, selected.name.as_bytes(), selected.kind)
                    .ok_or(RenameRefusal::at(RefusalReason::Stale, file.id, range))?,
            });
        }
        let files: Vec<_> = self.database().files().filter(|file| file.file_type == FileType::Host).collect();
        for file in files {
            references::collect(self, &file, &selected, replacement, &family, &mut locations)?;
        }
        if selected.kind == Kind::Property {
            super::rename::extend_properties(self, &family.declarations, new_name, &mut locations)?;
        }
        locations.sort_unstable_by_key(|location| (location.file, location.range.start, location.range.end));
        locations.dedup();
        Ok(MemberRename { range: selection_range, locations })
    }
}

fn select(server: &Server, file: FileId, offset: u32) -> Result<Option<Selection>, RenameRefusal> {
    if let Some(editor) = server.editor_file(file) {
        if let Some(row) = methods_at(&editor, offset).first() {
            let range = Range::new(row.selector_span.0, row.selector_span.1);
            unsafe_use(file, range, row.dynamic, row.magic, row.uncertain)?;
            if server
                .codebase()
                .get_method_by_id(&row.method)
                .is_some_and(|method| method.name_span == Some(method.span))
            {
                return Err(RenameRefusal::at(RefusalReason::Builtin, file, range));
            }
            return Ok(Some(Selection {
                kind: Kind::Method,
                name: row.method.get_method_name(),
                file,
                range,
                declaration: row.declaration.ok_or(RenameRefusal::at(RefusalReason::Unresolved, file, range))?,
            }));
        }
        if let Some(row) = members_at(&editor, offset).first() {
            let range = Range::new(row.selector_span.0, row.selector_span.1);
            unsafe_use(file, range, row.dynamic, row.magic, row.uncertain)?;
            return Ok(Some(Selection {
                kind: if row.kind == MemberKind::Property { Kind::Property } else { Kind::Constant },
                name: row.name,
                file,
                range,
                declaration: row.declaration.ok_or(RenameRefusal::at(RefusalReason::Unresolved, file, range))?,
            }));
        }
    }
    let Some(span) = declared_member_at(server.codebase(), file, offset) else { return Ok(None) };
    for class in server.codebase().class_likes.values().filter(|class| class.span.file_id == file) {
        let names = class
            .methods
            .iter()
            .map(|name| (Kind::Method, *name))
            .chain(class.properties.keys().map(|name| (Kind::Property, *name)))
            .chain(class.constants.keys().chain(class.enum_cases.keys()).map(|name| (Kind::Constant, *name)));
        for (kind, name) in names {
            if family::declaring_span(server.codebase(), class.name, name, kind) == Some(span) {
                return Ok(Some(Selection {
                    kind,
                    name,
                    file,
                    range: Range::new(span.start.offset, span.end.offset),
                    declaration: span,
                }));
            }
        }
    }
    Ok(None)
}

const fn use_problem(dynamic: bool, magic: bool, uncertain: bool) -> Option<RefusalReason> {
    if dynamic {
        Some(RefusalReason::Dynamic)
    } else if magic {
        Some(RefusalReason::Magic)
    } else if uncertain {
        Some(RefusalReason::Unresolved)
    } else {
        None
    }
}

fn unsafe_use(file: FileId, range: Range, dynamic: bool, magic: bool, uncertain: bool) -> Result<(), RenameRefusal> {
    match use_problem(dynamic, magic, uncertain) {
        Some(reason) => Err(RenameRefusal::at(reason, file, range)),
        None => Ok(()),
    }
}

fn same_name(kind: Kind, left: &[u8], right: &[u8]) -> bool {
    let left = left.strip_prefix(b"$").unwrap_or(left);
    let right = right.strip_prefix(b"$").unwrap_or(right);
    if kind == Kind::Method { left.eq_ignore_ascii_case(right) } else { left == right }
}

fn checked_range(file: &File, range: Range, name: &[u8], kind: Kind) -> Option<Range> {
    let text = file.contents.get(range.start as usize..range.end as usize)?;
    if !same_name(kind, text, name) {
        return None;
    }
    Some(Range::new(range.start + u32::from(text.starts_with(b"$")), range.end))
}
