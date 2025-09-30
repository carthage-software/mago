use bumpalo::vec;

use mago_span::*;
use mago_syntax::ast::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::binaryish::should_inline_binary_expression;
use crate::internal::format::member_access::collect_member_access_chain;
use crate::internal::format::misc::is_breaking_expression;
use crate::internal::format::misc::is_simple_expression;
use crate::internal::utils::string_width;
use crate::internal::utils::unwrap_parenthesized;

/// Represents nodes in the Abstract Syntax Tree (AST) that involve assignment-like operations.
#[derive(Debug, Clone, Copy)]
pub(super) enum AssignmentLikeNode<'arena> {
    /// Represents a standard assignment operation, such as `$a = $b`.
    AssignmentOperation(&'arena Assignment<'arena>),

    /// Represents a class-like constant item.
    ///
    /// - `A = 1` in `class A { public const A = 1; }`.
    ClassLikeConstantItem(&'arena ClassLikeConstantItem<'arena>),

    /// Represents a global constant item.
    ///
    /// - `A = 1` in `const A = 1;`.
    ConstantItem(&'arena ConstantItem<'arena>),

    /// Represents a backed enum case item.
    ///
    /// - `A = 1` in `enum A: int { case A = 1; }`.
    EnumCaseBackedItem(&'arena EnumCaseBackedItem<'arena>),

    /// Represents a property declaration with an initializer in a class.
    ///
    /// - `$foo = 1` in `class A { public int $foo = 1; }`.
    PropertyConcreteItem(&'arena PropertyConcreteItem<'arena>),

    /// Represents a key-value pair in an array, list, or similar structure.
    ///
    /// - `$a => $b` in `[ $a => $b ]`
    /// - `$a => $b` in `array($a => $b)`
    /// - `$a => $b` in `list($a => $b)`
    KeyValueArrayElement(&'arena KeyValueArrayElement<'arena>),

    /// Represents a yield pair in a generator function.
    ///
    /// - `yield $a => $b` in `function gen() { yield $a => $b; }`
    YieldPair(&'arena YieldPair<'arena>),
}

#[derive(Debug)]
enum Layout {
    Chain,
    ChainTailArrowChain,
    ChainTail,
    BreakAfterOperator,
    NeverBreakAfterOperator,
    BreakLhs,
    Fluid,
}

pub(super) fn print_assignment<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    assignment_node: AssignmentLikeNode<'arena>,
    lhs: Document<'arena>,
    operator: Document<'arena>,
    rhs_expression: &'arena Expression<'arena>,
) -> Document<'arena> {
    let layout = choose_layout(f, &lhs, &assignment_node, rhs_expression);
    let rhs = rhs_expression.format(f);

    match layout {
        Layout::Chain => Document::Array(vec![
            in f.arena;
            Document::Group(Group::new(vec![in f.arena; lhs])),
            Document::space(),
            operator,
            Document::Line(Line::default()),
            rhs,
        ]),
        Layout::ChainTailArrowChain => Document::Array(vec![
            in f.arena;
            Document::Group(Group::new(vec![in f.arena; lhs])),
            Document::space(),
            operator,
            rhs,
        ]),
        Layout::ChainTail => Document::Group(Group::new(vec![
            in f.arena;
            lhs,
            Document::space(),
            operator,
            Document::Indent(vec![in f.arena; Document::Line(Line::hard()), rhs]),
        ])),
        Layout::BreakAfterOperator => {
            let id = f.next_id();

            Document::Group(
                Group::new(vec![
                    in f.arena;
                    Document::Group(Group::new(vec![in f.arena; lhs])),
                    Document::space(),
                    operator,
                    Document::IndentIfBreak(IndentIfBreak::new(id, vec![
                        in f.arena;
                        Document::Line(Line::default()),
                        rhs,
                    ])),
                ])
                .with_id(id),
            )
        }
        Layout::NeverBreakAfterOperator => Document::Group(Group::new(vec![
            in f.arena;
            Document::Group(Group::new(vec![in f.arena; lhs])),
            Document::space(),
            operator,
            Document::space(),
            Document::Group(Group::new(vec![in f.arena; rhs])),
        ])),
        Layout::BreakLhs => Document::Group(Group::new(vec![
            in f.arena;
            lhs,
            Document::space(),
            operator,
            Document::space(),
            Document::Group(Group::new(vec![in f.arena; rhs])),
        ])),
        Layout::Fluid => {
            let assignment_id = f.next_id();

            Document::Group(Group::new(vec![
                in f.arena;
                lhs,
                Document::space(),
                operator,
                Document::Group(
                    Group::new(vec![
                        in f.arena;
                        Document::Indent(vec![in f.arena; Document::Line(Line::default())])
                    ]).with_id(assignment_id),
                ),
                Document::IndentIfBreak(IndentIfBreak::new(assignment_id, vec![in f.arena; rhs])),
            ]))
        }
    }
}

