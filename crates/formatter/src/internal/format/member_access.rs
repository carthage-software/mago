use bumpalo::collections::Vec;
use bumpalo::vec;

use bumpalo::Bump;
use mago_database::file::HasFileId;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::call_arguments::print_argument_list;
use crate::internal::format::misc;
use crate::internal::format::misc::is_breaking_expression;
use crate::internal::format::misc::is_simple_expression;
use crate::internal::format::misc::is_string_word_type;
use crate::internal::utils::string_width;
use crate::internal::utils::unwrap_parenthesized;

#[derive(Debug)]
pub(super) struct MemberAccessChain<'arena> {
    pub base: &'arena Expression<'arena>,
    pub accesses: Vec<'arena, MemberAccess<'arena>>,
}

#[derive(Debug)]
pub(super) enum MemberAccess<'arena> {
    PropertyAccess(&'arena PropertyAccess<'arena>),
    NullSafePropertyAccess(&'arena NullSafePropertyAccess<'arena>),
    StaticMethodCall(&'arena StaticMethodCall<'arena>),
    MethodCall(&'arena MethodCall<'arena>),
    NullSafeMethodCall(&'arena NullSafeMethodCall<'arena>),
}

impl<'arena> MemberAccess<'arena> {
    #[inline]
    const fn is_property_access(&self) -> bool {
        matches!(self, MemberAccess::PropertyAccess(_) | MemberAccess::NullSafePropertyAccess(_))
    }

    #[inline]
    const fn get_operator_as_str(&self) -> &'static str {
        match self {
            MemberAccess::PropertyAccess(_) | MemberAccess::MethodCall(_) => "->",
            MemberAccess::NullSafePropertyAccess(_) | MemberAccess::NullSafeMethodCall(_) => "?->",
            MemberAccess::StaticMethodCall(_) => "::",
        }
    }

    #[inline]
    const fn get_operator_span(&self) -> Span {
        match self {
            MemberAccess::PropertyAccess(c) => c.arrow,
            MemberAccess::NullSafePropertyAccess(c) => c.question_mark_arrow,
            MemberAccess::MethodCall(c) => c.arrow,
            MemberAccess::NullSafeMethodCall(c) => c.question_mark_arrow,
            MemberAccess::StaticMethodCall(c) => c.double_colon,
        }
    }

    #[inline]
    const fn get_selector(&self) -> &'arena ClassLikeMemberSelector<'arena> {
        match self {
            MemberAccess::PropertyAccess(c) => &c.property,
            MemberAccess::NullSafePropertyAccess(c) => &c.property,
            MemberAccess::MethodCall(c) => &c.method,
            MemberAccess::NullSafeMethodCall(c) => &c.method,
            MemberAccess::StaticMethodCall(c) => &c.method,
        }
    }

    #[inline]
    fn get_arguments_list(&self) -> Option<&'arena ArgumentList<'arena>> {
        match self {
            MemberAccess::MethodCall(call) => Some(&call.argument_list),
            MemberAccess::NullSafeMethodCall(call) => Some(&call.argument_list),
            MemberAccess::StaticMethodCall(call) => Some(&call.argument_list),
            _ => None,
        }
    }
}

