use std::cmp::Ordering;

use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;
use bumpalo::vec;

use bumpalo::Bump;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::document::Align;
use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::document::Trim;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;

pub fn print_statement_sequence<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    stmts: &'arena Sequence<'arena, Statement<'arena>>,
) -> Vec<'arena, Document<'arena>> {
    let statements = stmts.nodes.iter().collect_in::<Vec<_>>(f.arena);

    print_statement_slice(f, statements.as_slice())
}

fn print_statement_slice<'ctx, 'arena>(
    f: &mut FormatterState<'ctx, 'arena>,
    stmts: &[&'arena Statement<'arena>],
) -> Vec<'arena, Document<'arena>> {
    let mut use_statements: std::vec::Vec<&'arena Use<'arena>> = std::vec::Vec::new();
    let mut parts = vec![in f.arena;];

    let last_statement_index = if stmts.is_empty() { None } else { stmts.len().checked_sub(1) };
    let mut i = 0;
    while i < stmts.len() {
        let stmt = stmts[i];

        if let Statement::Use(use_stmt) = stmt {
            use_statements.push(use_stmt);
            i += 1;
            continue;
        }

        if let Some(last_use) = use_statements.last() {
            let (should_add_line, should_add_space) = should_add_new_line_after_use(f, stmts, i, last_use);

            parts.extend(print_use_statements(f, use_statements.as_slice()));
            use_statements.clear();

            if should_add_line {
                parts.push(Document::Line(Line::hard()));

                if f.settings.empty_line_after_use {
                    parts.push(Document::Line(Line::hard()));
                }
            } else if should_add_space {
                parts.push(Document::space());
            }
        }

        let mut formatted_statement = format_statement_with_spacing(f, i, stmt, stmts, last_statement_index, i == 0);

        if let Statement::OpeningTag(tag) = stmt {
            let offset = tag.span().start.offset;
            let line = f.file.line_number(offset);

            if let Some(line_start_offset) = f.file.get_line_start_offset(line) {
                let c = &f.source_text[line_start_offset as usize..offset as usize];
                let ws = c.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                if !ws.is_empty() {
                    let mut j = i + 1;
                    let mut stmts_to_format = vec![in f.arena];
                    while j < stmts.len() {
                        let next_stmt = stmts[j];
                        stmts_to_format.push(next_stmt);
                        if next_stmt.terminates_scripting() {
                            break;
                        }

                        j += 1;
                    }

                    parts.push(Document::Group(Group::new(vec![in f.arena; Document::Align(Align {
                        alignment: f.as_str(&ws),
                        contents: {
                            formatted_statement.extend(print_statement_slice(f, stmts_to_format.as_slice()));

                            formatted_statement
                        },
                    })])));

                    i = j + 1;

                    continue;
                }
            }
        }

        parts.extend(formatted_statement);

        i += 1;
    }

    if !use_statements.is_empty() {
        parts.extend(print_use_statements(f, &use_statements));
    }

    parts
}

// New function to format statements with spacing and newlines
fn format_statement_with_spacing<'ctx, 'arena>(
    f: &mut FormatterState<'ctx, 'arena>,
    i: usize,
    stmt: &'arena Statement<'arena>,
    stmts: &[&'arena Statement<'arena>],
    last_statement_index: Option<usize>,
    is_first_statement: bool,
) -> Vec<'arena, Document<'arena>> {
    let mut statement_parts = vec![in f.arena;];

    let (should_add_new_line, should_add_space) = should_add_new_line_or_space_after_stmt(f, stmts, i, stmt);

    statement_parts.push(stmt.format(f));
    if should_add_space {
        let is_last = if let Some(index) = last_statement_index { i == index } else { i == stmts.len() - 1 };
        if !is_last {
            statement_parts.push(Document::space());
        }
    } else if should_add_new_line
        && let Some(index) = last_statement_index
        && i != index
    {
        statement_parts.push(Document::Line(Line::hard()));
        if should_add_empty_line_after(f, stmt) || f.is_next_line_empty(stmt.span()) {
            statement_parts.push(Document::Line(Line::hard()));
        }
    }

    if !is_first_statement && should_add_empty_line_before(f, stmt) {
        statement_parts.insert(
            0,
            Document::Array(vec![
                in f.arena;
                Document::Trim(Trim::Newlines),
                Document::Line(Line::hard()),
                Document::Line(Line::hard()),
            ]),
        );
    }

    statement_parts
}