fn choose_layout<'arena>(
    f: &FormatterState<'_, 'arena>,
    lhs: &Document<'arena>,
    assignment_like_node: &AssignmentLikeNode<'arena>,
    rhs_expression: &'arena Expression<'arena>,
) -> Layout {
    if let Expression::Parenthesized(parenthesized) = rhs_expression {
        return choose_layout(f, lhs, assignment_like_node, parenthesized.expression);
    }

    let is_tail = !is_assignment(rhs_expression);

    let should_use_chain_formatting = matches!(assignment_like_node, AssignmentLikeNode::AssignmentOperation(_))
        && matches!(f.parent_node(), Node::Assignment(_))
        && (!is_tail || !matches!(f.grandparent_node(), Some(Node::ExpressionStatement(_))));

    if should_use_chain_formatting {
        if !is_tail {
            return Layout::Chain;
        } else if let Expression::ArrowFunction(arrow_function) = rhs_expression
            && let Expression::ArrowFunction(_) = arrow_function.expression
        {
            return Layout::ChainTailArrowChain;
        }

        return Layout::ChainTail;
    }

    if !is_tail || f.has_leading_own_line_comment(rhs_expression.span()) {
        return Layout::BreakAfterOperator;
    }

    if let Expression::Construct(construct) = rhs_expression
        && matches!(
            construct,
            Construct::Require(_) | Construct::RequireOnce(_) | Construct::Include(_) | Construct::IncludeOnce(_)
        )
    {
        // special case for require/include constructs.
        return Layout::NeverBreakAfterOperator;
    }

    if let Expression::Binary(binary) = rhs_expression {
        if is_member_chain_or_single_arg_call(f, binary.lhs) && is_simple_expression(binary.rhs) {
            return Layout::NeverBreakAfterOperator;
        }

        if is_breaking_expression(f, binary.rhs, false) {
            return Layout::NeverBreakAfterOperator;
        }
    }

    if let Expression::Binary(Binary { lhs, rhs, .. }) = rhs_expression
        && is_member_chain_or_single_arg_call(f, lhs)
        && is_simple_expression(rhs)
    {
        return Layout::NeverBreakAfterOperator;
    }

    let can_break_left_doc = lhs.can_break();
    if is_complex_destructuring(assignment_like_node)
        || (is_arrow_function_variable_declarator(assignment_like_node) && can_break_left_doc)
    {
        return Layout::BreakLhs;
    }

    // wrapping class property-like with very short keys usually doesn't add much value
    let has_short_key = is_property_like_with_short_key(f, assignment_like_node);
    if should_break_after_operator(f, rhs_expression, has_short_key) {
        return Layout::BreakAfterOperator;
    }

    if !can_break_left_doc
        && (has_short_key
            || matches!(
                rhs_expression,
                Expression::Literal(_)
                    | Expression::CompositeString(_)
                    | Expression::AnonymousClass(_)
                    | Expression::Call(_)
            ))
    {
        return Layout::NeverBreakAfterOperator;
    }

    Layout::Fluid
}

#[inline]
fn is_member_chain_or_single_arg_call<'arena>(
    f: &FormatterState<'_, 'arena>,
    expr: &'arena Expression<'arena>,
) -> bool {
    let is_chain = |arena, e| collect_member_access_chain(arena, e).is_some_and(|c| c.is_eligible_for_chaining(f));

    if is_chain(f.arena, expr) {
        return true;
    }

    if let Expression::Call(call) = expr {
        if let Call::Function(function_call) = call
            && function_call.argument_list.arguments.len() == 1
            && let Some(arg) = function_call.argument_list.arguments.first()
            && is_chain(f.arena, arg.value())
        {
            return true;
        }
        if let Call::Method(method_call) = call
            && method_call.argument_list.arguments.len() == 1
            && let Some(arg) = method_call.argument_list.arguments.first()
            && is_chain(f.arena, arg.value())
        {
            return true;
        }
    }

    false
}

#[inline]
const fn is_assignment(expression: &Expression) -> bool {
    matches!(expression, Expression::Assignment(_))
}

