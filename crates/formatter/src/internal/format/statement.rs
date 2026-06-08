use mago_allocator::vec_in;
use std::cmp::Ordering;

use mago_allocator::Arena;
use mago_allocator::CollectIn;
use mago_allocator::vec::Vec;

use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::ast::ClosingTag;
use mago_syntax::ast::Constant;
use mago_syntax::ast::Declare;
use mago_syntax::ast::DeclareBody;
use mago_syntax::ast::Echo;
use mago_syntax::ast::ExpressionStatement;
use mago_syntax::ast::Global;
use mago_syntax::ast::Goto;
use mago_syntax::ast::MaybeTypedUseItem;
use mago_syntax::ast::Sequence;
use mago_syntax::ast::Statement;
use mago_syntax::ast::Static;
use mago_syntax::ast::Terminator;
use mago_syntax::ast::Unset;
use mago_syntax::ast::Use;
use mago_syntax::ast::UseItem;
use mago_syntax::ast::UseItems;
use mago_syntax::ast::UseType;

use mago_syntax::ast::Expression;
use mago_syntax::walker::MutWalker;

use crate::document::Align;
use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::document::Trim;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::alignment::AlignmentWidths;
use crate::internal::format::alignment::detect_statement_ref_alignment_runs;
use crate::internal::format::alignment::get_statement_alignment;
use crate::internal::format::assignment::AssignmentAlignment;
use crate::internal::format::misc::has_new_line_in_range;

pub fn print_statement_sequence<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    stmts: &'arena Sequence<'arena, Statement<'arena>>,
) -> Vec<'arena, Document<'arena, A>, A>
where
    A: Arena,
{
    let statements = stmts.nodes.iter().collect_in::<Vec<'arena, _, A>>(f.arena);

    print_statement_slice(f, statements.as_slice())
}

