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

pub fn print_statement_sequence<'a>(f: &mut FormatterState<'a>, stmts: &'a Sequence<Statement>) -> Vec<Document<'a>> {
    let stmts = stmts.nodes.iter().collect::<Vec<_>>();

    print_statement_slice(f, &stmts)
}

fn print_statement_slice<'a>(f: &mut FormatterState<'a>, stmts: &[&'a Statement]) -> Vec<Document<'a>> {
    let mut use_statements: Vec<&'a Use> = Vec::new();
    let mut parts = vec![];

    let last_non_noop_index = stmts.iter().rposition(|stmt| !matches!(stmt, Statement::Noop(_)));
    let mut i = 0;
    while i < stmts.len() {
        let stmt = stmts[i];

        if let Statement::Use(use_stmt) = stmt {
            use_statements.push(use_stmt);
            i += 1;
            continue;
        }

        if !use_statements.is_empty() {
            parts.extend(print_use_statements(f, std::mem::take(&mut use_statements)));
            parts.push(Document::Line(Line::hard()));

            if f.settings.empty_line_after_use {
                parts.push(Document::Line(Line::hard()));
            }
        }

        let mut formatted_statement = format_statement_with_spacing(f, i, stmt, stmts, last_non_noop_index);

        if let Statement::OpeningTag(tag) = stmt {
            let offset = tag.span().start.offset;
            let line = f.source.line_number(offset);

            if let Some(line_start_offset) = f.source.get_line_start_offset(line) {
                let c = &f.source_text[line_start_offset..offset];
                let ws = c.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                if !ws.is_empty() {
                    let mut j = i + 1;
                    let mut stmts_to_format = vec![];
                    while j < stmts.len() {
                        let next_stmt = stmts[j];
                        stmts_to_format.push(next_stmt);
                        if next_stmt.terminates_scripting() {
                            break;
                        }

                        j += 1;
                    }

                    parts.push(Document::Group(Group::new(vec![Document::Align(Align {
                        alignment: f.as_str(&ws),
                        contents: {
                            formatted_statement.extend(print_statement_slice(f, &stmts_to_format));

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
        parts.extend(print_use_statements(f, use_statements));
    }

    parts
}

// New function to format statements with spacing and newlines
fn format_statement_with_spacing<'a>(
    f: &mut FormatterState<'a>,
    i: usize,
    stmt: &'a Statement,
    stmts: &[&'a Statement],
    last_non_noop_index: Option<usize>,
) -> Vec<Document<'a>> {
    let mut statement_parts = vec![];

    let (should_add_new_line, should_add_space) = should_add_new_line_or_space_after_stmt(f, stmts, i, stmt);

    statement_parts.push(stmt.format(f));
    if should_add_space {
        let is_last = if let Some(index) = last_non_noop_index { i == index } else { i == stmts.len() - 1 };
        if !is_last {
            statement_parts.push(Document::space());
        }
    } else if should_add_new_line {
        if let Some(index) = last_non_noop_index {
            if i != index {
                statement_parts.push(Document::Line(Line::hard()));
                if should_add_empty_line_after(f, stmt) || f.is_next_line_empty(stmt.span()) {
                    statement_parts.push(Document::Line(Line::hard()));
                }
            }
        }
    }

    if should_add_empty_line_before(f, stmt) {
        statement_parts.insert(
            0,
            Document::Array(vec![
                Document::Trim(Trim::Newlines),
                Document::Line(Line::hard()),
                Document::Line(Line::hard()),
            ]),
        );
    }

    statement_parts
}

#[inline(always)]
const fn should_add_empty_line_after<'a>(f: &FormatterState<'a>, stmt: &'a Statement) -> bool {
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

#[inline(always)]
fn should_add_empty_line_before<'a>(f: &FormatterState<'a>, stmt: &'a Statement) -> bool {
    match stmt {
        Statement::Return(_) => {
            if !f.settings.empty_line_before_return {
                return false;
            }

            f.has_newline(stmt.span().start.offset, /* backwards */ true)
        }
        _ => false,
    }
}

fn should_add_new_line_or_space_after_stmt<'a>(
    f: &mut FormatterState<'a>,
    stmts: &[&'a Statement],
    i: usize,
    stmt: &'a Statement,
) -> (bool, bool) {
    let mut should_add_space = false;

    let should_add_line = match stmt {
        Statement::ClosingTag(_) => false,
        Statement::Inline(_) => false,
        Statement::Expression(ExpressionStatement { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Echo(Echo { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Global(Global { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Static(Static { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Unset(Unset { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::HaltCompiler(HaltCompiler { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Goto(Goto { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Constant(Constant { terminator: Terminator::ClosingTag(_), .. }) => false,
        Statement::Declare(Declare { body, .. }) => match body {
            DeclareBody::Statement(statement) => {
                return should_add_new_line_or_space_after_stmt(f, stmts, i, statement);
            }
            DeclareBody::ColonDelimited(_) => true,
        },
        Statement::OpeningTag(_) => {
            if let Some(index) = f.skip_to_line_end(Some(stmt.span().end_position().offset)) {
                if f.has_newline(index, false) {
                    return (true, false);
                }
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

fn print_use_statements<'a>(f: &mut FormatterState<'a>, stmts: Vec<&'a Use>) -> Vec<Document<'a>> {
    let should_sort = f.settings.sort_uses;
    let should_separate = f.settings.separate_use_types;
    let should_expand = f.settings.expand_use_groups;

    let mut all_expanded_items: Vec<ExpandedUseItem<'a>> = Vec::new();
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

            let mut a_full_name = a.namespace.join("\\");
            if !a_full_name.is_empty() {
                a_full_name.push('\\');
            }
            a_full_name.push_str(a.name);

            let mut b_full_name = b.namespace.join("\\");
            if !b_full_name.is_empty() {
                b_full_name.push('\\');
            }
            b_full_name.push_str(b.name);

            a_full_name.to_lowercase().cmp(&b_full_name.to_lowercase())
        });
    }

    let mut grouped_items: Vec<Vec<ExpandedUseItem<'a>>> = Vec::new();
    if should_separate {
        #[derive(PartialEq, Eq)]
        enum UseTypeDiscriminant {
            Function,
            Const,
        }

        let mut current_group: Vec<ExpandedUseItem<'a>> = Vec::new();
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

    let mut result_docs: Vec<Document<'a>> = Vec::new();
    let grouped_items_count = grouped_items.len();
    for (index, group) in grouped_items.into_iter().enumerate() {
        let is_last_grouped_items = index + 1 == grouped_items_count;

        let group_count = group.len();
        for (item_index, item) in group.into_iter().enumerate() {
            let is_last_group = item_index + 1 == group_count;

            if should_expand {
                let mut parts = vec![];
                parts.push(item.original_node.r#use.format(f));
                parts.push(Document::space());

                if let Some(ty) = item.use_type {
                    parts.push(ty.format(f));
                    parts.push(Document::space());
                }

                let joined_namespace = item.namespace.join("\\");

                if !joined_namespace.is_empty() {
                    parts.push(Document::String(f.as_str(joined_namespace)));
                    parts.push(Document::String("\\"));
                }

                parts.push(Document::String(item.name));

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
struct ExpandedUseItem<'a> {
    use_type: Option<&'a UseType>,
    namespace: Vec<&'a str>,
    name: &'a str,
    alias: Option<&'a str>,
    original_node: &'a Use,
}

fn expand_use<'a>(f: &mut FormatterState<'a>, use_stmt: &'a Use, should_expand: bool) -> Vec<ExpandedUseItem<'a>> {
    let mut expanded_items = Vec::new();

    fn expand_items<'a>(
        f: &mut FormatterState<'a>,
        items: &'a UseItems,
        current_namespace: Vec<&'a str>,
        use_type: Option<&'a UseType>,
        expanded_items: &mut Vec<ExpandedUseItem<'a>>,
        original_node: &'a Use,
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
                    new_namespace.push(f.interner.lookup(list.namespace.value()));
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
                    new_namespace.push(f.interner.lookup(list.namespace.value()));
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

    fn expand_single_item<'a>(
        f: &mut FormatterState<'a>,
        item: &'a UseItem,
        mut current_namespace: Vec<&'a str>,
        use_type: Option<&'a UseType>,
        expanded_items: &mut Vec<ExpandedUseItem<'a>>,
        original_node: &'a Use,
    ) {
        let mut parts = f.interner.lookup(item.name.value()).split("\\").collect::<Vec<_>>();
        // SAFETY: split always returns at least one element
        let name = unsafe { parts.pop().unwrap_unchecked() };
        current_namespace.extend(parts);

        expanded_items.push(ExpandedUseItem {
            use_type,
            namespace: current_namespace,
            name,
            alias: item.alias.as_ref().map(|a| f.interner.lookup(&a.identifier.value)),
            original_node,
        });
    }

    expand_items(f, &use_stmt.items, Vec::new(), None, &mut expanded_items, use_stmt, should_expand); // Pass should_expand
    expanded_items
}

pub fn sort_use_items<'a>(f: &mut FormatterState<'a>, items: impl Iterator<Item = &'a UseItem>) -> Vec<&'a UseItem> {
    let mut items = items.collect::<Vec<_>>();
    items.sort_by(|a, b| {
        let a_name = f.interner.lookup(a.name.value());
        let b_name = f.interner.lookup(b.name.value());

        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });

    items
}

pub fn sort_maybe_typed_use_items<'a>(
    f: &mut FormatterState<'a>,
    items: impl Iterator<Item = &'a MaybeTypedUseItem>,
) -> Vec<&'a MaybeTypedUseItem> {
    let mut items = items.collect::<Vec<_>>();
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

        let a_name = f.interner.lookup(a.item.name.value());
        let b_name = f.interner.lookup(b.item.name.value());

        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });

    items
}
