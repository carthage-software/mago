//! Member candidates from retained receiver types and PHP visibility rules.

use std::collections::hash_map::Entry;

use foldhash::HashMap;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::class_like_constant::ClassLikeConstantMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::ttype::display::qualified_bytes;
use mago_codex::visibility::Visibility;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use mago_span::Span;
use mago_word::Word;

use crate::Server;
use crate::domain::CompletionAlternative;
use crate::domain::CompletionDocumentation;
use crate::domain::CompletionEntry;
use crate::domain::CompletionInsert;
use crate::domain::CompletionKind;
use crate::domain::HoverEntry;
use crate::matcher;
use crate::matcher::Match;
use crate::method_presentation::MethodPresentation;

use super::items::finalize;
use super::items::render_signature;
use super::rank::Rank;

const CASE: u8 = 0;
const CONSTANT: u8 = 1;
const MEMBER: u8 = 2;
const MAGIC: u8 = 4;

struct MemberCandidate {
    rank: Rank,
    owner: Word,
    entry: CompletionEntry,
}

impl MemberCandidate {
    fn alternative(&self) -> CompletionAlternative {
        let declaration = if self.entry.kind == CompletionKind::Method {
            format!("function {}", self.entry.detail.as_deref().unwrap_or(&self.entry.label))
        } else {
            let name = self.entry.label.trim_start_matches('$');
            self.entry.detail.as_ref().map_or_else(|| format!("${name}"), |ty| format!("{ty} ${name}"))
        };
        CompletionAlternative {
            description: HoverEntry::in_class(self.owner.as_bytes(), declaration),
            documentation_source: self.entry.documentation_source,
        }
    }

    fn merge(&mut self, other: Self) {
        let mut alternatives =
            self.entry.member_alternatives.take().map_or_else(|| vec![self.alternative()], Vec::from);
        alternatives.push(other.alternative());
        if other.rank < self.rank {
            *self = other;
        }
        self.entry.member_alternatives = Some(alternatives.into_boxed_slice());
    }
}

pub(super) fn instance_member_items(
    server: &mut Server,
    classes: &[Word],
    prefix: &[u8],
    static_access: bool,
    access_class: Option<Word>,
) -> Vec<CompletionEntry> {
    let mut best: HashMap<(u8, String), MemberCandidate> = HashMap::default();
    for class in classes {
        for candidate in collect_class_members(server, *class, prefix, static_access, false, access_class) {
            let entry = &candidate.entry;
            let key = if entry.kind == CompletionKind::Method {
                entry.label.to_ascii_lowercase()
            } else {
                entry.label.clone()
            };
            match best.entry((entry.kind as u8, key)) {
                Entry::Occupied(mut e) => {
                    e.get_mut().merge(candidate);
                }
                Entry::Vacant(e) => {
                    e.insert(candidate);
                }
            }
        }
    }

    finalize(best.into_values().map(|candidate| (candidate.rank, candidate.entry)).collect())
}

pub(super) fn static_member_items(
    server: &mut Server,
    scope: &NamespaceScope,
    class: &[u8],
    prefix: &[u8],
    access_class: Option<Word>,
    binds_this: bool,
) -> Vec<CompletionEntry> {
    let codebase = server.codebase();
    let access_class = access_class.and_then(|name| codebase.get_class_like_by_name(name));
    // The receiver is a name written in source; the scope resolves it to its
    // FQCN exactly as PHP would (aliases, namespace relativity, leading `\`).
    let (fqcn, _) = scope.resolve_str(NameKind::Default, class);
    let meta = if class.eq_ignore_ascii_case(b"self") || class.eq_ignore_ascii_case(b"static") {
        access_class
    } else if class.eq_ignore_ascii_case(b"parent") {
        access_class.and_then(|class| class.direct_parent_class).and_then(|name| codebase.get_class_like_by_name(name))
    } else {
        codebase.get_class_like(&fqcn)
    };
    let Some(meta) = meta else {
        return Vec::new();
    };

    let instance_calls = binds_this
        && access_class.is_some_and(|access| {
            access.name == meta.name || codebase.class_extends(access.name.as_bytes(), meta.name.as_bytes())
        });
    let parent = class.eq_ignore_ascii_case(b"parent");
    let class = meta.name;
    let access_class = access_class.map(|class| class.name);
    finalize(
        collect_class_members(server, class, prefix, true, instance_calls, access_class)
            .into_iter()
            .map(|mut candidate| {
                if parent {
                    candidate.rank.group %= MAGIC;
                }
                (candidate.rank, candidate.entry)
            })
            .collect(),
    )
}