impl<'arena> MemberAccessChain<'arena> {
    #[inline]
    fn get_eligibility_score(&self, f: &FormatterState<'_, 'arena>) -> usize {
        let mut score: i32 = 0;
        let mut account_for_simple_calls = true;
        let mut always_account_for_simple_calls = false;

        match self.base {
            Expression::Instantiation(_) => {
                score += 2; // Instantiation adds extra score
            }
            Expression::Call(Call::Function(FunctionCall { argument_list, .. }))
                if argument_list.arguments.len() == 1 =>
            {
                let (is_breaking_argument, is_string_word_type) = match argument_list.arguments.first() {
                    Some(argument) => {
                        (is_breaking_expression(f, argument.value(), true), is_string_word_type(argument.value()))
                    }
                    None => (false, false),
                };

                if is_string_word_type {
                    always_account_for_simple_calls = true;
                } else if is_breaking_argument {
                    score -= 1;
                }
            }
            _ => {
                // For other base expressions, we don't add extra score
            }
        }

        if self.accesses.len() <= 2 && always_account_for_simple_calls {
            score -= 1;
        }

        for member_access in &self.accesses {
            let (method_selector, argument_list) = match member_access {
                MemberAccess::PropertyAccess(_) | MemberAccess::NullSafePropertyAccess(_) => {
                    score += 1;

                    continue;
                }
                MemberAccess::MethodCall(MethodCall { method, argument_list, .. })
                | MemberAccess::NullSafeMethodCall(NullSafeMethodCall { method, argument_list, .. })
                | MemberAccess::StaticMethodCall(StaticMethodCall { method, argument_list, .. }) => {
                    (method, argument_list)
                }
            };

            // Arbitrarily reduce score for very short method names
            // see: https://github.com/carthage-software/mago/issues/334
            if let ClassLikeMemberSelector::Identifier(method_name) = method_selector {
                if method_name.value.len() <= 2 {
                    score -= 2;
                } else if method_name.value.len() == 3 {
                    score -= 1;
                }
            }

            if (!account_for_simple_calls && !always_account_for_simple_calls) || argument_list.arguments.is_empty() {
                score += 2;
                account_for_simple_calls = false;

                continue;
            }

            let breaking_arguments = if argument_list.arguments.len() == 1 {
                argument_list
                    .arguments
                    .first()
                    .map(|argument| argument.value())
                    .is_some_and(|e| is_breaking_expression(f, e, true))
            } else {
                argument_list.arguments.iter().all(|arg| !arg.is_positional())
                    && argument_list.arguments.iter().any(|arg| is_breaking_expression(f, arg.value(), false))
            };

            if breaking_arguments {
                score += 1;
            } else {
                score += 2;
                account_for_simple_calls = false;
            }
        }

        if self.accesses.len() >= 6 {
            score += 3;
        }

        if f.in_condition {
            // In conditions, we lower the score to avoid breaking chains too eagerly
            score -= 3;
        }

        score.max(0) as usize
    }

    #[inline]
    pub fn is_eligible_for_chaining(&self, f: &FormatterState<'_, 'arena>) -> bool {
        if self.has_comments_in_chain(f) {
            return true;
        }

        if f.settings.preserve_breaking_member_access_chain && self.is_already_broken(f) {
            return true;
        }

        let score = self.get_eligibility_score(f);
        let threshold = 'threshold: {
            match self.base {
                Expression::Call(Call::Function(function_call)) => {
                    if function_call.argument_list.arguments.len() == 1
                        && let Some(first_argument) = function_call.argument_list.arguments.first()
                        && matches!(self.accesses.last(), Some(MemberAccess::MethodCall(MethodCall { argument_list, .. }) | MemberAccess::NullSafeMethodCall(NullSafeMethodCall { argument_list, .. })) if !argument_list.arguments.is_empty())
                    {
                        if !matches!(function_call.function, Expression::Identifier(Identifier::Local(_)))
                            || !first_argument.is_positional()
                            || !is_simple_expression(first_argument.value())
                        {
                            2
                        } else {
                            3
                        }
                    } else {
                        4
                    }
                }
                Expression::Variable(Variable::Direct(_))
                | Expression::Identifier(_)
                | Expression::ConstantAccess(_)
                | Expression::Instantiation(_) => {
                    // Check if the last access is a method call with arguments
                    let Some(
                        MemberAccess::MethodCall(MethodCall { argument_list, .. })
                        | MemberAccess::NullSafeMethodCall(NullSafeMethodCall { argument_list, .. }),
                    ) = self.accesses.last()
                    else {
                        break 'threshold 6;
                    };

                    // Check argument list length
                    if argument_list.arguments.len() > 1 {
                        break 'threshold 6;
                    }

                    match &self.base {
                        Expression::Variable(Variable::Direct(v)) => {
                            if string_width(v.name) > 5 {
                                4
                            } else {
                                5
                            }
                        }
                        Expression::Identifier(_) | Expression::ConstantAccess(_) => 6,
                        Expression::Instantiation(_) => 6,
                        _ => unreachable!(), // We already matched these variants
                    }
                }
                _ => 4,
            }
        };