fn print_statement_slice<'ctx, 'arena, A>(
    f: &mut FormatterState<'ctx, 'arena, A>,
    stmts: &[&'arena Statement<'arena>],
) -> Vec<'arena, Document<'arena, A>, A>
where
    A: Arena,
{
    let mut use_statements: std::vec::Vec<&'arena Use<'arena>> = std::vec::Vec::new();
    let mut parts = vec_in![f.arena;];

    // Detect alignment runs for consecutive assignment statements and global constants
    let alignment_runs = detect_statement_ref_alignment_runs(f, stmts);

    let last_statement_index = if stmts.is_empty() { None } else { stmts.len().checked_sub(1) };
    let mut i = 0;
    while i < stmts.len() {
        let stmt = stmts[i];
        let stmt_start = stmt.span().start.offset;

        // Check if this statement falls within an ignore region
        if let Some(region) = f.get_ignore_region_for(stmt_start).copied() {
            // First, flush any pending use statements that came before the ignore region
            if !use_statements.is_empty() {
                parts.extend(print_use_statements(f, use_statements.as_slice()));
                use_statements.clear();
                parts.push(Document::Line(Line::hard()));
            }

            // Output the preserved source for this region
            let preserved = f.get_source_slice(region.start, region.end);
            parts.push(Document::String(preserved));

            // Skip comments within the region
            f.skip_comments_until(region.end);

            // Skip all statements that fall within this region
            while i < stmts.len() && stmts[i].span().end.offset <= region.end {
                i += 1;
            }

            // Add newline if there are more statements
            if i < stmts.len() {
                parts.push(Document::Line(Line::hard()));
                // Preserve blank line after ignore region if it existed in source
                if f.is_next_line_empty_after_index(region.end) {
                    parts.push(Document::Line(Line::hard()));
                }
            }
            continue;
        }

        // Check if there's a format-ignore-next marker before this statement
        if let Some(marker_start) = f.consume_ignore_next_before(stmt_start) {
            // First, flush any pending use statements
            if !use_statements.is_empty() {
                parts.extend(print_use_statements(f, use_statements.as_slice()));
                use_statements.clear();
                parts.push(Document::Line(Line::hard()));
            }

            // Preserve the marker comment and statement as-is
            let stmt_end = stmt.span().end.offset;
            let preserved = f.get_source_slice(marker_start, stmt_end);
            parts.push(Document::String(preserved));

            // Skip comments within the preserved region
            f.skip_comments_until(stmt_end);

            i += 1;

            // Add newline if there are more statements
            if i < stmts.len() {
                parts.push(Document::Line(Line::hard()));
                if f.is_next_line_empty_after_index(stmt_end) {
                    parts.push(Document::Line(Line::hard()));
                }
            }
            continue;
        }

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

        if let Some(widths) = get_statement_alignment(&alignment_runs, i) {
            let alignment = calculate_statement_alignment(f, stmt, &widths);
            f.set_alignment_context(Some(alignment));
        }

        let mut formatted_statement = format_statement_with_spacing(f, i, stmt, stmts, last_statement_index, i == 0);

        f.set_alignment_context(None);

        let tag_offset = match stmt {
            Statement::OpeningTag(tag) => Some(tag.span().start.offset),
            Statement::EchoTag(tag) => Some(tag.tag.start.offset),
            _ => None,
        };

        if let Some(offset) = tag_offset {
            let line = f.file.line_number(offset);

            if let Some(line_start_offset) = f.file.get_line_start_offset(line) {
                let c = &f.source_text[line_start_offset as usize..offset as usize];
                let ws_len = c.iter().take_while(|&&b| b.is_ascii_whitespace()).count();
                let ws: &[u8] = &c[..ws_len];
                let c_ends_in_html = {
                    let (mut in_php, mut i, mut ok_to_align) = (false, 0usize, true);
                    while i < c.len() {
                        if !in_php && c[i..].starts_with(b"<?") {
                            in_php = true;
                            i += 2;
                        } else if in_php && c[i..].starts_with(b"?>") {
                            in_php = false;
                            i += 2;
                        } else if !in_php && c[i..].starts_with(b"?>") {
                            // `?>` seen while not in tracked PHP context: the line started
                            // inside PHP code. Only allow alignment if the tail between the
                            // leading whitespace and this `?>` is purely closing syntax
                            // (`)`, `]`, `}`, `;`, whitespace), i.e. the leftover of a
                            // broken multi-line expression — not actual statement code.
                            let tail = &c[ws_len..i];
                            if !tail.iter().all(|&byte| matches!(byte, b')' | b']' | b'}' | b';' | b' ' | b'\t')) {
                                ok_to_align = false;
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    !in_php && ok_to_align
                };
                let should_apply_align = !ws.is_empty()
                    && (matches!(stmt, Statement::OpeningTag(_)) || c.len() == ws.len() || c_ends_in_html);
                if should_apply_align {
                    let alignment_slice: &'arena [u8] = {
                        let mut buf = Vec::with_capacity_in(ws.len(), f.arena);
                        buf.extend_from_slice(ws);
                        buf.leak()
                    };
                    if matches!(stmt, Statement::OpeningTag(_)) {
                        let mut j = i + 1;
                        let mut stmts_to_format = vec_in![f.arena];
                        while j < stmts.len() {
                            let next_stmt = stmts[j];
                            stmts_to_format.push(next_stmt);
                            if next_stmt.terminates_scripting() {
                                break;
                            }

                            j += 1;
                        }

                        parts.push(Document::Group(Group::new(vec_in![f.arena; Document::Align(Align {
                            alignment: alignment_slice,
                            contents: {
                                formatted_statement.extend(print_statement_slice(f, stmts_to_format.as_slice()));
                                formatted_statement
                            },
                        })])));

                        i = j + 1;
                    } else {
                        parts.push(Document::Group(Group::new(vec_in![f.arena; Document::Align(Align {
                            alignment: alignment_slice,
                            contents: formatted_statement,
                        })])));

                        i += 1;
                    }

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
fn format_statement_with_spacing<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    i: usize,
    stmt: &'arena Statement<'arena>,
    stmts: &[&'arena Statement<'arena>],
    last_statement_index: Option<usize>,
    is_first_statement: bool,
) -> Vec<'arena, Document<'arena, A>, A>
where
    A: Arena,
{
    let mut statement_parts = vec_in![f.arena;];

    let (should_add_new_line, should_add_space) = should_add_new_line_or_space_after_stmt(f, stmts, i, stmt);

    match stmt.format(f) {
        Document::Array(arr) => statement_parts.extend(arr),
        other => statement_parts.push(other),
    }

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

        let should_add_empty_line = if should_add_empty_line_after(f, stmt) {
            if !f.settings.empty_line_between_same_symbols && is_symbol(stmt) {
                let next_stmt = stmts.get(i + 1);
                next_stmt.is_none_or(|next| !is_same_symbol_type(stmt, next))
            } else {
                true
            }
        } else {
            false
        };

        if should_add_empty_line || f.is_next_line_empty(stmt.span()) {
            statement_parts.push(Document::Line(Line::hard()));
        }
    }

    if !is_first_statement && should_add_empty_line_before(f, stmt) {
        statement_parts.insert(
            0,
            Document::Array(vec_in![f.arena;
                Document::Trim(Trim::Newlines),
                Document::Line(Line::hard()),
                Document::Line(Line::hard()),
            ]),
        );
    }

    statement_parts
}

#[inline]
const fn should_add_empty_line_after<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    stmt: &'arena Statement<'arena>,
) -> bool
where
    A: Arena,
{
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
fn should_add_empty_line_before<'arena, A>(f: &FormatterState<'_, 'arena, A>, stmt: &'arena Statement<'arena>) -> bool
where
    A: Arena,
{
    match stmt {
        Statement::Return(_) => f.settings.empty_line_before_return,
        _ => false,
    }
}

/// Check if a statement is a symbol (class, enum, interface, trait, function, const).
#[inline]
const fn is_symbol(stmt: &Statement<'_>) -> bool {
    matches!(
        stmt,
        Statement::Constant(_)
            | Statement::Function(_)
            | Statement::Class(_)
            | Statement::Interface(_)
            | Statement::Trait(_)
            | Statement::Enum(_)
    )
}

/// Check if two statements are the same symbol type.
#[inline]
const fn is_same_symbol_type(a: &Statement<'_>, b: &Statement<'_>) -> bool {
    matches!(
        (a, b),
        (Statement::Constant(_), Statement::Constant(_))
            | (Statement::Function(_), Statement::Function(_))
            | (Statement::Class(_), Statement::Class(_))
            | (Statement::Interface(_), Statement::Interface(_))
            | (Statement::Trait(_), Statement::Trait(_))
            | (Statement::Enum(_), Statement::Enum(_))
    )
}

fn should_add_new_line_or_space_after_stmt<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    stmts: &[&'arena Statement<'arena>],
    i: usize,
    stmt: &'arena Statement<'arena>,
) -> (bool, bool)
where
    A: Arena,
{
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
            if f.settings.opening_tag_on_own_line && !is_inline_php_template(stmts) {
                return (true, false);
            }

            if let Some(index) = f.skip_to_line_end(Some(stmt.end_position().offset()))
                && f.has_newline(index, false)
            {
                return (true, false);
            }

            should_add_space = !f.has_comment(stmt.span(), CommentFlags::TRAILING);

            false
        }
        _ => {
            if f.has_newline(stmt.end_position().offset(), false) {
                true
            } else if let Some(Statement::ClosingTag(_)) = stmts.get(i + 1) {
                should_add_space = !f.has_comment(stmt.span(), CommentFlags::TRAILING);

                false
            } else {
                true
            }
        }
    };

    (should_add_line, should_add_space)
}

/// Check if the program is an inline PHP template (mixes PHP and HTML).
///
/// When a file contains inline HTML content, the opening `<?php` tag should stay
/// on the same line as the following statement (e.g., `<?php if ($foo): ?>`).
/// When it's a pure PHP file, `<?php` should be on its own line.
///
/// Counts closing tags in the given statements. A trailing `ClosingTag` (last top-level statement)
/// is not counted since the formatter removes it. If any other closing tags exist,
/// the file is an inline PHP template.
#[inline]
#[allow(clippy::if_same_then_else, clippy::bool_to_int_with_if)]
fn is_inline_php_template(stmts: &[&Statement<'_>]) -> bool {
    let trailing_close_tag_count = match stmts.len() {
        0 => 0,
        n => {
            // Pattern: [..., ClosingTag, Inline(whitespace-only)] at end
            if matches!(stmts.get(n.wrapping_sub(2)), Some(Statement::ClosingTag(_)))
                && matches!(stmts.get(n - 1), Some(Statement::Inline(inline)) if inline.value.trim_ascii().is_empty())
            {
                1
            // Pattern: [..., ClosingTag] at end (no trailing inline)
            } else if matches!(stmts.last(), Some(Statement::ClosingTag(_))) {
                1
            } else {
                0
            }
        }
    };

    let count = count_closing_tags(stmts);

    count > trailing_close_tag_count
}

/// Count all `ClosingTag` nodes within a slice of statements at any depth.
fn count_closing_tags(stmts: &[&Statement<'_>]) -> usize {
    struct Counter(usize);

    impl<'ast> MutWalker<'ast, '_, ()> for Counter {
        fn walk_in_closing_tag(&mut self, _: &'ast ClosingTag, _: &mut ()) {
            self.0 += 1;
        }
    }

    let mut counter = Counter(0);
    for stmt in stmts {
        counter.walk_statement(stmt, &mut ());
    }

    counter.0
}

fn should_add_new_line_after_use<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    stmts: &[&'arena Statement<'arena>],
    i: usize,
    last_use: &'arena Use<'arena>,
) -> (bool, bool)
where
    A: Arena,
{
    let mut should_add_space = false;
    let should_add_line = if last_use.terminator.is_closing_tag() {
        false
    } else if f.has_newline(last_use.span().end_position().offset, false) {
        true
    } else if let Some(Statement::ClosingTag(_)) = stmts.get(i) {
        should_add_space = !f.has_comment(last_use.span(), CommentFlags::TRAILING);

        false
    } else {
        true
    };

    (should_add_line, should_add_space)
}

fn print_use_statements<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    stmts: &[&'arena Use<'arena>],
) -> Vec<'arena, Document<'arena, A>, A>
where
    A: Arena,
{
    use std::vec::Vec;

    use mago_allocator::vec::Vec as BumpVec;

    fn join_item_name<'arena, A>(arena: &'arena A, namespace: &[&'arena [u8]], name: &'arena [u8]) -> &'arena [u8]
    where
        A: Arena,
    {
        if namespace.is_empty() {
            return name;
        }

        let total_len = namespace.iter().map(|s| s.len()).sum::<usize>() + namespace.len();
        let mut bytes = BumpVec::with_capacity_in(total_len, arena);

        for (i, part) in namespace.iter().enumerate() {
            bytes.extend_from_slice(part);
            if i < namespace.len() {
                // Add a separator after every part
                bytes.push(b'\\');
            }
        }

        bytes.extend_from_slice(name);

        bytes.leak()
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

            let a_full_name = join_item_name(f.arena, a.namespace, a.name);
            let b_full_name = join_item_name(f.arena, b.namespace, b.name);

            let mut a_iter = a_full_name.iter().map(u8::to_ascii_lowercase);
            let mut b_iter = b_full_name.iter().map(u8::to_ascii_lowercase);

            loop {
                match (a_iter.next(), b_iter.next()) {
                    (Some(ac), Some(bc)) => match ac.cmp(&bc) {
                        Ordering::Equal => {}
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

    let mut result_docs: BumpVec<'arena, Document<'arena, A>, A> = vec_in![f.arena];
    let grouped_items_count = grouped_items.len();
    for (index, group) in grouped_items.into_iter().enumerate() {
        let is_last_grouped_items = index + 1 == grouped_items_count;

        let group_count = group.len();
        for (item_index, item) in group.into_iter().enumerate() {
            let is_last_group = item_index + 1 == group_count;

            if should_expand {
                let mut parts = vec_in![f.arena;];
                parts.push(item.original_node.r#use.format(f));
                parts.push(Document::space());

                if let Some(ty) = item.use_type {
                    parts.push(ty.format(f));
                    parts.push(Document::space());
                }

                parts.push(Document::String(join_item_name(f.arena, item.namespace, item.name)));

                if let Some(alias) = item.alias {
                    parts.push(Document::space());
                    parts.push(Document::String(b"as "));
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
    namespace: &'arena [&'arena [u8]],
    name: &'arena [u8],
    alias: Option<&'arena [u8]>,
    original_node: &'arena Use<'arena>,
}

fn expand_use<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    use_stmt: &'arena Use<'arena>,
    should_expand: bool,
) -> std::vec::Vec<ExpandedUseItem<'arena>>
where
    A: Arena,
{
    let mut expanded_items = std::vec::Vec::new();

    /// Extract namespace and name from a `UseItem` by splitting its full name path.
    fn extract_namespace_and_name_from_item<'arena, A>(
        f: &mut FormatterState<'_, 'arena, A>,
        item: &'arena UseItem<'arena>,
        mut namespace: Vec<'arena, &'arena [u8], A>,
    ) -> (Vec<'arena, &'arena [u8], A>, &'arena [u8])
    where
        A: Arena,
    {
        let mut parts: Vec<'arena, &'arena [u8], A> =
            item.name.value().split(|&b| b == b'\\').collect_in::<Vec<'arena, _, A>>(f.arena);
        // SAFETY: split always returns at least one element
        let name = unsafe { parts.pop().unwrap_unchecked() };
        namespace.extend(parts);
        (namespace, name)
    }

    /// Extract namespace and name from a grouped list (`TypedList` or `MixedList`).
    /// The namespace is the list's namespace appended to the current namespace,
    /// and the name is extracted from the first item.
    fn extract_namespace_and_name_from_grouped_list<'arena, A>(
        list_namespace: &'arena [u8],
        first_item_name: Option<&'arena [u8]>,
        mut namespace: Vec<'arena, &'arena [u8], A>,
    ) -> (Vec<'arena, &'arena [u8], A>, &'arena [u8])
    where
        A: Arena,
    {
        namespace.push(list_namespace);
        let name = first_item_name.unwrap_or(b"");
        (namespace, name)
    }

    fn expand_items<'arena, A>(
        f: &mut FormatterState<'_, 'arena, A>,
        items: &'arena UseItems<'arena>,
        current_namespace: Vec<'arena, &'arena [u8], A>,
        use_type: Option<&'arena UseType<'arena>>,
        expanded_items: &mut std::vec::Vec<ExpandedUseItem<'arena>>,
        original_node: &'arena Use<'arena>,
        should_expand: bool,
    ) where
        A: Arena,
    {
        match items {
            UseItems::Sequence(seq) => {
                if should_expand {
                    for item in &seq.items {
                        expand_single_item(f, item, current_namespace.clone(), use_type, expanded_items, original_node);
                    }
                } else {
                    // Extract namespace and name from first item for sorting
                    let (namespace, name) = seq
                        .items
                        .first()
                        .map(|item| extract_namespace_and_name_from_item(f, item, current_namespace.clone()))
                        .unwrap_or((current_namespace, b""));
                    expanded_items.push(ExpandedUseItem {
                        use_type,
                        namespace: namespace.leak(),
                        name,
                        alias: None,
                        original_node,
                    });
                }
            }
            UseItems::TypedSequence(seq) => {
                if should_expand {
                    for item in &seq.items {
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
                    // Extract namespace and name from first item for sorting
                    let (namespace, name) = seq
                        .items
                        .first()
                        .map(|item| extract_namespace_and_name_from_item(f, item, current_namespace.clone()))
                        .unwrap_or((current_namespace, b""));
                    expanded_items.push(ExpandedUseItem {
                        use_type: Some(&seq.r#type),
                        namespace: namespace.leak(),
                        name,
                        alias: None,
                        original_node,
                    });
                }
            }
            UseItems::TypedList(list) => {
                if should_expand {
                    let mut new_namespace = current_namespace.clone();
                    new_namespace.push(list.namespace.value());
                    for item in &list.items {
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
                    // Extract namespace and name from first item for sorting
                    // For grouped items, the name should be just the item name (not a full path)
                    let (namespace, name) = extract_namespace_and_name_from_grouped_list(
                        list.namespace.value(),
                        list.items.first().map(|item| item.name.value()),
                        current_namespace,
                    );
                    expanded_items.push(ExpandedUseItem {
                        use_type: Some(&list.r#type),
                        namespace: namespace.leak(),
                        name,
                        alias: None,
                        original_node,
                    });
                }
            }
            UseItems::MixedList(list) => {
                if should_expand {
                    let mut new_namespace = current_namespace.clone();
                    new_namespace.push(list.namespace.value());
                    for maybe_typed_item in &list.items {
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
                    // Extract namespace and name from first item for sorting
                    // For grouped items, the name should be just the item name (not a full path)
                    let (namespace, name) = extract_namespace_and_name_from_grouped_list(
                        list.namespace.value(),
                        list.items.first().map(|item| item.item.name.value()),
                        current_namespace,
                    );
                    expanded_items.push(ExpandedUseItem {
                        use_type: list.items.first().and_then(|item| item.r#type.as_ref()),
                        namespace: namespace.leak(),
                        name,
                        alias: None,
                        original_node,
                    });
                }
            }
        }
    }

    fn expand_single_item<'arena, A>(
        f: &mut FormatterState<'_, 'arena, A>,
        item: &'arena UseItem<'arena>,
        mut current_namespace: Vec<'arena, &'arena [u8], A>,
        use_type: Option<&'arena UseType<'arena>>,
        expanded_items: &mut std::vec::Vec<ExpandedUseItem<'arena>>,
        original_node: &'arena Use<'arena>,
    ) where
        A: Arena,
    {
        let mut parts: Vec<'arena, &'arena [u8], A> =
            item.name.value().split(|&b| b == b'\\').collect_in::<Vec<'arena, _, A>>(f.arena);
        // SAFETY: split always returns at least one element
        let name = unsafe { parts.pop().unwrap_unchecked() };
        current_namespace.extend(parts);

        expanded_items.push(ExpandedUseItem {
            use_type,
            namespace: current_namespace.leak(),
            name,
            alias: item.alias.as_ref().map(|a| a.identifier.value),
            original_node,
        });
    }

    expand_items(f, &use_stmt.items, vec_in![f.arena], None, &mut expanded_items, use_stmt, should_expand); // Pass should_expand

    expanded_items
}

pub fn sort_use_items<'arena>(
    items: impl Iterator<Item = &'arena UseItem<'arena>>,
) -> std::vec::Vec<&'arena UseItem<'arena>> {
    let mut items = items.collect::<std::vec::Vec<_>>();
    items.sort_by_cached_key(|item| item.name.value().to_ascii_lowercase());
    items
}

pub fn sort_maybe_typed_use_items<'arena>(
    items: impl Iterator<Item = &'arena MaybeTypedUseItem<'arena>>,
) -> std::vec::Vec<&'arena MaybeTypedUseItem<'arena>> {
    let mut items = items.collect::<std::vec::Vec<_>>();
    items.sort_by_cached_key(|item| {
        let type_order = match &item.r#type {
            None => 0u8,
            Some(ty) => {
                if ty.is_function() {
                    1
                } else {
                    2
                }
            }
        };

        (type_order, item.item.name.value().to_ascii_lowercase())
    });

    items
}

fn calculate_statement_alignment<A>(
    f: &mut FormatterState<'_, '_, A>,
    stmt: &Statement<'_>,
    widths: &AlignmentWidths,
) -> AssignmentAlignment
where
    A: Arena,
{
    let current_name_width = match stmt {
        Statement::Expression(expr_stmt) => {
            if let Expression::Assignment(assign) = &expr_stmt.expression {
                let lhs_span = assign.lhs.span();

                // Skip multiline LHS expressions for alignment
                if has_new_line_in_range(f.source_text, lhs_span.start_offset(), lhs_span.end_offset()) {
                    0
                } else {
                    lhs_span.length() as usize
                }
            } else {
                0
            }
        }
        Statement::Constant(constant) => constant.items.iter().map(|item| item.name.value.len()).max().unwrap_or(0),
        _ => 0,
    };

    let name_padding = widths.name_width.saturating_sub(current_name_width);

    AssignmentAlignment { type_padding: 0, name_padding, break_group_id: None }
}