fn collect_class_members(
    server: &mut Server,
    class: Word,
    prefix: &[u8],
    static_access: bool,
    instance_calls: bool,
    access_class: Option<Word>,
) -> Vec<MemberCandidate> {
    let codebase = server.codebase();
    let version = server.php_version;
    let Some(meta) = codebase.get_class_like_by_name(class) else { return Vec::new() };
    let access_class = access_class.and_then(|name| codebase.get_class_like_by_name(name));
    if !meta.is_available_in_version(version) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let demote_magic = !mago_bytes::trim_start_byte(prefix, b'$').starts_with(b"_");
    let rank = |matched: Match, name: &[u8], kind: u8, inherited: bool, deprecated: bool| Rank {
        group: if demote_magic && name.starts_with(b"__") { MAGIC } else { 0 } + if static_access { kind } else { 0 },
        discouraged: deprecated,
        proximity: u16::from(inherited),
        ..Rank::new(matched)
    };

    if static_access && !prefix.starts_with(b"$") {
        for (name, constant) in &meta.constants {
            let owner = constant_owner(codebase, meta, constant);
            if !constant.is_available_in_version(version)
                || !visible(codebase, constant.visibility, owner, access_class)
            {
                continue;
            }
            let s = name.as_bytes();
            let Some(matched) = matcher::score(prefix, s) else {
                continue;
            };

            let deprecated = constant.flags.is_deprecated();
            out.push(MemberCandidate {
                rank: rank(matched, s, CONSTANT, !declares(meta, constant.span), deprecated),
                owner: owner.original_name,
                entry: CompletionEntry {
                    documentation_source: Some(CompletionDocumentation::DeclarationName(constant.span)),
                    deprecated,
                    ..CompletionEntry::new(String::from_utf8_lossy(s).into_owned(), CompletionKind::Constant)
                },
            });
        }

        for (name, case) in &meta.enum_cases {
            if !case.is_available_in_version(version) {
                continue;
            }
            let s = name.as_bytes();
            let Some(matched) = matcher::score(prefix, s) else {
                continue;
            };

            let deprecated = case.flags.is_deprecated();
            out.push(MemberCandidate {
                rank: rank(matched, s, CASE, false, deprecated),
                owner: meta.original_name,
                entry: CompletionEntry {
                    documentation_source: Some(CompletionDocumentation::Declaration(case.span)),
                    deprecated,
                    ..CompletionEntry::new(String::from_utf8_lossy(s).into_owned(), CompletionKind::EnumMember)
                },
            });
        }
    }

    for (name, declaring_class) in meta.appearing_property_ids.iter() {
        let name_bytes = mago_bytes::trim_start_byte(name.as_bytes(), b'$');
        let Some(matched) = matcher::score(mago_bytes::trim_start_byte(prefix, b'$'), name_bytes) else {
            continue;
        };

        let Some(declaring) = codebase.get_class_like(declaring_class.as_bytes()) else { continue };
        let Some(property) = codebase.get_declaring_property(meta.name.as_bytes(), name.as_bytes()) else { continue };
        let owner = property_owner(codebase, meta, declaring, *name, property);
        if !declaring.is_available_in_version(version)
            || !property.is_available_in_version(version)
            || property.flags.is_static() != static_access
            || !visible(codebase, property.read_visibility, owner, access_class)
        {
            continue;
        }

        let detail = property
            .type_metadata
            .as_ref()
            .map(|t| String::from_utf8_lossy(&qualified_bytes(codebase, &t.type_union)).into_owned());
        let label = if static_access {
            format!("${}", String::from_utf8_lossy(name_bytes))
        } else {
            String::from_utf8_lossy(name_bytes).into_owned()
        };
        let deprecated = property.flags.is_deprecated();
        out.push(MemberCandidate {
            rank: rank(matched, name_bytes, MEMBER, *declaring_class != meta.name, deprecated),
            owner: owner.original_name,
            entry: CompletionEntry {
                detail,
                documentation_source: Some(CompletionDocumentation::Property { class: meta.name, name: *name }),
                deprecated,
                ..CompletionEntry::new(label, CompletionKind::Field)
            },
        });
    }

    if !static_access {
        for (name, declaring_class) in &meta.magic_property_ids {
            let name_bytes = mago_bytes::trim_start_byte(name.as_bytes(), b'$');
            let Some(matched) = matcher::score(prefix, name_bytes) else { continue };
            // A visible native property governs this access before a magic annotation.
            if out.iter().any(|candidate| {
                candidate.entry.kind == CompletionKind::Field && candidate.entry.label.as_bytes() == name_bytes
            }) {
                continue;
            }
            let Some(declaring) = codebase.get_class_like_by_name(*declaring_class) else { continue };
            let Some(property) = codebase.get_declaring_magic_property(meta.name.as_bytes(), name.as_bytes()) else {
                continue;
            };
            if !declaring.is_available_in_version(version) || !property.is_available_in_version(version) {
                continue;
            }
            let deprecated = property.flags.is_deprecated();
            out.push(MemberCandidate {
                rank: rank(matched, name_bytes, MEMBER, declaring.name != meta.name, deprecated),
                owner: declaring.original_name,
                entry: CompletionEntry {
                    documentation_source: Some(CompletionDocumentation::MagicProperty {
                        class: declaring.name,
                        name: *name,
                    }),
                    detail: property
                        .type_metadata
                        .as_ref()
                        .map(|ty| String::from_utf8_lossy(&qualified_bytes(codebase, &ty.type_union)).into_owned()),
                    deprecated,
                    ..CompletionEntry::new(String::from_utf8_lossy(name_bytes).into_owned(), CompletionKind::Field)
                },
            });
        }
    }

    let mut methods = Vec::new();
    if !prefix.starts_with(b"$") {
        for name in meta.appearing_method_ids.keys() {
            if !matcher::may_match(prefix, name.as_bytes()) {
                continue;
            }

            let Some(presentation) = MethodPresentation::resolve(codebase, meta.name, name.as_bytes()) else {
                continue;
            };
            let Some(method) = codebase.get_method_by_id(&presentation.declaring) else { continue };
            let Some(method_meta) = method.method_metadata.as_ref() else { continue };
            let Some(owner) = codebase.get_class_like_by_name(presentation.owner) else { continue };
            let callable = if method_meta.is_static { static_access } else { !static_access || instance_calls };
            if !owner.is_available_in_version(version)
                || !method.is_available_in_version(version)
                || !callable
                || !visible(codebase, presentation.visibility, owner, access_class)
            {
                continue;
            }

            methods.push((presentation, owner.name != meta.name, method.flags.is_deprecated()));
        }
    }

    for (mut presentation, inherited, deprecated) in methods {
        presentation.restore_alias_case(server);
        let display = presentation.name.as_bytes();
        let Some(matched) = matcher::score(prefix, display) else { continue };
        let Some(method) = server.codebase().get_method_by_id(&presentation.declaring) else { continue };
        let ranked = rank(matched, display, MEMBER, inherited, deprecated);
        let display_str = String::from_utf8_lossy(display).into_owned();
        out.push(MemberCandidate {
            rank: ranked,
            owner: presentation.owner,
            entry: CompletionEntry {
                detail: Some(render_signature(server.database(), server.codebase(), method, display)),
                documentation_source: Some(CompletionDocumentation::Callable(presentation.declaring.into())),
                insert: Some(CompletionInsert::Call(display_str.clone())),
                deprecated: ranked.discouraged,
                ..CompletionEntry::new(display_str, CompletionKind::Method)
            },
        });
    }

    out
}

