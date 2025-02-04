use mago_ast::*;
use mago_span::*;

use crate::comment::CommentFlags;
use crate::document::Document;
use crate::document::Group;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::format::binaryish::should_inline_logical_or_coalesce_expression;
use crate::format::Format;
use crate::Formatter;

/// Represents nodes in the Abstract Syntax Tree (AST) that involve assignment-like operations.
#[derive(Debug, Clone, Copy)]
pub(super) enum AssignmentLikeNode<'a> {
    /// Represents a standard assignment operation, such as `$a = $b`.
    AssignmentOperation(&'a Assignment),

    /// Represents a class-like constant item.
    ///
    /// - `A = 1` in `class A { public const A = 1; }`.
    ClassLikeConstantItem(&'a ClassLikeConstantItem),

    /// Represents a global constant item.
    ///
    /// - `A = 1` in `const A = 1;`.
    ConstantItem(&'a ConstantItem),

    /// Represents a backed enum case item.
    ///
    /// - `A = 1` in `enum A: int { case A = 1; }`.
    EnumCaseBackedItem(&'a EnumCaseBackedItem),

    /// Represents a property declaration with an initializer in a class.
    ///
    /// - `$foo = 1` in `class A { public int $foo = 1; }`.
    PropertyConcreteItem(&'a PropertyConcreteItem),

    /// Represents a key-value pair in an array, list, or similar structure.
    ///
    /// - `$a => $b` in `[ $a => $b ]`
    /// - `$a => $b` in `array($a => $b)`
    /// - `$a => $b` in `list($a => $b)`
    KeyValueArrayElement(&'a KeyValueArrayElement),
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

pub(super) fn print_assignment<'a>(
    f: &mut Formatter<'a>,
    assignment_node: AssignmentLikeNode<'a>,
    lhs: Document<'a>,
    operator: Document<'a>,
    rhs_expression: &'a Expression,
) -> Document<'a> {
    let layout = choose_layout(f, &lhs, &assignment_node, rhs_expression);
    let rhs = rhs_expression.format(f);

    match layout {
        Layout::Chain => Document::Array(vec![
            Document::Group(Group::new(vec![lhs])),
            Document::space(),
            operator,
            Document::Line(Line::default()),
            rhs,
        ]),
        Layout::ChainTailArrowChain => {
            Document::Array(vec![Document::Group(Group::new(vec![lhs])), Document::space(), operator, rhs])
        }
        Layout::ChainTail => Document::Group(Group::new(vec![
            lhs,
            Document::space(),
            operator,
            Document::Indent(vec![Document::Line(Line::hardline()), rhs]),
        ])),
        Layout::BreakAfterOperator => Document::Group(Group::new(vec![
            Document::Group(Group::new(vec![lhs])),
            Document::space(),
            operator,
            Document::Group(Group::new(vec![Document::Indent(vec![Document::Line(Line::default()), rhs])])),
        ])),
        Layout::NeverBreakAfterOperator => Document::Group(Group::new(vec![
            Document::Group(Group::new(vec![lhs])),
            Document::space(),
            operator,
            Document::space(),
            Document::Group(Group::new(vec![rhs])),
        ])),
        Layout::BreakLhs => Document::Group(Group::new(vec![
            lhs,
            Document::space(),
            operator,
            Document::space(),
            Document::Group(Group::new(vec![rhs])),
        ])),
        Layout::Fluid => {
            let assignment_id = f.next_id();

            Document::Group(Group::new(vec![
                lhs,
                Document::space(),
                operator,
                Document::Group(
                    Group::new(vec![Document::Indent(vec![Document::Line(Line::default())])]).with_id(assignment_id),
                ),
                Document::IndentIfBreak(IndentIfBreak::new(vec![rhs]).with_id(assignment_id)),
            ]))
        }
    }
}

fn choose_layout<'a, 'b>(
    f: &Formatter<'a>,
    lhs: &'b Document<'a>,
    assignment_like_node: &'b AssignmentLikeNode<'a>,
    rhs_expression: &'a Expression,
) -> Layout {
    if let Expression::Parenthesized(parenthesized) = rhs_expression {
        return choose_layout(f, lhs, assignment_like_node, &parenthesized.expression);
    }

    let is_tail = !is_assignment(rhs_expression);

    let should_use_chain_formatting = matches!(assignment_like_node, AssignmentLikeNode::AssignmentOperation(_))
        && matches!(f.parent_node(), Node::Assignment(_))
        && (!is_tail || !matches!(f.grandparent_node(), Some(Node::ExpressionStatement(_))));

    if should_use_chain_formatting {
        if !is_tail {
            return Layout::Chain;
        } else if let Expression::ArrowFunction(arrow_function) = rhs_expression {
            if let Expression::ArrowFunction(_) = arrow_function.expression.as_ref() {
                return Layout::ChainTailArrowChain;
            }
        }

        return Layout::ChainTail;
    }

    if !is_tail || f.has_leading_own_line_comment(rhs_expression.span()) {
        return Layout::BreakAfterOperator;
    }

    if let Expression::Construct(construct) = rhs_expression {
        if matches!(
            construct,
            Construct::Require(_) | Construct::RequireOnce(_) | Construct::Include(_) | Construct::IncludeOnce(_)
        ) {
            // special case for require/include constructs.
            return Layout::NeverBreakAfterOperator;
        }
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
                Expression::Literal(_) | Expression::CompositeString(_) | Expression::AnonymousClass(_)
            ))
    {
        return Layout::NeverBreakAfterOperator;
    }

    Layout::Fluid
}

fn is_assignment(expression: &Expression) -> bool {
    matches!(expression, Expression::Assignment(_))
}

/// Returns whether the given assignment-like node is complex destruction assignment.
///
/// A destruction assignment is considered complex if it has more than two elements
///  and at least one of them is a key-value pair.
fn is_complex_destructuring(assignment_like_node: &AssignmentLikeNode<'_>) -> bool {
    match assignment_like_node {
        AssignmentLikeNode::AssignmentOperation(assignment) => {
            let elements = match assignment.lhs.as_ref() {
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

fn is_arrow_function_variable_declarator(assignment_like_node: &AssignmentLikeNode<'_>) -> bool {
    match assignment_like_node {
        AssignmentLikeNode::AssignmentOperation(assignment) => {
            matches!(
                (assignment.lhs.as_ref(), assignment.rhs.as_ref()),
                (Expression::Variable(_), Expression::ArrowFunction(_))
            )
        }
        _ => false,
    }
}

const MIN_OVERLAP_FOR_BREAK: usize = 3;

fn is_property_like_with_short_key<'a>(f: &Formatter<'a>, assignment_like_node: &AssignmentLikeNode<'a>) -> bool {
    let width = match assignment_like_node {
        AssignmentLikeNode::ClassLikeConstantItem(constant_item) => f.lookup(&constant_item.name.value).len(),
        AssignmentLikeNode::ConstantItem(constant_item) => f.lookup(&constant_item.name.value).len(),
        AssignmentLikeNode::EnumCaseBackedItem(enum_case_backed_item) => {
            f.lookup(&enum_case_backed_item.name.value).len()
        }
        AssignmentLikeNode::PropertyConcreteItem(property_item) => f.lookup(&property_item.variable.name).len(),
        AssignmentLikeNode::KeyValueArrayElement(element) => match element.key.as_ref() {
            Expression::Variable(Variable::Direct(variable)) => f.lookup(&variable.name).len(),
            Expression::Identifier(Identifier::Local(local_identifier)) => f.lookup(&local_identifier.value).len(),
            Expression::Literal(Literal::String(string_literal)) => f.lookup(&string_literal.value).len(),
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
    width < f.settings.tab_width + MIN_OVERLAP_FOR_BREAK
}

/// <https://github.com/prettier/prettier/blob/eebf0e4b5ec8ac24393c56ced4b4819d4c551f31/src/language-js/print/assignment.js#L182>
fn should_break_after_operator<'a>(f: &Formatter<'a>, rhs_expression: &'a Expression, has_short_key: bool) -> bool {
    if let Expression::Parenthesized(parenthesized) = rhs_expression {
        return should_break_after_operator(f, &parenthesized.expression, has_short_key);
    }

    if rhs_expression.is_binary() && !should_inline_logical_or_coalesce_expression(rhs_expression) {
        return true;
    }

    match rhs_expression {
        Expression::Binary(operation) => {
            if let BinaryOperator::Elvis(_) = operation.operator {
                let mut condition = operation.lhs.as_ref();
                while let Expression::Parenthesized(parenthesized) = condition {
                    condition = &parenthesized.expression;
                }

                return condition.is_binary() && !should_inline_logical_or_coalesce_expression(condition);
            }
        }
        Expression::Conditional(conditional) => {
            let mut condition = conditional.condition.as_ref();
            while let Expression::Parenthesized(parenthesized) = condition {
                condition = &parenthesized.expression;
            }

            return condition.is_binary() && !should_inline_logical_or_coalesce_expression(condition);
        }
        Expression::AnonymousClass(anonymous_class) => {
            if !anonymous_class.attribute_lists.is_empty() {
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
            Expression::UnaryPrefix(operation) => operation.operand.as_ref(),
            _ => {
                break;
            }
        };
    }

    if current_expression.is_string_literal() || is_poorly_breakable_member_or_call_chain(f, rhs_expression) {
        return true;
    };

    false
}

fn is_poorly_breakable_member_or_call_chain<'a>(f: &Formatter<'a>, rhs_expression: &'a Expression) -> bool {
    let mut is_chain_expression = false;
    let mut is_identifier_or_variable = false;
    let mut call_argument_lists = vec![];

    let mut expression = Some(rhs_expression);
    while let Some(node) = expression.take() {
        expression = match node {
            Expression::Call(call) => {
                is_chain_expression = true;

                Some(match call {
                    Call::Function(function_call) => {
                        call_argument_lists.push(&function_call.argument_list);

                        function_call.function.as_ref()
                    }
                    Call::Method(method_call) => {
                        call_argument_lists.push(&method_call.argument_list);

                        method_call.object.as_ref()
                    }
                    Call::NullSafeMethod(null_safe_method_call) => {
                        call_argument_lists.push(&null_safe_method_call.argument_list);

                        null_safe_method_call.object.as_ref()
                    }
                    Call::StaticMethod(static_method_call) => {
                        call_argument_lists.push(&static_method_call.argument_list);

                        static_method_call.class.as_ref()
                    }
                })
            }
            Expression::Access(access) => {
                is_chain_expression = true;

                Some(match access {
                    Access::Property(property_access) => &property_access.object,
                    Access::NullSafeProperty(null_safe_property_access) => &null_safe_property_access.object,
                    Access::StaticProperty(static_property_access) => &static_property_access.class,
                    Access::ClassConstant(class_constant_access) => &class_constant_access.class,
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

fn is_lone_short_argument_list<'a>(f: &Formatter<'a>, argument_list: &'a ArgumentList) -> bool {
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

fn is_lone_short_argument<'a>(f: &Formatter<'a>, argument_value: &'a Expression) -> bool {
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
            let name = f.lookup(&direct_variable.name);

            name.len() <= threshold
        }
        Expression::Identifier(Identifier::Local(local_identifier)) => {
            let name = f.lookup(&local_identifier.value);

            name.len() <= threshold
        }
        Expression::UnaryPrefix(unary) if !unary.operator.is_cast() => is_lone_short_argument(f, &unary.operand),
        _ => false,
    }
}
