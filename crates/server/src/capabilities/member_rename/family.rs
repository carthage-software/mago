//! Find related declarations and reject collisions across their inheritance tree.

use foldhash::{HashMap, HashSet};
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_span::Span;
use mago_word::{Word, word};

use crate::capabilities::rename::{STALE, declared_outside};
use crate::{Range, RefusalReason, RenameRefusal, Server};

use super::{Kind, Selection};

pub(crate) struct Family {
    pub declarations: HashSet<Span>,
    pub classes: HashSet<Word>,
    kind: Kind,
    targets: HashMap<Word, Target>,
}

#[derive(Clone, Copy)]
struct Target {
    span: Span,
    private: bool,
    safe: bool,
    builtin: bool,
}

impl Family {
    pub(super) fn resolve(server: &Server, selected: &Selection) -> Result<Self, RenameRefusal> {
        let codebase = server.codebase();
        let targets: HashMap<_, _> = codebase
            .class_likes
            .values()
            .filter_map(|class| {
                target(codebase, class.name, selected.name, selected.kind).map(|target| (class.name, target))
            })
            .collect();
        let selected_target = targets.values().find(|target| target.span == selected.declaration).ok_or(STALE)?;
        if selected_target.builtin {
            return Err(RenameRefusal::at(RefusalReason::Builtin, selected.file, selected.range));
        }
        let mut family = HashSet::from_iter([selected.declaration]);
        if !selected_target.private {
            loop {
                let before = family.len();
                for (name, current) in &targets {
                    let class = codebase.get_class_like_by_name(*name).ok_or(STALE)?;
                    for parent in class
                        .direct_parent_class
                        .iter()
                        .chain(class.direct_parent_interfaces.iter())
                        .chain(class.used_traits.iter())
                    {
                        let Some(parent) = targets.get(parent) else { continue };
                        if !current.private
                            && !parent.private
                            && (family.contains(&current.span) || family.contains(&parent.span))
                        {
                            family.extend([current.span, parent.span]);
                        }
                    }
                }
                if family.len() == before {
                    break;
                }
            }
        }
        let mut generated = None;
        for (class, target) in &targets {
            if !target.builtin || !family.contains(&target.span) {
                continue;
            }
            let class = codebase.get_class_like_by_name(*class).ok_or(STALE)?;
            let declared = class
                .direct_parent_class
                .iter()
                .chain(class.direct_parent_interfaces.iter())
                .chain(class.used_traits.iter())
                .filter_map(|parent| targets.get(parent))
                .any(|parent| {
                    family.contains(&parent.span)
                        && server
                            .database()
                            .get_ref(&parent.span.file_id)
                            .is_ok_and(|file| !file.file_type.is_builtin())
                });
            let span = class.name_span.unwrap_or(class.span);
            let key = (!declared, order(server, span)?);
            if generated.is_none_or(|(other, _)| key < other) {
                generated = Some((key, span));
            }
        }
        if let Some((_, span)) = generated {
            return Err(RenameRefusal::at(RefusalReason::EnumMethod, span.file_id, range(span)));
        }
        let classes =
            targets.iter().filter_map(|(class, target)| family.contains(&target.span).then_some(*class)).collect();
        Ok(Self { declarations: family, classes, kind: selected.kind, targets })
    }

    pub(crate) fn check(&self, server: &Server, replacement: Option<Word>) -> Result<(), RenameRefusal> {
        let codebase = server.codebase();
        let mut classes = Vec::with_capacity(self.classes.len());
        for class in &self.classes {
            let metadata = codebase.get_class_like_by_name(*class).ok_or(STALE)?;
            classes.push((order(server, metadata.span)?, *class, metadata));
        }
        classes.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        for (_, class, metadata) in &classes {
            if let Some(refusal) = incomplete(server, metadata) {
                return Err(refusal);
            }
            let current = self.targets.get(class).ok_or(STALE)?;
            if !current.safe {
                return Err(RenameRefusal::at(RefusalReason::Magic, current.span.file_id, range(current.span)));
            }
        }
        let Some(name) = replacement else { return Ok(()) };
        for (_, class, _) in &classes {
            if let Some(taken) = target(codebase, *class, name, self.kind)
                && !self.declarations.contains(&taken.span)
            {
                return Err(RenameRefusal::at(RefusalReason::NameTaken, taken.span.file_id, range(taken.span)));
            }
        }
        Ok(())
    }