fn declares(class: &ClassLikeMetadata, span: Span) -> bool {
    span.file_id == class.span.file_id
        && class.span.start.offset <= span.start.offset
        && span.end.offset <= class.span.end.offset
}

fn constant_owner<'a>(
    codebase: &'a CodebaseMetadata,
    mut class: &'a ClassLikeMetadata,
    constant: &ClassLikeConstantMetadata,
) -> &'a ClassLikeMetadata {
    if !constant.visibility.is_protected() {
        return class;
    }
    for _ in 0..class.all_parent_classes.len() {
        if class.trait_constant_ids.contains_key(&constant.name) {
            break;
        }
        let Some(parent) = class.direct_parent_class.and_then(|name| codebase.get_class_like_by_name(name)) else {
            break;
        };
        if parent.constants.get(&constant.name).is_none_or(|parent_constant| parent_constant.span != constant.span) {
            break;
        }
        class = parent;
    }
    class
}

fn property_owner<'a>(
    codebase: &'a CodebaseMetadata,
    mut class: &'a ClassLikeMetadata,
    declaring: &'a ClassLikeMetadata,
    name: Word,
    property: &PropertyMetadata,
) -> &'a ClassLikeMetadata {
    if !declaring.kind.is_trait() || property.read_visibility.is_public() {
        return declaring;
    }
    let Some(span) = property.name_span.or(property.span) else { return declaring };
    // Trait properties belong to the class that imports them, not every user of that trait.
    for _ in 0..=class.all_parent_classes.len() {
        if class.properties.get(&name).is_some_and(|candidate| candidate.name_span.or(candidate.span) == Some(span)) {
            return class;
        }
        let Some(parent) = class.direct_parent_class.and_then(|name| codebase.get_class_like_by_name(name)) else {
            break;
        };
        class = parent;
    }
    declaring
}

pub(super) fn visible(
    codebase: &CodebaseMetadata,
    visibility: Visibility,
    declaring: &ClassLikeMetadata,
    access: Option<&ClassLikeMetadata>,
) -> bool {
    visibility.is_public()
        || access.is_some_and(|access| {
            access.name == declaring.name
                || visibility.is_protected()
                    && (codebase.class_extends(access.name.as_bytes(), declaring.name.as_bytes())
                        || codebase.class_extends(declaring.name.as_bytes(), access.name.as_bytes()))
        })
}