        score >= threshold
    }

    #[inline]
    fn is_first_link_static_method_call(&self) -> bool {
        matches!(self.accesses.first(), Some(MemberAccess::StaticMethodCall(_)))
    }

    #[inline]
    fn is_already_broken(&self, f: &FormatterState) -> bool {
        let mut accesses_len = self.accesses.len();
        if should_inline_first_access(f, self) {
            accesses_len -= 1; // First access is inline, so we don't count it for broken links
        }

        let mut broken_links = 0;
        if let Some(first_access) = self.accesses.first() {
            let base_end = self.base.span().end;
            let first_op_start = first_access.get_operator_span().start;

            if misc::has_new_line_in_range(f.source_text, base_end.offset, first_op_start.offset) {
                broken_links += 1;
            }
        }

        for (i, access) in self.accesses.iter().enumerate() {
            if i == 0 {
                continue; // Skip the first access since we need previous selector
            }

            let prev_access = &self.accesses[i - 1];
            let prev_selector = prev_access.get_selector();
            let prev_selector_end = match prev_access.get_arguments_list() {
                Some(args) => args.span().end,
                None => prev_selector.span().end,
            };

            let current_op_span = access.get_operator_span();

            if misc::has_new_line_in_range(f.source_text, prev_selector_end.offset, current_op_span.start.offset) {
                broken_links += 1;
            }
        }

        if broken_links == 0 {
            return false;
        }

        if broken_links >= accesses_len || broken_links >= 2 {
            return true;
        }

        if accesses_len <= 3 {
            // we must have at least 2 breaks for chains of length 3 or less
            return false;
        }

        true
    }

    #[inline]
    fn has_comments_in_chain(&self, f: &FormatterState) -> bool {
        // Check if there are comments after the base expression
        if let Some(first_access) = self.accesses.first() {
            let base_end = self.base.span().end;
            let first_op_start = first_access.get_operator_span().start;

            // Check for comments between base and first operator
            if f.has_inner_comment(Span::new(f.file_id(), base_end, first_op_start)) {
                return true;
            }
        }

        // Check for comments between chain elements
        for (i, access) in self.accesses.iter().enumerate() {
            if i == 0 {
                continue; // Skip the first access as we already checked between base and it
            }

            let prev_access = &self.accesses[i - 1];
            let prev_selector = prev_access.get_selector();
            let prev_span = match prev_access.get_arguments_list() {
                Some(args) => args.span(),
                None => prev_selector.span(),
            };

            let current_op_start = access.get_operator_span().start;

            // Check for comments between previous selector/args and current operator
            if f.has_inner_comment(Span::new(prev_span.file_id, prev_span.end, current_op_start)) {
                return true;
            }

            // Check for comments between operator and selector
            if f.has_inner_comment(Span::new(
                access.get_operator_span().file_id,
                access.get_operator_span().end_position(),
                access.get_selector().span().start_position(),
            )) {
                return true;
            }
        }

        false
    }

    #[inline]
    fn must_break(&self, f: &FormatterState) -> bool {
        if self.is_first_link_static_method_call() && self.accesses.len() > 3 {
            return true;
        }

        if let Expression::Variable(Variable::Direct(variable)) = self.base
            && variable.name.eq_ignore_ascii_case("$builder")
            && self.accesses.len() >= 3
            && self.accesses.iter().all(|access| {
                let id = match access {
                    MemberAccess::MethodCall(MethodCall {
                        method: ClassLikeMemberSelector::Identifier(id), ..
                    })
                    | MemberAccess::NullSafeMethodCall(NullSafeMethodCall {
                        method: ClassLikeMemberSelector::Identifier(id),
                        ..
                    }) => id,
                    _ => return false,
                };

                id.value.starts_with("add") || id.value.starts_with("with")
            })
        {
            return true;
        }

        let must_break = match self.base {
            Expression::Instantiation(_) => {
                self.accesses.iter().all(|access| {
                    matches!(access, MemberAccess::MethodCall(_) | MemberAccess::NullSafeMethodCall(_))
                }) && self.accesses.iter().any(|access| {
                    matches!(access, MemberAccess::MethodCall(MethodCall { argument_list, .. }) | MemberAccess::NullSafeMethodCall(NullSafeMethodCall { argument_list, .. }) if !argument_list.arguments.is_empty())
                })
            }
            Expression::Variable(Variable::Direct(variable)) => {
                variable.name == "$this" && self.accesses.len() > 3
            }
            _ => false,
        };

        if must_break || !f.settings.preserve_breaking_member_access_chain {
            return must_break;
        }

        self.is_already_broken(f)
    }

    #[inline]
    fn find_fluent_access_chain_start(&self) -> Option<usize> {
        // If empty, return None
        if self.accesses.is_empty() {
            return None;
        }

        let mut i = 0;
        let mut pattern_start_index = None;
        let mut patterns_count = 0;

        // Iterate through all accesses
        while i < self.accesses.len() {
            // Count consecutive property accesses
            let property_start = i;
            let mut property_count = 0;

            while i < self.accesses.len()
                && matches!(self.accesses[i], MemberAccess::PropertyAccess(_) | MemberAccess::NullSafePropertyAccess(_))
            {
                property_count += 1;
                i += 1;
            }

            // Skip if no properties found
            if property_count == 0 {
                i += 1;
                continue;
            }

            // Count consecutive method calls
            let mut method_count = 0;

            while i < self.accesses.len()
                && matches!(self.accesses[i], MemberAccess::MethodCall(_) | MemberAccess::NullSafeMethodCall(_))
            {
                method_count += 1;
                i += 1;
            }

            // If we found at least one property access followed by at least one method call,
            // consider it a valid pattern and record the start position
            if property_count > 0 && method_count > 0 {
                if pattern_start_index.is_none() {
                    pattern_start_index = Some(property_start);
                }

                patterns_count += 1;
            }
        }

        match pattern_start_index {
            Some(0) if patterns_count > 1 => Some(0),
            Some(start) if start > 0 => Some(start),
            _ => None,
        }
    }
}

