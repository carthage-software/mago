//! [`ResolvedPattern`] implementation for Mago.
//!
//! Resolved patterns are the "values" the grit engine passes between pattern nodes during
//! matching. This is the biggest trait in the ecosystem (~45 methods). For the first cut
//! we implement the matching-essential paths and stub the rewriter half with
//! [`unimplemented!`]; next-session work ports the rest from marzano-core.

#![allow(clippy::unnecessary_unwrap)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashMap;

use grit_pattern_matcher::binding::Binding;
use grit_pattern_matcher::constant::Constant;
use grit_pattern_matcher::context::ExecContext;
use grit_pattern_matcher::effects::Effect;
use grit_pattern_matcher::pattern::Accessor;
use grit_pattern_matcher::pattern::DynamicPattern;
use grit_pattern_matcher::pattern::FilePtr;
use grit_pattern_matcher::pattern::FileRegistry;
use grit_pattern_matcher::pattern::ListIndex;
use grit_pattern_matcher::pattern::Pattern;
use grit_pattern_matcher::pattern::ResolvedPattern;
use grit_pattern_matcher::pattern::ResolvedSnippet;
use grit_pattern_matcher::pattern::State;
use grit_util::AnalysisLogs;
use grit_util::CodeRange;
use grit_util::Range;
use grit_util::error::GritPatternError;
use grit_util::error::GritResult;

use crate::binding::MagoBinding;
use crate::file::MagoFile;
use crate::query_context::MagoQueryContext;

