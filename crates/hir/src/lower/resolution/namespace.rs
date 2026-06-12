use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_syntax::cst;

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

    pub fn populate_from_use(&mut self, r#use: &'scratch cst::Use<'scratch>) {
        match &r#use.items {
            cst::UseItems::Sequence(sequence) => {
                for item in sequence.items.iter() {
                    self.add_use_alias(NameResolutionKind::Default, item);
                }
            }
            cst::UseItems::TypedSequence(sequence) => {
                let kind = use_type_kind(&sequence.r#type);
                for item in sequence.items.iter() {
                    self.add_use_alias(kind, item);
                }
            }
            cst::UseItems::TypedList(list) => {
                let kind = use_type_kind(&list.r#type);
                let prefix = trim_leading_backslash(list.namespace.value());
                for item in list.items.iter() {
                    self.add_grouped_use_alias(kind, prefix, item);
                }
            }
            cst::UseItems::MixedList(list) => {
                let prefix = trim_leading_backslash(list.namespace.value());
                for item in list.items.iter() {
                    let kind = item.r#type.as_ref().map_or(NameResolutionKind::Default, use_type_kind);
                    self.add_grouped_use_alias(kind, prefix, &item.item);
                }
            }
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

    fn add_use_alias(&mut self, kind: NameResolutionKind, item: &'scratch cst::UseItem<'scratch>) {
        let name = trim_leading_backslash(item.name.value());
        let alias = match &item.alias {
            Some(alias) => alias.identifier.value,
            None => last_segment(name),
        };

        self.push_alias(kind, alias, name);
    }

    fn add_grouped_use_alias(
        &mut self,
        kind: NameResolutionKind,
        prefix: &'scratch [u8],
        item: &'scratch cst::UseItem<'scratch>,
    ) {
        let name_part = item.name.value();
        let fully_qualified_name = self.concat(prefix, name_part);
        let alias = match &item.alias {
            Some(alias) => alias.identifier.value,
            None => last_segment(name_part),
        };

        self.push_alias(kind, alias, fully_qualified_name);
    }

    fn push_alias(&mut self, kind: NameResolutionKind, alias: &'scratch [u8], fully_qualified_name: &'scratch [u8]) {
        match kind {
            NameResolutionKind::Default => self.default_aliases.push((alias, fully_qualified_name)),
            NameResolutionKind::Function => self.function_aliases.push((alias, fully_qualified_name)),
            NameResolutionKind::Constant => self.constant_aliases.push((alias, fully_qualified_name)),
        }
    }
}

fn use_type_kind(use_type: &cst::UseType<'_>) -> NameResolutionKind {
    match use_type {
        cst::UseType::Function(_) => NameResolutionKind::Function,
        cst::UseType::Const(_) => NameResolutionKind::Constant,
    }
}

fn trim_leading_backslash(name: &[u8]) -> &[u8] {
    if let [b'\\', rest @ ..] = name { rest } else { name }
}

fn last_segment(name: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\\', name) {
        Some(position) => &name[position + 1..],
        None => name,
    }
}