pub(super) fn collect_member_access_chain<'arena>(
    arena: &'arena Bump,
    expr: &'arena Expression<'arena>,
) -> Option<MemberAccessChain<'arena>> {
    let expr = unwrap_parenthesized(expr);

    let mut member_access: Vec<'arena, MemberAccess<'arena>> = Vec::new_in(arena);
    let mut current_expr = expr;

    loop {
        match current_expr {
            Expression::Call(Call::StaticMethod(static_method_call)) if !member_access.is_empty() => {
                member_access.push(MemberAccess::StaticMethodCall(static_method_call));

                current_expr = unwrap_parenthesized(static_method_call.class);

                break;
            }
            Expression::Access(Access::Property(property_access)) => {
                member_access.push(MemberAccess::PropertyAccess(property_access));

                current_expr = unwrap_parenthesized(property_access.object);
            }
            Expression::Access(Access::NullSafeProperty(null_safe_property_access)) => {
                member_access.push(MemberAccess::NullSafePropertyAccess(null_safe_property_access));

                current_expr = unwrap_parenthesized(null_safe_property_access.object);
            }
            Expression::Call(Call::Method(method_call)) => {
                member_access.push(MemberAccess::MethodCall(method_call));

                current_expr = unwrap_parenthesized(method_call.object);
            }
            Expression::Call(Call::NullSafeMethod(null_safe_method_call)) => {
                member_access.push(MemberAccess::NullSafeMethodCall(null_safe_method_call));

                current_expr = unwrap_parenthesized(null_safe_method_call.object);
            }
            _ => {
                break;
            }
        }
    }

    if member_access.is_empty() {
        None
    } else {
        member_access.reverse();

        Some(MemberAccessChain { base: current_expr, accesses: member_access })
    }
}

