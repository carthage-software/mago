use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::collections::HashSet;
use mago_allocator::vec::Vec;
use mago_allocator::vec_in;

use crate::document::Align;
use crate::document::BreakMode;
use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::document::Space;
use crate::document::Trim;
use crate::document::clone_in_arena;
use crate::document::group::GroupIdentifier;
use crate::internal::is_line_terminator_or_space;
use crate::internal::is_space;
use crate::internal::printer::command::Command;
use crate::internal::printer::command::CommandDocument;
use crate::internal::printer::command::Indentation;
use crate::internal::printer::command::Mode;
use crate::internal::printer::fits::FitsStack;
use crate::internal::utils::string_width;
use crate::settings::FormatSettings;

mod command;
mod fits;

const MAP_PREALLOCATION_THRESHOLD: usize = 4 * 1024;
const GROUP_MODE_CAPACITY_DIVISOR: usize = 64;
const PROPAGATED_BREAK_CAPACITY_DIVISOR: usize = 16;

#[derive(Debug)]
pub struct Printer<'arena, A>
where
    A: Arena,
{
    arena: &'arena A,
    settings: FormatSettings,
    out: Vec<'arena, u8, A>,
    position: usize,
    commands: Vec<'arena, Command<'arena, A>, A>,
    line_suffix: Vec<'arena, Command<'arena, A>, A>,
    group_mode_map: HashMap<'arena, GroupIdentifier, Mode, A>,
    propagated_breaks: HashSet<'arena, usize, A>,
    new_line: &'static str,
    can_trim: bool,
    range_start: Option<usize>,
    range_end: Option<usize>,
}

