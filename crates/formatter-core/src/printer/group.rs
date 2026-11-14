use crate::document::group::GroupIdentifier;
use crate::document::{BreakMode, Document, IfBreak, IndentIfBreak, clone_in_arena};
use crate::printer::Printer;
use crate::printer::command::{Command, Indentation, Mode};
use mago_allocator::Arena;

impl<'arena, A: Arena> Printer<'arena, A> {
    pub(super) fn remaining_width(&self) -> isize {
        (self.settings.print_width as isize) - (self.position as isize)
    }

    pub(super) fn handle_group(
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

    pub(super) fn handle_indent_if_break(
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

    pub(super) fn handle_if_break(
        &mut self,
        if_break: &'arena IfBreak<'arena, A>,
        indentation: Indentation<'arena>,
        mode: Mode,
    ) {
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

    pub(super) fn set_group_mode_from_last_cmd(&mut self, id: Option<GroupIdentifier>) {
        let Some(id) = id else {
            return;
        };

        let Some(mode) = self.commands.last().map(|cmd| cmd.mode) else {
            return;
        };

        self.group_mode_map.insert(id, mode);
    }
}