/// Returns whether the given assignment-like node is complex destruction assignment.
///
/// A destruction assignment is considered complex if it has more than two elements
///  and at least one of them is a key-value pair.
#[inline]
fn is_complex_destructuring(assignment_like_node: &AssignmentLikeNode<'_>) -> bool {
    match assignment_like_node {
        AssignmentLikeNode::AssignmentOperation(assignment) => {
            let elements = match assignment.lhs {
                Expression::Array(array) => &array.elements,
                Expression::List(list) => &list.elements,
                Expression::LegacyArray(array) => &array.elements,
                _ => {
                    return false;
                }
            };

            elements.len() > 2 && elements.iter().any(|element| matches!(element, ArrayElement::KeyValue(_)))
        }
        _ => false,
    }
}

#[inline]
fn is_arrow_function_variable_declarator(assignment_like_node: &AssignmentLikeNode<'_>) -> bool {
    match assignment_like_node {
        AssignmentLikeNode::AssignmentOperation(assignment) => {
            matches!((assignment.lhs, assignment.rhs), (Expression::Variable(_), Expression::ArrowFunction(_)))
        }
        _ => false,
    }
}

const MIN_OVERLAP_FOR_BREAK: usize = 3;

#[inline]
fn is_property_like_with_short_key<'arena>(
    f: &FormatterState<'_, 'arena>,
    assignment_like_node: &AssignmentLikeNode<'arena>,
) -> bool {
    let str = match assignment_like_node {
        AssignmentLikeNode::ClassLikeConstantItem(constant_item) => &constant_item.name.value,
        AssignmentLikeNode::ConstantItem(constant_item) => &constant_item.name.value,
        AssignmentLikeNode::EnumCaseBackedItem(enum_case_backed_item) => &enum_case_backed_item.name.value,
        AssignmentLikeNode::PropertyConcreteItem(property_item) => &property_item.variable.name,
        AssignmentLikeNode::YieldPair(yield_pair) => match yield_pair.key {
            Expression::Variable(Variable::Direct(variable)) => &variable.name,
            Expression::Identifier(Identifier::Local(local_identifier)) => &local_identifier.value,
            Expression::Literal(Literal::String(string_literal)) => &string_literal.raw,
            _ => {
                return false;
            }
        },
        AssignmentLikeNode::KeyValueArrayElement(element) => match element.key {
            Expression::Variable(Variable::Direct(variable)) => &variable.name,
            Expression::Identifier(Identifier::Local(local_identifier)) => &local_identifier.value,
            Expression::Literal(Literal::String(string_literal)) => &string_literal.raw,
            _ => {
                return false;
            }
        },
        _ => {
            return false;
        }
    };

    // ↓↓↓ - insufficient overlap for a line break
    // $id = $reallyLongValue;
    // ↓↓↓↓↓↓↓↓↓ - overlap is long enough to break
    // $username =
    //     $reallyLongValue;
    string_width(str) < f.settings.tab_width + MIN_OVERLAP_FOR_BREAK
}

#[inline]
fn should_break_after_operator<'arena>(
    f: &FormatterState<'_, 'arena>,
    rhs_expression: &'arena Expression<'arena>,
    has_short_key: bool,
) -> bool {
    if let Expression::Parenthesized(parenthesized) = rhs_expression {
        return should_break_after_operator(f, parenthesized.expression, has_short_key);
    }

    match rhs_expression {
        Expression::Binary(Binary { lhs, operator: BinaryOperator::NullCoalesce(_), rhs }) => {
            if should_inline_binary_expression(f, rhs_expression) {
                return false;
            }

            if !matches!(unwrap_parenthesized(lhs), Expression::Access(_) | Expression::Call(_)) {
                return true;
            }

            return !collect_member_access_chain(f.arena, rhs).is_some_and(|c| c.is_eligible_for_chaining(f))
                && !matches!(unwrap_parenthesized(rhs), Expression::Instantiation(_));
        }
        Expression::Binary(_) if !should_inline_binary_expression(f, rhs_expression) => {
            return true;
        }
        Expression::Conditional(conditional) => {
            let condition = unwrap_parenthesized(conditional.condition);

            if let binary @ Expression::Binary(Binary { lhs, rhs, .. }) = condition {
                if !lhs.is_binary() || !rhs.is_binary() {
                    return false;
                }

                return !should_inline_binary_expression(f, binary);
            }

            return false;
        }
        Expression::AnonymousClass(anonymous_class) => {
            if !anonymous_class.attribute_lists.is_empty() {
                return true;
            }
        }
        Expression::Closure(closure) => {
            if !closure.attribute_lists.is_empty() {
                return true;
            }
        }
        Expression::ArrowFunction(arrow_func) => {
            if !arrow_func.attribute_lists.is_empty() {
                return true;
            }
        }
        _ => {}
    }

    if has_short_key {
        return false;
    }

    let mut current_expression = rhs_expression;
    loop {
        current_expression = match current_expression {
            Expression::UnaryPrefix(operation) => operation.operand,
            _ => {
                break;
            }
        };
    }

    if is_poorly_breakable_member_or_call_chain(f, rhs_expression) {
        return true;
    };

    false
}