    /// Refuse at a declaration outside the project sources, preferring one that a project
    /// class in the family inherits, then the first by file name and offset.
    pub(crate) fn refuse_outside(
        &self,
        server: &Server,
        declarations: impl IntoIterator<Item = Span>,
        owner: impl Fn(Span) -> Option<Word>,
    ) -> Result<(), RenameRefusal> {
        let mut outside = Vec::new();
        for span in declarations {
            let key = order(server, span)?;
            if key.0 {
                outside.push((key, span));
            }
        }
        if outside.is_empty() {
            return Ok(());
        }
        let codebase = server.codebase();
        let mut hosts = Vec::new();
        for class in &self.classes {
            let metadata = codebase.get_class_like_by_name(*class).ok_or(STALE)?;
            if !order(server, metadata.span)?.0 {
                hosts.push(*class);
            }
        }
        let inherited = |span| {
            owner(span).is_some_and(|owner| {
                hosts.iter().any(|class| {
                    codebase.is_instance_of(class.as_bytes(), owner.as_bytes())
                        || codebase.class_uses_trait(class.as_bytes(), owner.as_bytes())
                })
            })
        };
        let blocker = outside
            .into_iter()
            .map(|(key, span)| ((!inherited(span), key.1, key.2), span))
            .min_by(|left, right| left.0.cmp(&right.0));
        match blocker {
            Some((_, span)) => {
                Err(declared_outside(server.database().get_ref(&span.file_id).map_err(|_| STALE)?, span))
            }
            None => Ok(()),
        }
    }

    /// The class whose own member is declared at `span`.
    pub(crate) fn owner(&self, codebase: &CodebaseMetadata, span: Span) -> Option<Word> {
        self.targets.iter().find_map(|(class, target)| {
            (target.span == span
                && codebase.get_class_like_by_name(*class).is_some_and(|metadata| {
                    metadata.span.file_id == span.file_id
                        && metadata.span.start.offset <= span.start.offset
                        && span.end.offset <= metadata.span.end.offset
                }))
            .then_some(*class)
        })
    }
}

fn order(server: &Server, span: Span) -> Result<(bool, &[u8], u32), RenameRefusal> {
    let file = server.database().get_ref(&span.file_id).map_err(|_| STALE)?;
    Ok((file.file_type != FileType::Host, file.name.as_ref(), span.start.offset))
}

pub(super) fn declaring_span(codebase: &CodebaseMetadata, class: Word, name: Word, kind: Kind) -> Option<Span> {
    target(codebase, class, name, kind).map(|target| target.span)
}

const fn range(span: Span) -> Range {
    Range::new(span.start.offset, span.end.offset)
}

fn incomplete(server: &Server, class: &ClassLikeMetadata) -> Option<RenameRefusal> {
    let direct = |dependency: &Word| {
        class.direct_parent_class == Some(*dependency) || class.direct_parent_interfaces.contains(dependency)
    };
    let fallback = class
        .incomplete_hierarchy_dependencies()
        .min_by(|left, right| (!direct(left), left.as_bytes()).cmp(&(!direct(right), right.as_bytes())))?;
    let span = class.name_span.unwrap_or(class.span);
    let spelled = server.database().get_ref(&class.span.file_id).ok().and_then(|file| {
        let start = class.name_span.map_or(class.span.start.offset, |name| name.end.offset) as usize;
        let rest = file.contents.get(start..class.span.end.offset as usize)?;
        let header = &rest[..rest.iter().position(|byte| *byte == b'{')?];
        header
            .split(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\\') || *byte >= 0x80))
            .find(|token| {
                class
                    .incomplete_hierarchy_dependencies()
                    .any(|dependency| direct(&dependency) && spells(token, dependency))
            })
            .map(word)
    });
    Some(RenameRefusal::at(RefusalReason::IncompleteHierarchy(spelled.unwrap_or(fallback)), span.file_id, range(span)))
}

fn spells(token: &[u8], dependency: Word) -> bool {
    let token = token.strip_prefix(b"\\").unwrap_or(token);
    let dependency = dependency.as_bytes();
    let Some(at) = dependency.len().checked_sub(token.len()) else { return false };
    !token.is_empty() && dependency[at..].eq_ignore_ascii_case(token) && (at == 0 || dependency[at - 1] == b'\\')
}

fn target(codebase: &CodebaseMetadata, class: Word, name: Word, kind: Kind) -> Option<Target> {
    match kind {
        Kind::Method => {
            let method = codebase.get_declaring_method(class.as_bytes(), name.as_bytes())?;
            let span = method.name_span?;
            Some(Target {
                span,
                private: method.method_metadata.as_ref()?.visibility.is_private(),
                safe: !method.flags.is_magic_method(),
                builtin: span == method.span,
            })
        }
        Kind::Property => {
            let property = codebase.get_declaring_property(class.as_bytes(), name.as_bytes())?;
            Some(Target {
                span: property.name_span?,
                private: property.read_visibility.is_private(),
                safe: !property.flags.is_virtual_property(),
                builtin: false,
            })
        }
        Kind::Constant => {
            if let Some(case) = codebase.get_enum_case(class.as_bytes(), name.as_bytes()) {
                return Some(Target { span: case.name_span, private: false, safe: true, builtin: false });
            }
            let constant = codebase.get_class_constant(class.as_bytes(), name.as_bytes())?;
            Some(Target {
                span: constant.span.subspan(0, constant.name.as_bytes().len() as u32),
                private: constant.visibility.is_private(),
                safe: true,
                builtin: false,
            })
        }
    }
}