impl<'arena, A> Printer<'arena, A>
where
    A: Arena,
{
    pub fn new(
        arena: &'arena A,
        document: Document<'arena, A>,
        capacity_hint: usize,
        settings: FormatSettings,
    ) -> Self {
        // Preallocate for performance because the output will very likely
        // be the same size as the original text.
        let out = Vec::with_capacity_in(capacity_hint, arena);
        let cmds = vec_in![arena; Command::new(Indentation::root(), Mode::Break, arena.alloc(document))];
        let estimated_map_capacity = |divisor| {
            if capacity_hint < MAP_PREALLOCATION_THRESHOLD { 0 } else { capacity_hint / divisor }
        };

        Self {
            arena,
            settings,
            out,
            position: 0,
            commands: cmds,
            line_suffix: vec_in![arena],
            group_mode_map: HashMap::with_capacity_in(estimated_map_capacity(GROUP_MODE_CAPACITY_DIVISOR), arena),
            propagated_breaks: HashSet::with_capacity_in(
                estimated_map_capacity(PROPAGATED_BREAK_CAPACITY_DIVISOR),
                arena,
            ),
            new_line: settings.end_of_line.as_str(),
            can_trim: true,
            range_start: None,
            range_end: None,
        }
    }

    pub fn build(mut self) -> &'arena [u8] {
        self.print_doc_to_string();

        self.out.leak()
    }

    pub(crate) fn build_range(mut self) -> Option<(&'arena [u8], std::ops::Range<usize>)> {
        self.print_doc_to_string();
        Some((self.out.leak(), self.range_start?..self.range_end?))
    }

    /// Turn Doc into a string
    fn print_doc_to_string(&mut self) {
        let mut should_remeasure = false;
        while let Some(Command { indentation, document, mode }) = self.commands.pop() {
            match document {
                CommandDocument::Array(docs) => {
                    docs.iter().rev().any(|doc| Self::propagate_breaks(&mut self.propagated_breaks, doc));
                    self.push_docs(indentation, mode, docs);
                }
                CommandDocument::Document(document) => {
                    if matches!(
                        document,
                        Document::Group(_)
                            | Document::Array(_)
                            | Document::Indent(_)
                            | Document::Align(_)
                            | Document::IndentIfBreak(_)
                            | Document::IfBreak(_)
                    ) {
                        Self::propagate_breaks(&mut self.propagated_breaks, document);
                    }

                    match document {
                        Document::String(s) => self.handle_bytes(s),
                        Document::RangeStart => self.range_start = Some(self.out.len()),
                        Document::RangeEnd => {
                            if self.line_suffix.is_empty() {
                                self.range_end = Some(self.out.len());
                            } else {
                                self.line_suffix.push(Command::new(indentation, mode, document));
                            }
                        }
                        Document::Space(space) => self.handle_space(*space),
                        Document::Array(docs) => self.push_docs(indentation, mode, docs),
                        Document::Indent(docs) => self.push_docs(indentation.indented(), mode, docs),
                        Document::Align(align) => {
                            self.push_docs(Indentation::aligned(align.alignment), mode, &align.contents)
                        }
                        Document::Group(_) => {
                            should_remeasure = self.handle_group(&indentation, mode, document, should_remeasure);
                        }
                        Document::IndentIfBreak(docs) => self.handle_indent_if_break(&indentation, mode, docs),
                        Document::Line(line) => {
                            should_remeasure = self.handle_line(*line, &indentation, mode, document, should_remeasure);
                        }
                        Document::LineSuffix(docs) => {
                            self.line_suffix.push(Command::array(indentation, mode, docs));
                        }
                        Document::LineSuffixBoundary => {
                            should_remeasure = self.handle_line_suffix_boundary(indentation, mode, should_remeasure);
                        }
                        Document::IfBreak(if_break) => self.handle_if_break(if_break, indentation, mode),
                        Document::BreakParent => { /* No op */ }
                        Document::Trim(trim) => self.handle_trim(*trim),
                        Document::DoNotTrim => {
                            self.can_trim = false;
                        }
                    }
                }
            }

            if self.commands.is_empty() && !self.line_suffix.is_empty() {
                self.commands.extend(self.line_suffix.drain(..).rev());
            }
        }
    }

    fn remaining_width(&self) -> isize {
        (self.settings.print_width as isize) - (self.position as isize)
    }

    fn handle_bytes(&mut self, s: &'arena [u8]) {
        self.out.extend(s);
        self.position += string_width(s);
    }

    fn handle_space(&mut self, s: Space) {
        if s.soft {
            // If the previous character is a line terminator, space, or a tab, we don't need to add another one.
            if let Some(&last) = self.out.last()
                && is_line_terminator_or_space(last)
            {
                return;
            }
        }

        self.out.push(32u8);
        self.position += 1;
    }

    fn push_docs(&mut self, indentation: Indentation<'arena>, mode: Mode, docs: &'arena [Document<'arena, A>]) {
        self.commands.extend(docs.iter().rev().map(|doc| Command::new(indentation, mode, doc)));
    }

    #[inline]
    fn handle_trim(&mut self, trim: Trim) {
        if !self.can_trim {
            self.can_trim = true;

            return;
        }

        let should_pop: fn(u8) -> bool = match trim {
            Trim::Whitespace => is_space,
            Trim::Newlines => is_line_terminator_or_space,
        };

        while let Some(&last) = self.out.last() {
            if !should_pop(last) {
                break;
            }

            self.out.pop();
        }
        self.range_start = self.range_start.map(|offset| offset.min(self.out.len()));
        self.range_end = self.range_end.map(|offset| offset.min(self.out.len()));
    }

    fn handle_group(
        &mut self,
        indentation: &Indentation<'arena>,
        mode: Mode,
        doc: &'arena Document<'arena, A>,
        mut should_remeasure: bool,
    ) -> bool {
        #[allow(clippy::unreachable)]
        let Document::Group(group) = doc else {
            unreachable!();
        };

        let should_break = !matches!(*group.break_mode.borrow(), BreakMode::Auto);
        let group_id = group.id;

        if mode.is_flat() && !should_remeasure {
            self.push_docs(*indentation, if should_break { Mode::Break } else { mode }, &group.contents);

            self.set_group_mode_from_last_cmd(group_id);

            return should_remeasure;
        }

        should_remeasure = false;
        let remaining_width = self.remaining_width();
        let cmd = Command::new(*indentation, Mode::Flat, doc);
        if !should_break && self.fits(&cmd, remaining_width) {
            self.commands.push(cmd);
        } else {
            if let Some(expanded_states) = &group.expanded_states {
                #[allow(clippy::unwrap_used)]
                let (most_expanded, expanded_states) = expanded_states.split_last().unwrap();
                if should_break {
                    self.commands.push(Command::new(*indentation, Mode::Break, most_expanded));

                    return should_remeasure;
                }

                for state in expanded_states {
                    let cmd = Command::new(*indentation, Mode::Flat, state);
                    if self.fits(&cmd, remaining_width) {
                        self.commands.push(cmd);

                        return should_remeasure;
                    }
                }

                self.commands.push(Command::new(*indentation, Mode::Break, most_expanded));
            } else {
                self.commands.push(Command::array(*indentation, Mode::Break, &group.contents));
            }
        }

        self.set_group_mode_from_last_cmd(group_id);

        should_remeasure
    }

    fn handle_indent_if_break(
        &mut self,
        indentation: &Indentation<'arena>,
        mode: Mode,
        doc: &'arena IndentIfBreak<'arena, A>,
    ) {
        let IndentIfBreak { contents, group_id } = doc;
        let group_mode = self.group_mode_map.get(group_id).copied().unwrap_or(mode);

        let new_indentation = match group_mode {
            Mode::Flat => *indentation,
            Mode::Break => indentation.indented(),
        };

        self.push_docs(new_indentation, mode, contents);
    }

    fn handle_line(
        &mut self,
        line: Line,
        indentation: &Indentation<'arena>,
        mode: Mode,
        doc: &'arena Document<'arena, A>,
        mut should_remeasure: bool,
    ) -> bool {
        if mode.is_flat() {
            if !line.hard {
                if !line.soft {
                    self.out.push(b' ');
                    self.position += 1;
                }

                return should_remeasure;
            }

            should_remeasure = true;
        }

        if !self.line_suffix.is_empty() {
            self.commands.push(Command::new(*indentation, mode, doc));
            self.commands.extend(self.line_suffix.drain(..).rev());

            return should_remeasure;
        }

        if line.literal {
            self.out.extend(self.new_line.as_bytes());
            if indentation.is_root() {
                self.position = self.add_indentation(indentation);
            } else {
                self.position = 0;
            }

            return should_remeasure;
        }

        self.handle_trim(Trim::Whitespace);
        self.out.extend(self.new_line.as_bytes());
        self.position = self.add_indentation(indentation);

        should_remeasure
    }

    fn handle_line_suffix_boundary(
        &mut self,
        indentation: Indentation<'arena>,
        mode: Mode,
        mut should_remeasure: bool,
    ) -> bool {
        if !self.line_suffix.is_empty() {
            self.commands.push(Command::new(indentation, mode, self.arena.alloc(Document::space())));
            self.commands.extend(self.line_suffix.drain(..).rev());

            should_remeasure = true;
        }

        should_remeasure
    }

    fn handle_if_break(&mut self, if_break: &'arena IfBreak<'arena, A>, indentation: Indentation<'arena>, mode: Mode) {
        let IfBreak { break_contents, flat_content, group_id } = if_break;
        let group_mode = group_id.map_or(Some(mode), |id| self.group_mode_map.get(&id).copied()).unwrap_or(mode);

        match group_mode {
            Mode::Flat => {
                let flat_content = clone_in_arena(self.arena, flat_content);

                self.commands.push(Command::new(indentation, Mode::Flat, self.arena.alloc(flat_content)));
            }
            Mode::Break => {
                let break_contents = clone_in_arena(self.arena, break_contents);

                self.commands.push(Command::new(indentation, Mode::Break, self.arena.alloc(break_contents)));
            }
        }
    }

    fn add_indentation(&mut self, indentation: &Indentation<'arena>) -> usize {
        let depth = indentation.depth();
        let indentation_width = depth * self.settings.tab_width;
        let indentation_byte = if self.settings.use_tabs { b'\t' } else { b' ' };
        let indentation_length = if self.settings.use_tabs { depth } else { indentation_width };
        self.out.resize(self.out.len() + indentation_length, indentation_byte);

        let alignment_width = if let Some(alignment) = indentation.alignment() {
            self.out.extend(alignment);
            string_width(alignment)
        } else {
            0
        };

        indentation_width + alignment_width
    }

    fn set_group_mode_from_last_cmd(&mut self, id: Option<GroupIdentifier>) {
        let Some(id) = id else {
            return;
        };

        let Some(mode) = self.commands.last().map(|cmd| cmd.mode) else {
            return;
        };

        self.group_mode_map.insert(id, mode);
    }

    fn fits(&self, next: &Command<'arena, A>, width: isize) -> bool {
        let mut remaining_width = width;
        let mut has_line_suffix = false;
        let mut stack = FitsStack::new(self.arena);
        let mut cmds = self.commands.iter().rev();

        let mut first = Some(next);
        while let Some((mode, document)) =
            stack.next().map(|(mode, doc)| (mode, CommandDocument::Document(doc))).or_else(|| {
                first
                    .take()
                    .or_else(|| if has_line_suffix { None } else { cmds.next() })
                    .map(|cmd| (cmd.mode, cmd.document))
            })
        {
            let doc = match document {
                CommandDocument::Document(doc) => doc,
                CommandDocument::Array(docs) => {
                    if remaining_width < 0 {
                        return false;
                    }

                    stack.push(mode, docs);
                    continue;
                }
            };
            match doc {
                Document::String(string) => {
                    remaining_width -= string_width(string) as isize;
                }
                Document::Space(space) => {
                    if !space.soft {
                        remaining_width -= 1;
                    // Note: The check against `self.out` is an intentional simplification
                    // for `fits`, as the exact output isn't known. A soft space
                    // is assumed to have a width of 1 unless it's a line break.
                    } else if self.out.last().is_none_or(|&b| !is_space(b)) {
                        remaining_width -= 1;
                    }
                }
                Document::IndentIfBreak(IndentIfBreak { contents, .. })
                | Document::Indent(contents)
                | Document::Align(Align { contents, .. })
                | Document::Array(contents) => {
                    stack.push(mode, contents);
                }
                Document::Group(group) => {
                    let group_break_mode = *group.break_mode.borrow();
                    if group_break_mode == BreakMode::Preserve && mode.is_flat() {
                        return false;
                    }

                    let group_mode = if group_break_mode == BreakMode::Force { Mode::Break } else { mode };
                    if group.expanded_states.is_some() && group_mode.is_break() {
                        if let Some(expanded_states) = group.expanded_states.as_ref()
                            && let Some(last_state) = expanded_states.last()
                        {
                            stack.push(group_mode, std::slice::from_ref(last_state));
                        }
                    } else {
                        stack.push(group_mode, &group.contents);
                    }
                }
                Document::IfBreak(if_break_doc) => {
                    let group_mode =
                        if_break_doc.group_id.map_or(mode, |id| *self.group_mode_map.get(&id).unwrap_or(&Mode::Flat));
                    let contents =
                        if group_mode.is_break() { if_break_doc.break_contents } else { if_break_doc.flat_content };
                    stack.push(mode, std::slice::from_ref(contents));
                }
                Document::Line(line) => {
                    if mode.is_break() || line.hard {
                        return true;
                    }
                    if !line.soft {
                        remaining_width -= 1;
                    }
                }
                Document::LineSuffix(_) => {
                    has_line_suffix = true;
                }
                Document::LineSuffixBoundary => {
                    if has_line_suffix || !self.line_suffix.is_empty() {
                        return false;
                    }
                }
                Document::BreakParent
                | Document::Trim(_)
                | Document::DoNotTrim
                | Document::RangeStart
                | Document::RangeEnd => {}
            }

            if remaining_width < 0 {
                return false;
            }
        }

        true
    }

    #[inline]
    fn propagate_breaks(propagated_breaks: &mut HashSet<'arena, usize, A>, mut doc: &Document<'_, A>) -> bool {
        loop {
            match doc {
                Document::BreakParent => return true,
                Document::Group(group) => return Self::propagate_group_breaks(propagated_breaks, group),
                Document::IfBreak(d) => doc = d.break_contents,
                Document::Array(arr)
                | Document::Indent(arr)
                | Document::Align(Align { contents: arr, .. })
                | Document::IndentIfBreak(IndentIfBreak { contents: arr, .. }) => {
                    return Self::propagate_slice_breaks(propagated_breaks, arr);
                }
                _ => return false,
            }
        }
    }

    fn propagate_slice_breaks(
        propagated_breaks: &mut HashSet<'arena, usize, A>,
        documents: &[Document<'_, A>],
    ) -> bool {
        documents.iter().rev().any(|doc| Self::propagate_breaks(propagated_breaks, doc))
    }

    fn propagate_group_breaks(propagated_breaks: &mut HashSet<'arena, usize, A>, group: &Group<'_, A>) -> bool {
        if *group.break_mode.borrow() == BreakMode::Force {
            return true;
        }

        let cache_key = group.contents.first().map(|document| std::ptr::from_ref(document).addr()).or_else(|| {
            group
                .expanded_states
                .as_ref()
                .and_then(|states| states.first())
                .map(|document| std::ptr::from_ref(document).addr())
        });
        // The printer only changes break mode to Force, which returns above.
        if let Some(cache_key) = cache_key
            && propagated_breaks.contains(&cache_key)
        {
            return false;
        }

        let mut should_break = false;

        if let Some(expanded_states) = &group.expanded_states {
            should_break |= Self::propagate_slice_breaks(propagated_breaks, expanded_states);
        }

        should_break |= Self::propagate_slice_breaks(propagated_breaks, &group.contents);

        if group.expanded_states.is_none() && should_break && *group.break_mode.borrow() != BreakMode::Preserve {
            group.break_mode.replace(BreakMode::Force);
        }

        let should_break = *group.break_mode.borrow() == BreakMode::Force;
        if !should_break && let Some(cache_key) = cache_key {
            propagated_breaks.insert(cache_key);
        }

        should_break
    }
}
