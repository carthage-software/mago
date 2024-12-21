#![allow(dead_code)]

use std::collections::HashMap;

use ahash::RandomState;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_ast::ast::*;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum NameKind {
    Default,
    Function,
    Constant,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct NameResolutionContext {
    namespace_name: String,
    default_aliases: HashMap<String, String, RandomState>,
    function_aliases: HashMap<String, String, RandomState>,
    constant_aliases: HashMap<String, String, RandomState>,
}

#[derive(Debug)]
pub struct NameContext<'a> {
    pub interner: &'a ThreadedInterner,
    name_resolution_contexts: Vec<NameResolutionContext>,
    namespace_name: Option<String>,
}

impl<'a> NameContext<'a> {
    pub fn new(interner: &'a ThreadedInterner) -> Self {
        NameContext {
            interner,
            name_resolution_contexts: vec![NameResolutionContext::default()],
            namespace_name: Default::default(),
        }
    }

    pub fn get_namespace_name(&self) -> Option<String> {
        self.namespace_name.clone()
    }

    pub fn get_namespaced_identifier(&mut self, identifier: &LocalIdentifier) -> StringIdentifier {
        if let Some(mut namespaced) = self.get_namespace_name() {
            if namespaced.is_empty() {
                return identifier.value;
            }

            namespaced.push('\\');
            namespaced.push_str(self.interner.lookup(&identifier.value));

            self.interner.intern(namespaced)
        } else {
            identifier.value
        }
    }

    pub fn enter_namespace(&mut self, namespace: StringIdentifier) {
        let previous_context =
            self.name_resolution_contexts.last().expect("expected there to be at least one name resolution context");

        let namespace_name = self.interner.lookup(&namespace);
        self.name_resolution_contexts.push(NameResolutionContext {
            namespace_name: namespace_name.to_owned(),
            default_aliases: previous_context.default_aliases.clone(),
            function_aliases: previous_context.function_aliases.clone(),
            constant_aliases: previous_context.constant_aliases.clone(),
        });

        self.namespace_name = Some(if let Some(mut previous_namespace) = self.namespace_name.clone() {
            if !previous_namespace.is_empty() {
                previous_namespace.push('\\');
                previous_namespace.push_str(namespace_name);

                previous_namespace
            } else {
                namespace_name.to_owned()
            }
        } else {
            namespace_name.to_owned()
        });
    }

    pub fn exit_namespace(&mut self) {
        if self.name_resolution_contexts.len() <= 1 {
            return;
        }

        self.name_resolution_contexts.pop();
        self.namespace_name =
            self.name_resolution_contexts.last().map(|last_context| last_context.namespace_name.clone());
    }

    pub fn add_name(&mut self, kind: NameKind, name_id: StringIdentifier, alias_id: Option<StringIdentifier>) {
        let name = self.interner.lookup(&name_id);

        let alias = match alias_id {
            Some(alias_id) => self.interner.lookup(&alias_id).to_ascii_lowercase(),
            None => {
                if let Some(last_backslash_pos) = name.rfind('\\') {
                    name[last_backslash_pos + 1..].to_ascii_lowercase()
                } else {
                    name.to_ascii_lowercase()
                }
            }
        };

        let context = self
            .name_resolution_contexts
            .last_mut()
            .expect("expected there to be at least one resolution context in the context");

        match kind {
            NameKind::Default => context.default_aliases.insert(alias, name.to_owned()),
            NameKind::Function => context.function_aliases.insert(alias, name.to_owned()),
            NameKind::Constant => context.constant_aliases.insert(alias, name.to_owned()),
        };
    }

    pub fn resolve_name(&mut self, kind: NameKind, name_id: StringIdentifier) -> (StringIdentifier, bool) {
        let name = self.interner.lookup(&name_id);

        if let Some(stripped) = name.strip_prefix('\\') {
            return (self.interner.intern(stripped), true);
        }

        if let Some(alias) = self.resolve_alias(kind, name) {
            return (alias, true);
        }

        match self.get_namespace_name() {
            Some(namespace_name) => {
                if namespace_name.is_empty() {
                    return (name_id, false);
                }

                let mut resolved = namespace_name.clone();
                resolved.push('\\');
                resolved.push_str(name);

                (self.interner.intern(resolved), false)
            }
            None => (name_id, false),
        }
    }

    fn resolve_alias(&mut self, kind: NameKind, name: &str) -> Option<StringIdentifier> {
        let context = self
            .name_resolution_contexts
            .last()
            .expect("expected there to be at least one resolution context in the context");

        let parts = name.split('\\').collect::<Vec<_>>();
        let first_part = parts.first().expect("expected there to be at least one part in the name");
        let first_part_lower = first_part.to_ascii_lowercase();

        if parts.len() > 1 {
            let suffix = parts[1..].join("\\");

            let alias = if first_part_lower == "namespace" {
                if let Some(namespace_name) = &self.namespace_name {
                    let mut resolved = namespace_name.clone();
                    resolved.push('\\');
                    resolved.push_str(&suffix);

                    return Some(self.interner.intern(resolved));
                }

                return Some(self.interner.intern(suffix));
            } else {
                context.default_aliases.get(first_part_lower.as_str())
            };

            if let Some(alias) = alias {
                let mut resolved = alias.clone();
                resolved.push('\\');
                resolved.push_str(&suffix);

                return Some(self.interner.intern(resolved));
            }
        } else {
            let alias = match kind {
                NameKind::Default => context.default_aliases.get(first_part_lower.as_str()),
                NameKind::Function => context.function_aliases.get(first_part_lower.as_str()),
                NameKind::Constant => context.constant_aliases.get(first_part_lower.as_str()),
            };

            if let Some(resolved) = alias {
                return Some(self.interner.intern(resolved));
            }
        }

        None
    }
}