/// Resolved pattern variants. Mirrors the sparse union marzano uses, reduced to the
/// matching-essential kinds. Rewriting will add more.
#[derive(Debug, Clone)]
pub enum MagoResolvedPattern<'a> {
    Binding(Vec<MagoBinding<'a>>),
    Constant(Constant),
    File(MagoFile<'a>),
    Files(Box<MagoResolvedPattern<'a>>),
    List(Vec<MagoResolvedPattern<'a>>),
    Map(BTreeMap<String, MagoResolvedPattern<'a>>),
    Snippets(Vec<ResolvedSnippet<'a, MagoQueryContext>>),
}

impl<'a> PartialEq for MagoResolvedPattern<'a> {
    fn eq(&self, other: &Self) -> bool {
        use MagoResolvedPattern::*;
        match (self, other) {
            (Binding(a), Binding(b)) => a == b,
            (Constant(a), Constant(b)) => a == b,
            (File(a), File(b)) => a == b,
            (Files(a), Files(b)) => a == b,
            (List(a), List(b)) => a == b,
            (Map(a), Map(b)) => a == b,
            (Snippets(_), Snippets(_)) => false,
            _ => false,
        }
    }
}

impl<'a> MagoResolvedPattern<'a> {
    pub fn from_bindings(bindings: Vec<MagoBinding<'a>>) -> Self {
        Self::Binding(bindings)
    }
}

impl<'a> ResolvedPattern<'a, MagoQueryContext> for MagoResolvedPattern<'a> {
    fn from_binding(binding: MagoBinding<'a>) -> Self {
        Self::Binding(vec![binding])
    }

    fn from_constant(constant: Constant) -> Self {
        Self::Constant(constant)
    }

    fn from_file_pointer(_file: FilePtr) -> Self {
        unimplemented!("file-pointer resolved patterns require file registry integration (deferred)")
    }

    fn from_files(files: Self) -> Self {
        Self::Files(Box::new(files))
    }

    fn from_list_parts(parts: impl Iterator<Item = Self>) -> Self {
        Self::List(parts.collect())
    }

    fn from_string(string: String) -> Self {
        Self::Snippets(vec![ResolvedSnippet::Text(Cow::Owned(string))])
    }

    fn from_resolved_snippet(snippet: ResolvedSnippet<'a, MagoQueryContext>) -> Self {
        Self::Snippets(vec![snippet])
    }

    fn from_dynamic_snippet(
        snippet: &'a grit_pattern_matcher::pattern::DynamicSnippet,
        state: &mut State<'a, MagoQueryContext>,
        context: &'a <MagoQueryContext as grit_pattern_matcher::context::QueryContext>::ExecContext<'a>,
        logs: &mut AnalysisLogs,
    ) -> GritResult<Self> {
        use grit_pattern_matcher::pattern::DynamicSnippetPart;

        let _ = logs;
        let _ = context;

        let mut parts: Vec<ResolvedSnippet<'a, MagoQueryContext>> = Vec::new();
        for part in &snippet.parts {
            match part {
                DynamicSnippetPart::String(s) => {
                    parts.push(ResolvedSnippet::Text(Cow::Owned(s.clone())));
                }
                DynamicSnippetPart::Variable(var) => {
                    let scope = var.try_scope()? as usize;
                    let index = var.try_index()? as usize;
                    let slot = state
                        .bindings
                        .get(scope)
                        .and_then(|stack| stack.last())
                        .and_then(|frame| frame.get(index))
                        .ok_or_else(|| GritPatternError::new("variable slot out of range"))?;
                    let value =
                        slot.value.as_ref().ok_or_else(|| GritPatternError::new("variable has no bound value"))?;
                    if let Some(binding) = value.get_last_binding() {
                        parts.push(ResolvedSnippet::Binding(binding.clone()));
                    } else {
                        let rendered = value.text(&state.files, context.language())?;
                        parts.push(ResolvedSnippet::Text(Cow::Owned(rendered.into_owned())));
                    }
                }
            }
        }
        Ok(Self::Snippets(parts))
    }

    fn from_dynamic_pattern(
        pattern: &'a DynamicPattern<MagoQueryContext>,
        state: &mut State<'a, MagoQueryContext>,
        context: &'a <MagoQueryContext as grit_pattern_matcher::context::QueryContext>::ExecContext<'a>,
        logs: &mut AnalysisLogs,
    ) -> GritResult<Self> {
        match pattern {
            DynamicPattern::Snippet(snippet) => Self::from_dynamic_snippet(snippet, state, context, logs),
            DynamicPattern::Variable(var) => {
                let scope = var.try_scope()? as usize;
                let index = var.try_index()? as usize;
                let slot = state
                    .bindings
                    .get(scope)
                    .and_then(|stack| stack.last())
                    .and_then(|frame| frame.get(index))
                    .ok_or_else(|| GritPatternError::new("variable slot out of range"))?;
                Ok(slot.value.clone().unwrap_or_else(Self::undefined))
            }
            _ => Err(GritPatternError::new("unsupported dynamic pattern variant")),
        }
    }

    fn from_accessor(
        _accessor: &'a Accessor<MagoQueryContext>,
        _state: &mut State<'a, MagoQueryContext>,
        _context: &'a <MagoQueryContext as grit_pattern_matcher::context::QueryContext>::ExecContext<'a>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<Self> {
        Err(GritPatternError::new("accessor patterns are not yet supported for PHP patterns"))
    }

    fn from_list_index(
        _index: &'a ListIndex<MagoQueryContext>,
        _state: &mut State<'a, MagoQueryContext>,
        _context: &'a <MagoQueryContext as grit_pattern_matcher::context::QueryContext>::ExecContext<'a>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<Self> {
        Err(GritPatternError::new("list-index patterns are not yet supported for PHP patterns"))
    }

    fn from_pattern(
        pattern: &'a Pattern<MagoQueryContext>,
        state: &mut State<'a, MagoQueryContext>,
        context: &'a <MagoQueryContext as grit_pattern_matcher::context::QueryContext>::ExecContext<'a>,
        logs: &mut AnalysisLogs,
    ) -> GritResult<Self> {
        match pattern {
            Pattern::StringConstant(s) => Ok(Self::from_string(s.text.clone())),
            Pattern::IntConstant(i) => Ok(Self::Constant(Constant::Integer(i.value))),
            Pattern::FloatConstant(f) => Ok(Self::Constant(Constant::Float(f.value))),
            Pattern::BooleanConstant(b) => Ok(Self::Constant(Constant::Boolean(b.value))),
            Pattern::Undefined => Ok(Self::undefined()),
            _ => {
                let _ = logs;
                let _ = context;
                let _ = state;
                Err(GritPatternError::new("pattern kind cannot be resolved to a value yet"))
            }
        }
    }

    fn extend(
        &mut self,
        with: Self,
        _effects: &mut Vec<Effect<'a, MagoQueryContext>>,
        _language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
    ) -> GritResult<()> {
        match (self, with) {
            (Self::Snippets(ours), Self::Snippets(theirs)) => {
                ours.extend(theirs);
                Ok(())
            }
            (Self::Snippets(ours), Self::Binding(bindings)) => {
                for binding in bindings {
                    ours.push(ResolvedSnippet::Binding(binding));
                }
                Ok(())
            }
            (target, other) => {
                *target = match (std::mem::replace(target, Self::Constant(Constant::Undefined)), other) {
                    (a, b) => {
                        let _ = a;
                        b
                    }
                };
                Ok(())
            }
        }
    }

    fn float(
        &self,
        _state: &FileRegistry<'a, MagoQueryContext>,
        _language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
    ) -> GritResult<f64> {
        match self {
            Self::Constant(Constant::Float(f)) => Ok(*f),
            Self::Constant(Constant::Integer(i)) => Ok(*i as f64),
            _ => Err(GritPatternError::new("resolved pattern is not a numeric value")),
        }
    }

    fn get_bindings(&self) -> Option<impl Iterator<Item = MagoBinding<'a>>> {
        match self {
            Self::Binding(bindings) => Some(bindings.clone().into_iter()),
            _ => None,
        }
    }

    fn get_file(&self) -> Option<&MagoFile<'a>> {
        match self {
            Self::File(file) => Some(file),
            _ => None,
        }
    }

    fn get_file_pointers(&self) -> Option<Vec<FilePtr>> {
        None
    }

    fn get_files(&self) -> Option<&Self> {
        match self {
            Self::Files(inner) => Some(inner.as_ref()),
            _ => None,
        }
    }

    fn get_last_binding(&self) -> Option<&MagoBinding<'a>> {
        match self {
            Self::Binding(bindings) => bindings.last(),
            _ => None,
        }
    }

    fn get_list_item_at(&self, index: isize) -> Option<&Self> {
        match self {
            Self::List(items) => {
                let idx = if index < 0 { items.len().checked_sub((-index) as usize)? } else { index as usize };
                items.get(idx)
            }
            _ => None,
        }
    }

    fn get_list_item_at_mut(&mut self, index: isize) -> Option<&mut Self> {
        match self {
            Self::List(items) => {
                let idx = if index < 0 { items.len().checked_sub((-index) as usize)? } else { index as usize };
                items.get_mut(idx)
            }
            _ => None,
        }
    }

    fn get_list_items(&self) -> Option<impl Iterator<Item = &Self>> {
        match self {
            Self::List(items) => Some(items.iter()),
            _ => None,
        }
    }

    fn get_list_binding_items(&self) -> Option<impl Iterator<Item = Self> + Clone> {
        None::<std::iter::Empty<Self>>
    }

    fn get_map(&self) -> Option<&BTreeMap<String, Self>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    fn get_map_mut(&mut self) -> Option<&mut BTreeMap<String, Self>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    fn get_snippets(&self) -> Option<impl Iterator<Item = ResolvedSnippet<'a, MagoQueryContext>>> {
        match self {
            Self::Snippets(s) => Some(s.clone().into_iter()),
            _ => None,
        }
    }

    fn is_binding(&self) -> bool {
        matches!(self, Self::Binding(_))
    }

    fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    fn is_truthy(
        &self,
        _state: &mut State<'a, MagoQueryContext>,
        language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
    ) -> GritResult<bool> {
        Ok(match self {
            Self::Binding(bindings) => bindings.last().map(|b| b.is_truthy()).unwrap_or(false),
            Self::Constant(c) => c.is_truthy(),
            Self::List(items) => !items.is_empty(),
            Self::Map(m) => !m.is_empty(),
            Self::File(_) | Self::Files(_) => true,
            Self::Snippets(s) => !s.is_empty(),
        })
        .map(|_| {
            let _ = language;

            match self {
                Self::Binding(bindings) => bindings.last().map(|b| b.is_truthy()).unwrap_or(false),
                Self::Constant(c) => c.is_truthy(),
                Self::List(items) => !items.is_empty(),
                Self::Map(m) => !m.is_empty(),
                Self::File(_) | Self::Files(_) => true,
                Self::Snippets(s) => !s.is_empty(),
            }
        })
    }

    fn linearized_text(
        &self,
        language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
        _effects: &[Effect<'a, MagoQueryContext>],
        _files: &FileRegistry<'a, MagoQueryContext>,
        _memo: &mut HashMap<CodeRange, Option<String>>,
        _should_pad_snippet: bool,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<Cow<'a, str>> {
        match self {
            Self::Binding(bindings) => {
                if let Some(last) = bindings.last() {
                    last.text(language).map(|c| match c {
                        Cow::Borrowed(_) => Cow::Owned(c.into_owned()),
                        Cow::Owned(s) => Cow::Owned(s),
                    })
                } else {
                    Ok(Cow::Borrowed(""))
                }
            }
            Self::Constant(c) => Ok(Cow::Owned(c.to_string())),
            Self::Snippets(s) => {
                let mut out = String::new();
                for snip in s {
                    match snip {
                        ResolvedSnippet::Text(t) => out.push_str(t),
                        ResolvedSnippet::Binding(b) => out.push_str(&b.text(language)?),
                        ResolvedSnippet::LazyFn(_) => {
                            return Err(GritPatternError::new("lazy snippet functions are not supported"));
                        }
                    }
                }
                Ok(Cow::Owned(out))
            }
            _ => Err(GritPatternError::new("linearized_text not supported for this resolved pattern")),
        }
    }

    fn matches_undefined(&self) -> bool {
        matches!(self, Self::Constant(Constant::Undefined))
    }

    fn matches_false_or_undefined(&self) -> bool {
        matches!(self, Self::Constant(Constant::Undefined) | Self::Constant(Constant::Boolean(false)))
    }

    fn normalize_insert(
        &mut self,
        _binding: &MagoBinding<'a>,
        _is_first: bool,
        _language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
    ) -> GritResult<()> {
        Ok(())
    }

    fn position(
        &self,
        _language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
    ) -> Option<Range> {
        None
    }

    fn push_binding(&mut self, binding: MagoBinding<'a>) -> GritResult<()> {
        match self {
            Self::Binding(bindings) => {
                bindings.push(binding);
                Ok(())
            }
            _ => Err(GritPatternError::new("cannot push binding onto a non-binding resolved pattern")),
        }
    }

    fn set_list_item_at_mut(&mut self, index: isize, value: Self) -> GritResult<bool> {
        if let Self::List(items) = self {
            let idx = if index < 0 {
                items
                    .len()
                    .checked_sub((-index) as usize)
                    .ok_or_else(|| GritPatternError::new("negative list index out of range"))?
            } else {
                index as usize
            };
            if idx >= items.len() {
                return Ok(false);
            }
            items[idx] = value;
            Ok(true)
        } else {
            Err(GritPatternError::new("cannot set list item on non-list resolved pattern"))
        }
    }

    fn text(
        &self,
        files: &FileRegistry<'a, MagoQueryContext>,
        language: &<MagoQueryContext as grit_pattern_matcher::context::QueryContext>::Language<'a>,
    ) -> GritResult<Cow<'a, str>> {
        match self {
            Self::Binding(bindings) => {
                if let Some(last) = bindings.last() {
                    last.text(language).map(|c| match c {
                        Cow::Borrowed(_) => Cow::Owned(c.into_owned()),
                        Cow::Owned(s) => Cow::Owned(s),
                    })
                } else {
                    Ok(Cow::Borrowed(""))
                }
            }
            Self::Constant(c) => Ok(Cow::Owned(c.to_string())),
            Self::Snippets(s) => {
                let mut out = String::new();
                for snip in s {
                    out.push_str(&snip.text(files, language)?);
                }
                Ok(Cow::Owned(out))
            }
            _ => Ok(Cow::Borrowed("")),
        }
    }
}
