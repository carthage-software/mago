use fennec_ast::*;
use fennec_span::HasSpan;
use fennec_token::GetPrecedence;

use crate::binaryish::BinaryishOperator;
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
            || self.ternary_or_assignment_needs_parenthesis(node)
            || self.cast_needs_parenthesis(node)
        {
            return true;
        }

        false
    }

    fn ternary_or_assignment_needs_parenthesis(&self, node: Node<'a>) -> bool {
        if !matches!(node, Node::AssignmentOperation(_) | Node::TernaryOperation(_)) {
            return false;
        }

        let Some(parent_node) = self.nth_parent_kind(2) else {
            return false;
        };

        self.is_unary_or_binary_or_ternary(parent_node)
    }

    fn cast_needs_parenthesis(&self, node: Node<'a>) -> bool {
        if !matches!(node, Node::CastOperation(_)) {
            return false;
        }

        let Some(parent_node) = self.nth_parent_kind(2) else {
            return false;
        };

        self.is_unary_or_binary_or_ternary(parent_node)
    }

    fn binarish_node_needs_parenthesis(&self, node: Node<'a>) -> bool {
        let (n, operator) = match node {
            Node::LogicalInfixOperation(e) => (3, BinaryishOperator::from(e.operator)),
            Node::ComparisonOperation(e) => (2, BinaryishOperator::from(e.operator)),
            Node::BitwiseInfixOperation(e) => (3, BinaryishOperator::from(e.operator)),
            Node::ArithmeticInfixOperation(e) => (3, BinaryishOperator::from(e.operator)),
            Node::ConcatOperation(o) => (2, BinaryishOperator::Concat(o.dot)),
            Node::CoalesceOperation(o) => (2, BinaryishOperator::Coalesce(o.double_question_mark)),
            _ => return false,
        };

        let parent_node = self.nth_parent_kind(n);
        let parent_operator = match parent_node {
            Some(Node::LogicalInfixOperation(e)) => BinaryishOperator::from(e.operator),
            Some(Node::ComparisonOperation(e)) => BinaryishOperator::from(e.operator),
            Some(Node::BitwiseInfixOperation(e)) => BinaryishOperator::from(e.operator),
            Some(Node::ArithmeticInfixOperation(e)) => BinaryishOperator::from(e.operator),
            Some(Node::ConcatOperation(o)) => BinaryishOperator::Concat(o.dot),
            Some(Node::CoalesceOperation(_)) => {
                // Add parentheses if parent is a coalesce operator, unless the child is a coalesce operator
                // as well.
                if let BinaryishOperator::Coalesce(_) = operator {
                    return false;
                } else {
                    return true;
                }
            }
            Some(
                Node::CastOperation(_)
                | Node::InstanceofOperation(_)
                | Node::ConditionalTernaryOperation(_)
                | Node::ElvisTernaryOperation(_)
                | Node::ArrayAppend(_),
            ) => {
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
                let grand_parent_node = self.nth_parent_kind(n + 1);

                if let Some(Node::Access(_)) = grand_parent_node {
                    return true;
                } else {
                    return false;
                }
            }
        };

        if operator.is_bitwise_shift() {
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

        if !operator.should_flatten(parent_operator) {
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
        self.is_unary(node) || self.is_binaryish(node) || self.is_ternary(node)
    }

    const fn is_binaryish(&self, node: Node<'a>) -> bool {
        match node {
            Node::ConcatOperation(_)
            | Node::CoalesceOperation(_)
            | Node::LogicalInfixOperation(_)
            | Node::ComparisonOperation(_)
            | Node::BitwiseInfixOperation(_)
            | Node::ArithmeticInfixOperation(_)
            | Node::CastOperation(_)
            | Node::InstanceofOperation(_)
            | Node::ElvisTernaryOperation(_) => true,
            Node::ConditionalTernaryOperation(op) => op.then.is_none(),
            _ => false,
        }
    }

    const fn is_unary(&self, node: Node<'a>) -> bool {
        match node {
            Node::ArithmeticPostfixOperation(_)
            | Node::BitwisePrefixOperation(_)
            | Node::LogicalPrefixOperation(_)
            | Node::ArithmeticPrefixOperation(_) => true,
            Node::ConditionalTernaryOperation(op) => op.then.is_none(),
            _ => false,
        }
    }

    const fn is_ternary(&self, node: Node<'a>) -> bool {
        if let Node::ConditionalTernaryOperation(op) = node {
            op.then.is_some()
        } else {
            false
        }
    }
}
