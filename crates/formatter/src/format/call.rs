use mago_ast::*;

use crate::Formatter;
use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::format::Format;
use crate::parens::instantiation_needs_parens;

use super::call_arguments::print_argument_list;

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
    fn is_first_link_static_method_call(&self) -> bool {
        matches!(self.accesses.first(), Some(MemberAccess::StaticMethodCall(_)))
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
    f: &mut Formatter<'a>,
) -> Document<'a> {
    let base_document = member_access_chain.base.format(f);
    let mut parts = if base_needs_parerns(f, member_access_chain.base) {
        vec![Document::String("("), base_document, Document::String(")")]
    } else {
        vec![base_document]
    };

    let mut accesses_iter = member_access_chain.accesses.iter();

    // Handle the first method call
    if !f.settings.method_chain_breaking_style.is_next_line() || member_access_chain.is_first_link_static_method_call()
    {
        if let Some(first_chain_link) = accesses_iter.next() {
            // Format the base object and first method call together
            let (operator, method) = match first_chain_link {
                MemberAccess::PropertyAccess(c) => (Document::String("->"), c.property.format(f)),
                MemberAccess::NullSafePropertyAccess(c) => (Document::String("?->"), c.property.format(f)),
                MemberAccess::MethodCall(c) => (Document::String("->"), c.method.format(f)),
                MemberAccess::NullSafeMethodCall(c) => (Document::String("?->"), c.method.format(f)),
                MemberAccess::StaticMethodCall(c) => (Document::String("::"), c.method.format(f)),
            };

            parts.push(operator);
            parts.push(method);

            if let Some(argument_list) = first_chain_link.get_arguments_list() {
                parts.push(Document::Group(Group::new(vec![print_argument_list(f, argument_list)])));
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
                vec![Document::String("->"), c.property.format(f)]
            }
            MemberAccess::NullSafePropertyAccess(c) => {
                last_was_property = true;
                vec![Document::String("?->"), c.property.format(f)]
            }
            MemberAccess::MethodCall(c) => {
                last_was_property = false;
                vec![Document::String("->"), c.method.format(f)]
            }
            MemberAccess::NullSafeMethodCall(c) => {
                last_was_property = false;
                vec![Document::String("?->"), c.method.format(f)]
            }
            MemberAccess::StaticMethodCall(c) => {
                last_was_property = false;
                vec![Document::String("::"), c.method.format(f)]
            }
        });

        if let Some(argument_list) = chain_link.get_arguments_list() {
            contents.push(Document::Group(Group::new(vec![print_argument_list(f, argument_list)])));
        }

        parts.push(Document::Indent(contents));
    }

    if member_access_chain.is_first_link_static_method_call() {
        parts.push(Document::BreakParent);
    }

    // Wrap everything in a group to manage line breaking
    Document::Group(Group::new(parts))
}

fn base_needs_parerns(f: &Formatter<'_>, base: &Expression) -> bool {
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