#[inline]
fn is_poorly_breakable_member_or_call_chain<'arena>(
    f: &FormatterState<'_, 'arena>,
    rhs_expression: &'arena Expression<'arena>,
) -> bool {
    if collect_member_access_chain(f.arena, rhs_expression).is_some_and(|c| c.is_eligible_for_chaining(f)) {
        return false;
    }

    let mut is_chain_expression = false;
    let mut is_identifier_or_variable = false;
    let mut call_argument_lists = vec![in f.arena];

    let mut expression = Some(rhs_expression);
    while let Some(node) = expression.take() {
        expression = match node {
            Expression::Call(call) => {
                is_chain_expression = true;

                Some(match call {
                    Call::Function(function_call) => {
                        call_argument_lists.push(&function_call.argument_list);

                        function_call.function
                    }
                    Call::Method(method_call) => {
                        call_argument_lists.push(&method_call.argument_list);

                        method_call.object
                    }
                    Call::NullSafeMethod(null_safe_method_call) => {
                        call_argument_lists.push(&null_safe_method_call.argument_list);

                        null_safe_method_call.object
                    }
                    Call::StaticMethod(static_method_call) => {
                        call_argument_lists.push(&static_method_call.argument_list);

                        static_method_call.class
                    }
                })
            }
            Expression::Access(access) => {
                is_chain_expression = true;

                Some(match access {
                    Access::Property(property_access) => property_access.object,
                    Access::NullSafeProperty(null_safe_property_access) => null_safe_property_access.object,
                    Access::StaticProperty(static_property_access) => static_property_access.class,
                    Access::ClassConstant(class_constant_access) => class_constant_access.class,
                })
            }
            Expression::Identifier(_)
            | Expression::Variable(_)
            | Expression::Static(_)
            | Expression::Self_(_)
            | Expression::Parent(_) => {
                is_identifier_or_variable = true;

                None
            }
            _ => None,
        }
    }

    if !is_chain_expression || !is_identifier_or_variable || call_argument_lists.is_empty() {
        return false;
    }

    for call_argument_list in call_argument_lists {
        let is_poorly_breakable_call = is_lone_short_argument_list(f, call_argument_list);
        if !is_poorly_breakable_call {
            return false;
        }
    }

    true
}

#[inline]
fn is_lone_short_argument_list<'arena>(
    f: &FormatterState<'_, 'arena>,
    argument_list: &'arena ArgumentList<'arena>,
) -> bool {
    if let Some(first_argument) = argument_list.arguments.first() {
        if argument_list.arguments.len() == 1 {
            return is_lone_short_argument(f, first_argument.value());
        }

        false
    } else {
        true
    }
}

const LONE_SHORT_ARGUMENT_THRESHOLD_RATE: f32 = 0.25;

#[inline]
fn is_lone_short_argument<'arena>(f: &FormatterState<'_, 'arena>, argument_value: &'arena Expression<'arena>) -> bool {
    let argument_span = argument_value.span();
    if f.has_comment(argument_span, CommentFlags::all()) {
        return false;
    }

    let print_width = f.settings.print_width;
    let threshold: usize = (print_width as f32 * LONE_SHORT_ARGUMENT_THRESHOLD_RATE).ceil() as usize;

    match argument_value {
        Expression::Literal(
            Literal::False(_) | Literal::True(_) | Literal::Null(_) | Literal::Integer(_) | Literal::Float(_),
        )
        | Expression::Static(_)
        | Expression::Self_(_)
        | Expression::Parent(_)
        | Expression::MagicConstant(_) => true,
        Expression::Variable(Variable::Direct(direct_variable)) => {
            let name = &direct_variable.name;

            string_width(name) <= threshold
        }
        Expression::Identifier(Identifier::Local(local_identifier)) => {
            let name = &local_identifier.value;

            string_width(name) <= threshold
        }
        Expression::UnaryPrefix(unary) if !unary.operator.is_cast() => is_lone_short_argument(f, unary.operand),
        _ => false,
    }
}
