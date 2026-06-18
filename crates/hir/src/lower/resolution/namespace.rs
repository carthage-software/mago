use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::ir::statement::UseItemKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NameResolutionKind {
    Default,
    Function,
    Constant,
}

impl NameResolutionKind {
    #[inline]
    #[must_use]
    pub const fn is_case_sensitive(self) -> bool {
        matches!(self, NameResolutionKind::Constant)
    }
}

#[derive(Debug)]
pub struct NamespaceResolution<'scratch, S>
where
    S: Arena,
{
    scratch: &'scratch S,
    namespace: Option<&'scratch [u8]>,
    default_aliases: Vec<'scratch, (&'scratch [u8], &'scratch [u8]), S>,
    function_aliases: Vec<'scratch, (&'scratch [u8], &'scratch [u8]), S>,
    constant_aliases: Vec<'scratch, (&'scratch [u8], &'scratch [u8]), S>,
}

impl<'scratch, S> NamespaceResolution<'scratch, S>
where
    S: Arena,
{
    pub fn new_in(scratch: &'scratch S) -> NamespaceResolution<'scratch, S> {
        NamespaceResolution {
            scratch,
            namespace: None,
            default_aliases: Vec::new_in(scratch),
            function_aliases: Vec::new_in(scratch),
            constant_aliases: Vec::new_in(scratch),
        }
    }

    pub fn enter_namespace(&mut self, namespace: Option<&'scratch [u8]>) {
        self.namespace = namespace;
        self.default_aliases.clear();
        self.function_aliases.clear();
        self.constant_aliases.clear();
    }

    pub fn leave_namespace(&mut self) {
        self.namespace = None;
        self.default_aliases.clear();
        self.function_aliases.clear();
        self.constant_aliases.clear();
    }

    pub fn add_import(
        &mut self,
        kind: UseItemKind,
        fully_qualified_name: &'scratch [u8],
        alias: Option<&'scratch [u8]>,
    ) {
        let alias = alias.unwrap_or_else(|| match memchr::memrchr(b'\\', fully_qualified_name) {
            Some(position) => &fully_qualified_name[position + 1..],
            None => fully_qualified_name,
        });

        match kind {
            UseItemKind::Default => self.default_aliases.push((alias, fully_qualified_name)),
            UseItemKind::Function => self.function_aliases.push((alias, fully_qualified_name)),
            UseItemKind::Const => self.constant_aliases.push((alias, fully_qualified_name)),
        }
    }

    pub(crate) fn resolve_name(&self, kind: NameResolutionKind, name: &'scratch [u8]) -> &'scratch [u8] {
        if let [b'\\', rest @ ..] = name {
            return rest;
        }

        match memchr::memchr(b'\\', name) {
            Some(separator) => {
                let first = &name[..separator];
                let suffix = &name[separator + 1..];

                if first.eq_ignore_ascii_case(b"namespace") {
                    self.qualify(suffix)
                } else if let Some(resolved) = self.lookup_alias(NameResolutionKind::Default, first) {
                    self.concat(resolved, suffix)
                } else {
                    self.qualify(name)
                }
            }
            None => match self.lookup_alias(kind, name) {
                Some(resolved) => resolved,
                None => self.qualify(name),
            },
        }
    }

    #[must_use]
    pub fn lookup_alias(&self, kind: NameResolutionKind, query: &[u8]) -> Option<&'scratch [u8]> {
        let table = match kind {
            NameResolutionKind::Default => &self.default_aliases,
            NameResolutionKind::Function => &self.function_aliases,
            NameResolutionKind::Constant => &self.constant_aliases,
        };

        table.iter().find_map(|&(alias, fully_qualified_name)| {
            let matches = if kind.is_case_sensitive() { alias == query } else { alias.eq_ignore_ascii_case(query) };

            if matches { Some(fully_qualified_name) } else { None }
        })
    }

    #[must_use]
    pub fn qualify(&self, name: &'scratch [u8]) -> &'scratch [u8] {
        match self.namespace {
            Some(namespace) if !namespace.is_empty() => self.concat(namespace, name),
            _ => name,
        }
    }

    fn concat(&self, prefix: &[u8], suffix: &[u8]) -> &'scratch [u8] {
        let mut buffer = Vec::with_capacity_in(prefix.len() + 1 + suffix.len(), self.scratch);
        buffer.extend_from_slice(prefix);
        buffer.push(b'\\');
        buffer.extend_from_slice(suffix);

        buffer.leak()
    }
}