#[inline]
const fn should_add_empty_line_after<'arena>(f: &FormatterState<'_, 'arena>, stmt: &'arena Statement<'arena>) -> bool {
    match stmt {
        Statement::OpeningTag(_) => f.settings.empty_line_after_opening_tag,
        Statement::Namespace(_) => f.settings.empty_line_after_namespace,
        Statement::Use(_) => f.settings.empty_line_after_use,
        Statement::Constant(_)
        | Statement::Function(_)
        | Statement::Class(_)
        | Statement::Interface(_)
        | Statement::Trait(_)
        | Statement::Enum(_) => f.settings.empty_line_after_symbols,
        Statement::Declare(_) => f.settings.empty_line_after_declare,
        Statement::Try(_)
        | Statement::Foreach(_)
        | Statement::For(_)
        | Statement::While(_)
        | Statement::DoWhile(_)
        | Statement::If(_)
        | Statement::Switch(_) => f.settings.empty_line_after_control_structure,
        _ => false,
    }
}

#[inline]
fn should_add_empty_line_before<'arena>(f: &FormatterState<'_, 'arena>, stmt: &'arena Statement<'arena>) -> bool {
    match stmt {
        Statement::Return(_) => f.settings.empty_line_before_return,
        _ => false,
    }
}

fn should_add_new_line_or_space_after_stmt<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    stmts: &[&'arena Statement<'arena>],
    i: usize,
    stmt: &'arena Statement<'arena>,
) -> (bool, bool) {
    if stmt.terminates_scripting() {
        return (false, false);
    }

    let mut should_add_space = false;
    let should_add_line = match stmt {
        Statement::HaltCompiler(_) | Statement::ClosingTag(_) | Statement::Inline(_) => false,
        Statement::Expression(ExpressionStatement { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Echo(Echo { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Global(Global { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Static(Static { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Unset(Unset { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Goto(Goto { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Constant(Constant { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Declare(Declare { body, .. }) => match body {
            DeclareBody::Statement(statement) => {
                return should_add_new_line_or_space_after_stmt(f, stmts, i, statement);
            }
            DeclareBody::ColonDelimited(_) => true,
        },
        Statement::OpeningTag(_) => {
            if let Some(index) = f.skip_to_line_end(Some(stmt.span().end_position().offset))
                && f.has_newline(index, false)
            {
                return (true, false);
            }

            should_add_space = !f.has_comment(stmt.span(), CommentFlags::Trailing);

            false
        }
        _ => {
            if f.has_newline(stmt.span().end_position().offset, false) {
                true
            } else if let Some(Statement::ClosingTag(_)) = stmts.get(i + 1) {
                should_add_space = !f.has_comment(stmt.span(), CommentFlags::Trailing);

                false
            } else {
                true
            }
        }
    };

    (should_add_line, should_add_space)
}

fn should_add_new_line_after_use<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    stmts: &[&'arena Statement<'arena>],
    i: usize,
    last_use: &'arena Use<'arena>,
) -> (bool, bool) {
    let mut should_add_space = false;
    let should_add_line = if last_use.terminator.is_closing_tag() {
        false
    } else if f.has_newline(last_use.span().end_position().offset, false) {
        true
    } else if let Some(Statement::ClosingTag(_)) = stmts.get(i) {
        should_add_space = !f.has_comment(last_use.span(), CommentFlags::Trailing);

        false
    } else {
        true
    };

    (should_add_line, should_add_space)
}

fn print_use_statements<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    stmts: &[&'arena Use<'arena>],
) -> Vec<'arena, Document<'arena>> {
    use std::vec::Vec;

    use bumpalo::collections::Vec as BumpVec;

    fn join_item_name<'arena>(arena: &'arena Bump, namespace: &[&'arena str], name: &'arena str) -> &'arena str {
        if namespace.is_empty() {
            return name;
        }

        let total_len = namespace.iter().map(|s| s.len()).sum::<usize>() + namespace.len();
        let mut bytes = BumpVec::with_capacity_in(total_len, arena);

        for (i, part) in namespace.iter().enumerate() {
            bytes.extend_from_slice(part.as_bytes());
            if i < namespace.len() {
                // Add a separator after every part
                bytes.push(b'\\');
            }
        }

        bytes.extend_from_slice(name.as_bytes());

        // SAFETY: We are joining valid UTF-8 slices with an ASCII separator.
        unsafe { std::str::from_utf8_unchecked(bytes.into_bump_slice()) }
    }

    let should_sort = f.settings.sort_uses;
    let should_separate = f.settings.separate_use_types;
    let should_expand = f.settings.expand_use_groups;

    let mut all_expanded_items: Vec<ExpandedUseItem<'arena>> = Vec::new();
    for use_stmt in stmts {
        all_expanded_items.extend(expand_use(f, use_stmt, should_expand));
    }

    if should_sort {
        all_expanded_items.sort_by(|a, b| {
            let a_type_order = match a.use_type {
                None => 0,
                Some(ty) => {
                    if ty.is_function() {
                        1
                    } else {
                        2
                    }
                }
            };

            let b_type_order = match b.use_type {
                None => 0,
                Some(ty) => {
                    if ty.is_function() {
                        1
                    } else {
                        2
                    }
                }
            };

            if a_type_order != b_type_order {
                return a_type_order.cmp(&b_type_order);
            }

            let a_full_name = join_item_name(f.arena, &a.namespace, a.name);
            let b_full_name = join_item_name(f.arena, &b.namespace, b.name);

            let mut a_chars = a_full_name.chars().flat_map(|c| c.to_lowercase());
            let mut b_chars = b_full_name.chars().flat_map(|c| c.to_lowercase());

            loop {
                match (a_chars.next(), b_chars.next()) {
                    (Some(ac), Some(bc)) => match ac.cmp(&bc) {
                        Ordering::Equal => continue,
                        other => return other,
                    },
                    (None, Some(_)) => return Ordering::Less,
                    (Some(_), None) => return Ordering::Greater,
                    (None, None) => return Ordering::Equal,
                }
            }
        });
    }

    let mut grouped_items: Vec<Vec<ExpandedUseItem<'arena>>> = Vec::new();
    if should_separate {
        #[derive(PartialEq, Eq)]
        enum UseTypeDiscriminant {
            Function,
            Const,
        }

        let mut current_group: Vec<ExpandedUseItem<'arena>> = Vec::new();
        let mut current_type: Option<UseTypeDiscriminant> = None;

        for item in all_expanded_items {
            let item_type = item
                .use_type
                .map(|ty| if ty.is_function() { UseTypeDiscriminant::Function } else { UseTypeDiscriminant::Const });

            if current_type != item_type {
                if !current_group.is_empty() {
                    grouped_items.push(std::mem::take(&mut current_group));
                }

                current_type = item_type;
            }
            current_group.push(item);
        }
        if !current_group.is_empty() {
            grouped_items.push(current_group);
        }
    } else {
        grouped_items.push(all_expanded_items);
    }

    let mut result_docs: BumpVec<'arena, Document<'arena>> = vec![in f.arena];
    let grouped_items_count = grouped_items.len();
    for (index, group) in grouped_items.into_iter().enumerate() {
        let is_last_grouped_items = index + 1 == grouped_items_count;

        let group_count = group.len();
        for (item_index, item) in group.into_iter().enumerate() {
            let is_last_group = item_index + 1 == group_count;

            if should_expand {
                let mut parts = vec![in f.arena;];
                parts.push(item.original_node.r#use.format(f));
                parts.push(Document::space());

                if let Some(ty) = item.use_type {
                    parts.push(ty.format(f));
                    parts.push(Document::space());
                }

                parts.push(Document::String(join_item_name(f.arena, &item.namespace, item.name)));

                if let Some(alias) = item.alias {
                    parts.push(Document::space());
                    parts.push(Document::String("as "));
                    parts.push(Document::String(alias));
                }

                parts.push(item.original_node.terminator.format(f));
                result_docs.push(Document::Group(Group::new(parts)));
            } else {
                result_docs.push(item.original_node.format(f));
            }

            if !is_last_grouped_items || !is_last_group {
                result_docs.push(Document::Line(Line::hard()));
            }
        }

        if !is_last_grouped_items {
            result_docs.push(Document::Line(Line::hard()));
        }
    }

    result_docs
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ExpandedUseItem<'arena> {
    use_type: Option<&'arena UseType<'arena>>,
    namespace: Vec<'arena, &'arena str>,
    name: &'arena str,
    alias: Option<&'arena str>,
    original_node: &'arena Use<'arena>,
}

fn expand_use<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    use_stmt: &'arena Use<'arena>,
    should_expand: bool,
) -> std::vec::Vec<ExpandedUseItem<'arena>> {
    let mut expanded_items = std::vec::Vec::new();

    fn expand_items<'arena>(
        f: &mut FormatterState<'_, 'arena>,
        items: &'arena UseItems<'arena>,
        current_namespace: Vec<'arena, &'arena str>,
        use_type: Option<&'arena UseType<'arena>>,
        expanded_items: &mut std::vec::Vec<ExpandedUseItem<'arena>>,
        original_node: &'arena Use<'arena>,
        should_expand: bool,
    ) {
        match items {
            UseItems::Sequence(seq) => {
                if should_expand {
                    for item in seq.items.iter() {
                        expand_single_item(f, item, current_namespace.clone(), use_type, expanded_items, original_node);
                    }
                } else {
                    // If not expanding, create *one* ExpandedUseItem for the entire sequence.
                    expanded_items.push(ExpandedUseItem {
                        use_type,
                        namespace: current_namespace,
                        name: "", // We don't need name/alias when not expanding.
                        alias: None,
                        original_node,
                    });
                }
            }
            UseItems::TypedSequence(seq) => {
                if should_expand {
                    for item in seq.items.iter() {
                        expand_single_item(
                            f,
                            item,
                            current_namespace.clone(),
                            Some(&seq.r#type),
                            expanded_items,
                            original_node,
                        );
                    }
                } else {
                    expanded_items.push(ExpandedUseItem {
                        use_type,
                        namespace: current_namespace,
                        name: "", // We don't need name/alias when not expanding.
                        alias: None,
                        original_node,
                    });
                }
            }
            UseItems::TypedList(list) => {
                if should_expand {
                    let mut new_namespace = current_namespace.clone();
                    new_namespace.push(list.namespace.value());
                    for item in list.items.iter() {
                        expand_single_item(
                            f,
                            item,
                            new_namespace.clone(),
                            Some(&list.r#type),
                            expanded_items,
                            original_node,
                        );
                    }
                } else {
                    expanded_items.push(ExpandedUseItem {
                        use_type,
                        namespace: current_namespace,
                        name: "", // We don't need name/alias when not expanding.
                        alias: None,
                        original_node,
                    });
                }
            }
            UseItems::MixedList(list) => {
                if should_expand {
                    let mut new_namespace = current_namespace.clone();
                    new_namespace.push(list.namespace.value());
                    for maybe_typed_item in list.items.iter() {
                        expand_single_item(
                            f,
                            &maybe_typed_item.item,
                            new_namespace.clone(),
                            maybe_typed_item.r#type.as_ref(),
                            expanded_items,
                            original_node,
                        );
                    }
                } else {
                    expanded_items.push(ExpandedUseItem {
                        use_type,
                        namespace: current_namespace,
                        name: "", // We don't need name/alias when not expanding.
                        alias: None,
                        original_node,
                    });
                }
            }
        }
    }

    fn expand_single_item<'arena>(
        f: &mut FormatterState<'_, 'arena>,
        item: &'arena UseItem<'arena>,
        mut current_namespace: Vec<'arena, &'arena str>,
        use_type: Option<&'arena UseType<'arena>>,
        expanded_items: &mut std::vec::Vec<ExpandedUseItem<'arena>>,
        original_node: &'arena Use<'arena>,
    ) {
        let mut parts = item.name.value().split("\\").collect_in::<Vec<_>>(f.arena);
        // SAFETY: split always returns at least one element
        let name = unsafe { parts.pop().unwrap_unchecked() };
        current_namespace.extend(parts);

        expanded_items.push(ExpandedUseItem {
            use_type,
            namespace: current_namespace,
            name,
            alias: item.alias.as_ref().map(|a| a.identifier.value),
            original_node,
        });
    }

    expand_items(f, &use_stmt.items, vec![in f.arena], None, &mut expanded_items, use_stmt, should_expand); // Pass should_expand

    expanded_items
}

pub fn sort_use_items<'arena>(
    items: impl Iterator<Item = &'arena UseItem<'arena>>,
) -> std::vec::Vec<&'arena UseItem<'arena>> {
    let mut items = items.collect::<std::vec::Vec<_>>();
    items.sort_by(|a, b| a.name.value().to_lowercase().cmp(&b.name.value().to_lowercase()));
    items
}

pub fn sort_maybe_typed_use_items<'arena>(
    items: impl Iterator<Item = &'arena MaybeTypedUseItem<'arena>>,
) -> std::vec::Vec<&'arena MaybeTypedUseItem<'arena>> {
    let mut items = items.collect::<std::vec::Vec<_>>();
    items.sort_by(|a, b| {
        let a_type_order = match &a.r#type {
            None => 0,
            Some(ty) => {
                if ty.is_function() {
                    1
                } else {
                    2
                }
            }
        };

        let b_type_order = match &b.r#type {
            None => 0,
            Some(ty) => {
                if ty.is_function() {
                    1
                } else {
                    2
                }
            }
        };

        if a_type_order != b_type_order {
            return a_type_order.cmp(&b_type_order);
        }

        a.item.name.value().to_lowercase().cmp(&b.item.name.value().to_lowercase())
    });

    items
}
