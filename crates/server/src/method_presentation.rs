//! A method's exposed name and modifiers, separate from its source declaration.

use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::visibility::Visibility;
use mago_database::DatabaseReader;
use mago_word::Word;
use mago_word::ascii_lowercase_word;
use mago_word::word;

use crate::Server;
use crate::semantic_index::SelectorKind;

#[derive(Clone, Copy)]
pub(crate) struct MethodPresentation {
    pub declaring: MethodIdentifier,
    pub owner: Word,
    pub name: Word,
    pub visibility: Visibility,
    pub is_final: bool,
    alias_source: Option<Word>,
}

impl MethodPresentation {
    pub fn resolve(codebase: &CodebaseMetadata, class: Word, name: &[u8]) -> Option<Self> {
        let class = codebase.get_class_like_by_name(class)?;
        let name = ascii_lowercase_word(name);
        let declaring = codebase.get_declaring_method_identifier(&MethodIdentifier::new(class.name, name));
        let metadata = codebase.get_method_by_id(&declaring)?;
        let method = metadata.method_metadata.as_ref()?;
        let appearing = class.appearing_method_ids.get(&name).copied().unwrap_or(declaring);
        let owner = codebase.get_class_like_by_name(appearing.get_class_name())?;
        let mut presentation = Self {
            declaring,
            owner: owner.original_name,
            name: metadata.original_name,
            visibility: method.visibility,
            is_final: method.is_final,
            alias_source: None,
        };
        // Ordinary methods need no source lookup or ancestry walk.
        if declaring.get_class_name() == owner.name
            || !codebase.get_class_like_by_name(declaring.get_class_name())?.kind.is_trait()
        {
            return Some(presentation);
        }
        if name != metadata.name {
            presentation.name = name;
        }
        presentation.alias_source =
            adaptation_owner(codebase, owner, declaring, name, |class| class.trait_alias_map.contains_key(&name))
                .map(|class| class.name);
        presentation.visibility = codebase
            .get_method_visibility_by_id(&MethodIdentifier::new(class.name, name))
            .unwrap_or(presentation.visibility);
        presentation.is_final = codebase.method_is_final(class.name.as_bytes(), name.as_bytes());
        Some(presentation)
    }

    /// Read spelling only for an alias, using the existing per-file semantic cache.
    pub fn restore_alias_case(&mut self, server: &mut Server) {
        let Some(source) = self.alias_source.and_then(|name| server.codebase().get_class_like_by_name(name)) else {
            return;
        };
        let span = source.span;
        let Some(analysis) = server.file_analysis_for(span.file_id) else { return };
        let Ok(file) = server.database().get(&span.file_id) else { return };
        if let Some(name) = analysis.semantic().member_selectors.iter().find_map(|selector| {
            if selector.kind != SelectorKind::TraitAlias || selector.owner != Some(span.start.offset) {
                return None;
            }
            file.contents
                .get(selector.range.start as usize..selector.range.end as usize)
                .filter(|name| name.eq_ignore_ascii_case(self.name.as_bytes()))
        }) {
            self.name = word(name);
        }
    }
}

/// An outer trait adaptation takes precedence over one in a nested trait.
/// Competing sources are left unresolved instead of choosing a hash-map order.
fn adaptation_owner<'a>(
    codebase: &'a CodebaseMetadata,
    owner: &'a ClassLikeMetadata,
    declaring: MethodIdentifier,
    name: Word,
    applies: impl Fn(&ClassLikeMetadata) -> bool,
) -> Option<&'a ClassLikeMetadata> {
    if applies(owner) {
        return Some(owner);
    }
    let candidates: Vec<_> = owner
        .used_traits
        .iter()
        .filter_map(|name| codebase.get_class_like_by_name(*name))
        .filter(|class| class.declaring_method_ids.get(&name) == Some(&declaring) && applies(class))
        .collect();
    let mut outer =
        candidates.iter().filter(|class| !candidates.iter().any(|other| other.used_traits.contains(&class.name)));
    let first = *outer.next()?;
    outer.next().is_none().then_some(first)
}

impl Server {
    pub(crate) fn method_presentation(&mut self, class: Word, name: &[u8]) -> Option<MethodPresentation> {
        let mut presentation = MethodPresentation::resolve(self.codebase(), class, name)?;
        presentation.restore_alias_case(self);
        Some(presentation)
    }
}
