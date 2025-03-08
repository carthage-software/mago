use mago_ast::*;
use mago_span::Span;

use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::call_arguments::print_argument_list;
use crate::internal::parens::instantiation_needs_parens;
use crate::internal::utils::unwrap_parenthesized;

#[derive(Debug)]
pub(super) struct MemberAccessChain<'a> {
    pub base: &'a Expression,
    pub accesses: Vec<MemberAccess<'a>>,
}

#[derive(Debug)]
pub(super) enum MemberAccess<'a> {
    PropertyAccess(&'a PropertyAccess),
    NullSafePropertyAccess(&'a NullSafePropertyAccess),
    StaticMethodCall(&'a StaticMethodCall),
    MethodCall(&'a MethodCall),
    NullSafeMethodCall(&'a NullSafeMethodCall),
}

impl<'a> MemberAccess<'a> {
    pub fn get_arguments_list(&self) -> Option<&'a ArgumentList> {
        match self {
            MemberAccess::MethodCall(call) => Some(&call.argument_list),
            MemberAccess::NullSafeMethodCall(call) => Some(&call.argument_list),
            MemberAccess::StaticMethodCall(call) => Some(&call.argument_list),
            _ => None,
        }
    }
}

impl MemberAccessChain<'_> {
    #[inline]
    fn get_eligibility_score(&self) -> usize {
        let score = self
            .accesses
            .iter()
            .map(|access| match access {
                MemberAccess::PropertyAccess(_) | MemberAccess::NullSafePropertyAccess(_) => 1,
                MemberAccess::MethodCall(_)
                | MemberAccess::NullSafeMethodCall(_)
                | MemberAccess::StaticMethodCall(_) => 2,
            })
            .sum();

        match self.base {
            Expression::Instantiation(_) => score + 2,
            _ => score,
        }
    }

    #[inline]
    pub fn is_eligible_for_chaining(&self, f: &FormatterState) -> bool {
        let score = self.get_eligibility_score();
        let threshold = 'threshold: {
            match self.base {
                Expression::Call(Call::Function(function_call)) => {
                    if function_call.argument_list.arguments.len() == 1
                        && matches!(self.accesses.last(), Some(MemberAccess::MethodCall(MethodCall { argument_list, .. }) | MemberAccess::NullSafeMethodCall(NullSafeMethodCall { argument_list, .. })) if !argument_list.arguments.is_empty())
                    {
                        2
                    } else {
                        4
                    }
                }
                Expression::Variable(Variable::Direct(_))
                | Expression::Identifier(_)
                | Expression::Instantiation(_) => {
                    // Check if the last access is a method call with arguments
                    let Some(
                        MemberAccess::MethodCall(MethodCall { argument_list, .. })
                        | MemberAccess::NullSafeMethodCall(NullSafeMethodCall { argument_list, .. }),
                    ) = self.accesses.last()
                    else {
                        break 'threshold 8;
                    };

                    // Check argument list length
                    if argument_list.arguments.len() > 1 {
                        break 'threshold 8;
                    }

                    match &self.base {
                        Expression::Variable(Variable::Direct(v)) => {
                            if f.interner.lookup(&v.name).len() > 5 {
                                4
                            } else {
                                5
                            }
                        }
                        Expression::Identifier(_) => 6,
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
    fn must_break(&self, f: &FormatterState) -> bool {
        if self.is_first_link_static_method_call() {
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
                f.interner.lookup(&variable.name) == "$this" && self.accesses.len() > 3
            }
            _ => false,
        };

        if must_break || !f.settings.preserve_breaking_member_access_chain {
            return must_break;
        }

        for link in &self.accesses {
            let span = match link {
                MemberAccess::PropertyAccess(c) => c.arrow,
                MemberAccess::NullSafePropertyAccess(c) => c.question_mark_arrow,
                MemberAccess::MethodCall(c) => c.arrow,
                MemberAccess::NullSafeMethodCall(c) => c.question_mark_arrow,
                MemberAccess::StaticMethodCall(c) => c.double_colon,
            };

            if f.has_newline(span.start.offset, /* backwards */ true) {
                return true;
            }
        }

        false
    }

    #[inline]
    fn find_fluent_access_chain_start(&self) -> Option<usize> {
        let mut p_count = 0;
        let mut pm_count = 0;
        let mut last_was_p = false;
        let mut pattern_start_index = None;

        for (i, access) in self.accesses.iter().enumerate() {
            match access {
                MemberAccess::PropertyAccess(_) | MemberAccess::NullSafePropertyAccess(_) => {
                    p_count += 1;
                    last_was_p = true;
                }
                MemberAccess::MethodCall(_) | MemberAccess::NullSafeMethodCall(_) => {
                    if last_was_p {
                        pm_count += 1;
                        if pattern_start_index.is_none() {
                            pattern_start_index = Some(i - 1);
                        }
                    } else {
                        pm_count = 0;
                    }

                    last_was_p = false;
                }
                _ => {
                    last_was_p = false;
                }
            }
        }

        if pm_count >= (p_count - pm_count) && pm_count > 0 && !last_was_p {
            return pattern_start_index;
        }

        None
    }
}

pub(super) fn collect_member_access_chain(expr: &Expression) -> Option<MemberAccessChain<'_>> {
    let expr = unwrap_parenthesized(expr);

    let mut member_access = Vec::new();
    let mut current_expr = expr;

    loop {
        match current_expr {
            Expression::Call(Call::StaticMethod(static_method_call)) if !member_access.is_empty() => {
                member_access.push(MemberAccess::StaticMethodCall(static_method_call));

                current_expr = &static_method_call.class;

                break;
            }
            Expression::Access(Access::Property(property_access)) => {
                member_access.push(MemberAccess::PropertyAccess(property_access));

                current_expr = &property_access.object;
            }
            Expression::Access(Access::NullSafeProperty(null_safe_property_access)) => {
                member_access.push(MemberAccess::NullSafePropertyAccess(null_safe_property_access));

                current_expr = &null_safe_property_access.object;
            }
            Expression::Call(Call::Method(method_call)) => {
                member_access.push(MemberAccess::MethodCall(method_call));

                current_expr = &method_call.object;
            }
            Expression::Call(Call::NullSafeMethod(null_safe_method_call)) => {
                member_access.push(MemberAccess::NullSafeMethodCall(null_safe_method_call));

                current_expr = &null_safe_method_call.object;
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

pub(super) fn print_member_access_chain<'a>(
    member_access_chain: &MemberAccessChain<'a>,
    f: &mut FormatterState<'a>,
) -> Document<'a> {
    let base_document = member_access_chain.base.format(f);
    let mut parts = if base_needs_parerns(f, member_access_chain.base) {
        vec![Document::String("("), base_document, Document::String(")")]
    } else {
        vec![base_document]
    };

    let mut accesses_iter = member_access_chain.accesses.iter();

    // Handle the first method call
    if !f.settings.method_chain_breaking_style.is_next_line()
        || member_access_chain.is_first_link_static_method_call()
        || matches!(member_access_chain.base, Expression::Variable(Variable::Direct(variable)) if f.interner.lookup(&variable.name) == "$this")
    {
        if let Some(first_chain_link) = accesses_iter.next() {
            // Format the base object and first method call together
            let (operator, method) = match first_chain_link {
                MemberAccess::PropertyAccess(c) => (format_op(f, c.arrow, "->"), c.property.format(f)),
                MemberAccess::NullSafePropertyAccess(c) => {
                    (format_op(f, c.question_mark_arrow, "?->"), c.property.format(f))
                }
                MemberAccess::MethodCall(c) => (format_op(f, c.arrow, "->"), c.method.format(f)),
                MemberAccess::NullSafeMethodCall(c) => (format_op(f, c.question_mark_arrow, "?->"), c.method.format(f)),
                MemberAccess::StaticMethodCall(c) => (format_op(f, c.double_colon, "::"), c.method.format(f)),
            };

            parts.push(operator);
            parts.push(method);

            if let Some(argument_list) = first_chain_link.get_arguments_list() {
                parts.push(Document::Group(Group::new(vec![print_argument_list(f, argument_list, false)])));
            }
        }
    }

    let fluent_access_chain_start = member_access_chain.find_fluent_access_chain_start();
    let mut last_was_property = false;

    // Now handle the remaining method calls
    for (i, chain_link) in accesses_iter.enumerate() {
        let is_in_fluent_chain = fluent_access_chain_start.is_some_and(|start| i >= start);

        let mut contents = if !is_in_fluent_chain || !last_was_property {
            vec![Document::Line(Line::soft())]
        } else {
            vec![] // No newline if in fluent chain and last was property
        };

        contents.extend(match chain_link {
            MemberAccess::PropertyAccess(c) => {
                last_was_property = true;

                [format_op(f, c.arrow, "->"), c.property.format(f)]
            }
            MemberAccess::NullSafePropertyAccess(c) => {
                last_was_property = true;
                [format_op(f, c.question_mark_arrow, "?->"), c.property.format(f)]
            }
            MemberAccess::MethodCall(c) => {
                last_was_property = false;
                [format_op(f, c.arrow, "->"), c.method.format(f)]
            }
            MemberAccess::NullSafeMethodCall(c) => {
                last_was_property = false;
                [format_op(f, c.question_mark_arrow, "?->"), c.method.format(f)]
            }
            MemberAccess::StaticMethodCall(c) => {
                last_was_property = false;
                [format_op(f, c.double_colon, "::"), c.method.format(f)]
            }
        });

        if let Some(argument_list) = chain_link.get_arguments_list() {
            contents.push(Document::Group(Group::new(vec![print_argument_list(f, argument_list, false)])));
        }

        parts.push(Document::Indent(contents));
    }

    if member_access_chain.must_break(f) {
        parts.push(Document::BreakParent);
    }

    // Wrap everything in a group to manage line breaking
    Document::Group(Group::new(parts))
}

fn base_needs_parerns(f: &FormatterState<'_>, base: &Expression) -> bool {
    if let Expression::Parenthesized(parenthesized) = base {
        return base_needs_parerns(f, &parenthesized.expression);
    }

    match base {
        Expression::Instantiation(instantiation) => instantiation_needs_parens(f, instantiation),
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

fn format_op<'a>(f: &mut FormatterState<'a>, span: Span, operator: &'a str) -> Document<'a> {
    let leading = f.print_leading_comments(span);
    let doc = Document::String(operator);
    let doc = f.print_comments(leading, doc, None);

    doc
}