pub(super) fn print_member_access_chain<'arena>(
    member_access_chain: &MemberAccessChain<'arena>,
    f: &mut FormatterState<'_, 'arena>,
) -> Document<'arena> {
    let base_document = member_access_chain.base.format(f);
    let mut parts = if base_needs_parens(f, member_access_chain.base) {
        vec![in f.arena; Document::String("("), base_document, Document::String(")")]
    } else {
        vec![in f.arena; base_document]
    };

    let mut accesses_iter = member_access_chain.accesses.iter().enumerate().peekable();
    let fluent_access_chain_start = member_access_chain.find_fluent_access_chain_start();
    let must_break = member_access_chain.must_break(f);
    let group_id = f.next_id();

    let mut last_element_end = member_access_chain.base.span().end;
    // Handle the first access

    if should_inline_first_access(f, member_access_chain)
        && let Some((_, first_chain_link)) = accesses_iter.next()
    {
        // Format the base object and first method call together
        parts.push(format_access_operator(
            f,
            first_chain_link.get_operator_span(),
            first_chain_link.get_operator_as_str(),
        ));

        let selector = first_chain_link.get_selector();
        parts.push(selector.format(f));
        last_element_end = selector.span().end;
        if let Some(argument_list) = first_chain_link.get_arguments_list() {
            let mut formatted_argument_list = vec![in f.arena; print_argument_list(f, argument_list, false, true)];
            if let Some(comments) = f.print_trailing_comments(argument_list.span()) {
                formatted_argument_list.push(comments);
            }

            parts.push(Document::Group(Group::new(formatted_argument_list)));
            last_element_end = argument_list.span().end;
        }
    }

    let mut should_reset = false;
    // Now handle the remaining method calls
    while let Some((i, chain_link)) = accesses_iter.next() {
        let is_in_fluent_chain = fluent_access_chain_start.is_some_and(|start| i >= start);

        let must_have_new_line = if !is_in_fluent_chain || should_reset || i == 0 {
            should_reset = false;
            true
        } else {
            f.has_inner_comment(Span::new(f.file_id(), last_element_end, chain_link.get_operator_span().start))
        };

        let mut contents = if must_have_new_line {
            if must_break {
                vec![in f.arena; Document::Line(Line::hard())]
            } else {
                vec![in f.arena; Document::Line(Line::soft())]
            }
        } else {
            vec![in f.arena;] // No newline if in fluent chain and last was property
        };

        contents.push(format_access_operator(f, chain_link.get_operator_span(), chain_link.get_operator_as_str()));
        let selector = chain_link.get_selector();
        last_element_end = selector.span().end;
        contents.push(selector.format(f));
        if let Some(argument_list) = chain_link.get_arguments_list() {
            let mut formatted_argument_list = vec![in f.arena; print_argument_list(f, argument_list, false, true)];
            if let Some(comments) = f.print_trailing_comments(argument_list.span()) {
                formatted_argument_list.push(comments);
            }

            contents.push(Document::Group(Group::new(formatted_argument_list)));
            last_element_end = argument_list.span().end;
        }

        if must_break {
            parts.push(Document::Indent(contents));
        } else {
            parts.push(Document::IndentIfBreak(IndentIfBreak::new(group_id, contents)));
        }

        let is_next_property = accesses_iter.peek().is_some_and(|(_, next)| next.is_property_access());
        if !chain_link.is_property_access() && is_next_property {
            should_reset = true;
        }
    }

    if must_break && !matches!(f.parent_node(), Node::Binary(_)) {
        parts.push(Document::BreakParent);
    }

    // Wrap everything in a group to manage line breaking
    Document::Group(Group::new(parts).with_id(group_id))
}

fn should_inline_first_access<'arena>(
    f: &FormatterState<'_, 'arena>,
    member_access_chain: &MemberAccessChain<'arena>,
) -> bool {
    if member_access_chain.find_fluent_access_chain_start().is_some_and(|start| start == 0) {
        return false;
    }

    if !f.settings.method_chain_breaking_style.is_next_line() {
        return true;
    }

    if member_access_chain.is_first_link_static_method_call() {
        return true;
    }

    if let Expression::Variable(Variable::Direct(variable)) = member_access_chain.base
        && variable.name == "$this"
    {
        return true;
    }

    false
}

fn base_needs_parens<'arena>(f: &FormatterState<'_, 'arena>, base: &'arena Expression<'arena>) -> bool {
    if let Expression::Parenthesized(parenthesized) = base {
        return base_needs_parens(f, parenthesized.expression);
    }

    match base {
        Expression::Instantiation(instantiation) => f.instantiation_needs_parens(instantiation),
        Expression::Binary(_)
        | Expression::UnaryPrefix(_)
        | Expression::UnaryPostfix(_)
        | Expression::Assignment(_)
        | Expression::Conditional(_)
        | Expression::AnonymousClass(_)
        | Expression::Closure(_)
        | Expression::ArrowFunction(_)
        | Expression::Match(_)
        | Expression::Yield(_)
        | Expression::Clone(_) => true,
        _ => false,
    }
}

pub(super) fn format_access_operator<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    span: Span,
    operator: &'arena str,
) -> Document<'arena> {
    let leading = f.print_leading_comments(span);
    let doc = Document::String(operator);

    f.print_comments(leading, doc, None)
}
