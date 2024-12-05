use fennec_ast::*;
use fennec_span::HasSpan;
use fennec_token::GetPrecedence;

use crate::binaryish::should_flatten;
use crate::document::Document;
use crate::document::Group;
use crate::Formatter;

impl<'a> Formatter<'a> {
    pub(crate) fn wrap_parens(&mut self, document: Document<'a>, node: Node<'a>) -> Document<'a> {
        if self.need_parens(node) {
            Document::Group(Group::new(vec![Document::String("("), document, Document::String(")")]))
        } else {
            document
        }
    }

    fn need_parens(&mut self, node: Node<'a>) -> bool {
        if matches!(node, Node::Program(_)) || node.is_statement() {
            return false;
        }

        if self.called_or_accessed_node_needs_parenthesis(node)
            || self.binarish_node_needs_parenthesis(node)
            || self.conditional_or_assignment_needs_parenthesis(node)
            || self.cast_needs_parenthesis(node)
        {
            return true;
        }

        false
    }

    fn conditional_or_assignment_needs_parenthesis(&self, node: Node<'a>) -> bool {
        if !matches!(node, Node::AssignmentOperation(_) | Node::Conditional(_)) {
            return false;
        }

        let Some(parent_node) = self.nth_parent_kind(2) else {
            return false;
        };

        self.is_unary_or_binary_or_ternary(parent_node)
    }

    fn cast_needs_parenthesis(&self, node: Node<'a>) -> bool {
        if !matches!(node, Node::UnaryPrefix(operation) if operation.operator.is_cast()) {
            return false;
        }

        let Some(parent_node) = self.nth_parent_kind(2) else {
            return false;
        };

        self.is_unary_or_binary_or_ternary(parent_node)
    }

    fn binarish_node_needs_parenthesis(&self, node: Node<'a>) -> bool {
        let operator = match node {
            Node::Binary(e) => &e.operator,
            _ => return false,
        };

        let parent_operator = match self.nth_parent_kind(2) {
            Some(Node::Binary(e)) => {
                if let BinaryOperator::NullCoalesce(_) = e.operator {
                    // Add parentheses if parent is a coalesce operator,
                    //  unless the child is a coalesce operator as well.
                    return !matches!(operator, BinaryOperator::NullCoalesce(_));
                }

                if let BinaryOperator::Instanceof(_) = e.operator {
                    // Add parentheses if parent is an instanceof operator.
                    return true;
                }

                if let BinaryOperator::Elvis(_) = e.operator {
                    // Add parentheses if parent is an elvis operator.
                    return true;
                }

                &e.operator
            }
            Some(Node::UnaryPrefix(operation)) if operation.operator.is_cast() => {
                return true;
            }
            Some(Node::Conditional(_) | Node::ArrayAppend(_)) => {
                return true;
            }
            Some(Node::ArrayAccess(access)) => {
                // we add parentheses if the parent is an array access and the child is a binaryish node
                //
                // Example:
                //
                // ```php
                // ($foo ?? $bar)[$baz];
                // ```
                //
                // requires parentheses, if we remove them, the code will be interpreted as:
                //
                // ```php
                // $foo ?? ($bar[$baz]);
                // ```
                return access.left_bracket.start.offset > node.span().start.offset;
            }
            Some(Node::Access(access)) => {
                // we add parentheses if the parent is an access and the child is a binaryish node
                //
                // Example:
                //
                // ```php
                // ($foo ?? $bar)->baz;
                // ($foo ?? $bar)?->baz;
                // ($foo ?? $bar)::$baz;
                // ($foo ?? $bar)::baz;
                // ```
                //
                // requires parentheses, if we remove them, the code will be interpreted as:
                //
                // ```php
                // $foo ?? $bar->baz;
                // $foo ?? $bar->baz;
                // $foo ?? $bar::$baz;
                // $foo ?? $bar)::baz;
                // ```
                return node.span().start.offset == access.span().start.offset;
            }
            Some(Node::Call(call)) => {
                // we add parentheses if the parent is a call and the child is a binaryish node
                //
                // Example:
                //
                // ```php
                // ($foo ?? $bar)();
                // ```
                //
                // requires parentheses, if we remove them, the code will be interpreted as:
                //
                // ```php
                // $foo ?? $bar();
                // ```
                return node.span().start.offset == call.span().start.offset;
            }
            _ => {
                let grand_parent_node = self.nth_parent_kind(3);

                if let Some(Node::Access(_)) = grand_parent_node {
                    return true;
                } else {
                    return false;
                }
            }
        };

        if operator.is_bit_shift() {
            return true;
        }

        if parent_operator.is_comparison() {
            return true;
        }

        if parent_operator.is_bitwise() {
            return !operator.is_same_as(&parent_operator);
        }

        if operator.is_comparison() {
            if parent_operator.is_logical() {
                return false;
            } else {
                return true;
            }
        }

        // Add parentheses if operators have different precedence
        let precedence = operator.precedence();
        let parent_precedence = parent_operator.precedence();
        if parent_precedence > precedence {
            return true;
        }

        if operator.is_arithmetic() && parent_operator.is_arithmetic() && !operator.is_same_as(&parent_operator) {
            return true;
        }

        if parent_precedence < precedence {
            return false;
        }

        if !should_flatten(operator, parent_operator) {
            return true;
        }

        false
    }

    fn called_or_accessed_node_needs_parenthesis(&self, node: Node<'a>) -> bool {
        let Node::Expression(expression) = node else {
            return false;
        };

        if let Node::Call(call) = self.parent_node() {
            if let Call::Function(_) = call {
                return matches!(expression, Expression::Access(_) | Expression::Instantiation(_));
            }

            if let Expression::Instantiation(new) = expression {
                if new.arguments.is_none() {
                    // parentheses are required if the instantiation has no arguments
                    // e.g. `new Foo->baz()` should be `(new Foo)->baz()`
                    return true;
                }

                // parentheses are not required if the instantiation has arguments
                // e.g. `new Foo()->baz()`.
                //
                // but this is only allowed in PHP 8.4, so for now, we add
                // parentheses to be safe, in the future, we can add an option
                // to remove them.
                //
                // TODO(azjezz): we should add an option to remove parentheses.
                return true;
            } else {
                return self.callee_expression_need_parenthesis(expression);
            }
        }

        if let Node::Instantiation(_) = self.parent_node() {
            return self.callee_expression_need_parenthesis(expression);
        }

        if let Some(Node::Access(_)) = self.grandparent_node() {
            return self.callee_expression_need_parenthesis(expression);
        }

        false
    }

    const fn callee_expression_need_parenthesis(&self, expression: &'a Expression) -> bool {
        match expression {
            Expression::Literal(_)
            | Expression::Array(_)
            | Expression::LegacyArray(_)
            | Expression::ArrayAccess(_)
            | Expression::Variable(_)
            | Expression::Identifier(_)
            | Expression::Construct(_)
            | Expression::Call(_)
            | Expression::Access(_)
            | Expression::ClosureCreation(_)
            | Expression::Static(_)
            | Expression::Self_(_)
            | Expression::Parent(_) => false,
            _ => true,
        }
    }

    const fn is_unary_or_binary_or_ternary(&self, node: Node<'a>) -> bool {
        self.is_unary(node) || self.is_binaryish(node) || self.is_conditional(node)
    }

    const fn is_binaryish(&self, node: Node<'a>) -> bool {
        match node {
            Node::Binary(_) => true,
            Node::Conditional(conditional) => conditional.then.is_none(),
            _ => false,
        }
    }

    const fn is_unary(&self, node: Node<'a>) -> bool {
        match node {
            Node::UnaryPrefix(_) | Node::UnaryPostfix(_) => true,
            _ => false,
        }
    }

    const fn is_conditional(&self, node: Node<'a>) -> bool {
        if let Node::Conditional(op) = node {
            op.then.is_some()
        } else {
            false
        }
    }
}
