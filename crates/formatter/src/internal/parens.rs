use bumpalo::vec;

use mago_php_version::feature::Feature;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::token::Associativity;
use mago_syntax::token::GetPrecedence;
use mago_syntax::token::Precedence;

use crate::document::Document;
use crate::document::Group;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::utils::unwrap_parenthesized;

impl<'ctx, 'arena> FormatterState<'ctx, 'arena> {
    pub(crate) fn add_parens(
        &mut self,
        document: Document<'arena>,
        node: Node<'arena, 'arena>,
        has_leading_comments: bool,
    ) -> Document<'arena> {
        if has_leading_comments || self.should_indent(node) {
            let group_id = self.next_id();

            Document::Group(
                Group::new(vec![
                    in self.arena;
                    Document::String("("),
                    Document::IndentIfBreak(IndentIfBreak::new(
                        group_id,
                        vec![
                            in self.arena;
                            if self.settings.space_within_grouping_parenthesis {
                                Document::Line(Line::default())
                            } else {
                                Document::Line(Line::soft())
                            },
                            document,
                        ]
                    )),
                    if self.settings.space_within_grouping_parenthesis {
                        Document::Line(Line::default())
                    } else {
                        Document::Line(Line::soft())
                    },
                    Document::String(")"),
                ])
                .with_id(group_id),
            )
        } else {
            Document::Group(Group::new(vec![
                in self.arena;
                Document::String("("),
                if self.settings.space_within_grouping_parenthesis { Document::space() } else { Document::empty() },
                document,
                if self.settings.space_within_grouping_parenthesis { Document::space() } else { Document::empty() },
                Document::String(")"),
            ]))
        }
    }

    pub(crate) fn need_parens(&mut self, node: Node<'arena, 'arena>) -> bool {
        if matches!(node, Node::Program(_)) || node.is_statement() {
            return false;
        }

        self.called_or_accessed_node_needs_parens(node)
            || self.conditional_needs_parens(node)
            || self.binary_node_needs_parens(node)
            || self.unary_node_needs_parens(node)
            || self.assignment_needs_parens(node)
            || self.literal_needs_parens(node)
            || self.pipe_node_needs_parens(node)
    }

    pub(crate) fn should_indent(&self, node: Node<'arena, 'arena>) -> bool {
        if matches!(node, Node::Program(_)) || node.is_statement() {
            return false;
        }

        self.is_unary_or_binary_or_ternary(node)
    }

    fn literal_needs_parens(&self, node: Node<'arena, 'arena>) -> bool {
        let Node::Literal(Literal::Integer(_) | Literal::Float(_)) = node else {
            return false;
        };

        if let Some(Node::Binary(binary)) = self.nth_parent_kind(2)
            && let BinaryOperator::StringConcat(_) = binary.operator
        {
            return true;
        }

        false
    }

    fn conditional_needs_parens(&self, node: Node) -> bool {
        let Node::Conditional(conditional) = node else {
            return false;
        };

        let Some(parent_node) = self.nth_parent_kind(2) else {
            return false;
        };

        // Handle the most specific case first: a conditional nested inside another conditional.
        if let Node::Conditional(parent_conditional) = parent_node {
            let node_is_elvis = conditional.then.is_none();
            let parent_is_elvis = parent_conditional.then.is_none();

            if parent_is_elvis && node_is_elvis {
                // This is a nested elvis operator (`?:`). Since Elvis is left-associative,
                // parentheses are ONLY needed to override the default grouping, which happens
                // when a nested Elvis is on the RIGHT side of its parent.
                //
                // Example:
                //
                // `($a ?: $b) ?: $c` -> Left side, follows associativity, NO parens needed.
                // `$a ?: ($b ?: $c)` -> Right side, overrides associativity, YES parens needed.

                // To determine the position, we check if the current node (`conditional`)
                // is the same as the parent's `condition` node.
                let parent_condition = unwrap_parenthesized(parent_conditional.condition);
                if let Expression::Conditional(parent_cond_as_conditional) = parent_condition
                    && std::ptr::eq(conditional, parent_cond_as_conditional)
                {
                    // It's the left-hand side child, no parens needed.
                    return false;
                }

                // If it's not the left-hand side, it must be the right-hand side,
                // which requires parentheses to preserve the AST structure.
                return true;
            }

            // All other combinations of nested ternaries (e.g., `? :` inside `?:` or vice-versa)
            // require parentheses because PHP considers them ambiguous and throws a fatal error.
            return true;
        }

        if let Node::ArrowFunction(_) = parent_node {
            return matches!(self.nth_parent_kind(3), Some(Node::Pipe(_)));
        }

        self.is_unary_or_binary_or_ternary(parent_node)
            || matches!(
                parent_node,
                Node::Clone(_) | Node::ArrayAppend(_) | Node::VariadicArrayElement(_) | Node::UnaryPostfix(_)
            )
    }

    fn assignment_needs_parens(&self, node: Node<'arena, 'arena>) -> bool {
        if !matches!(node, Node::Assignment(_)) {
            return false;
        }

        let Some(parent_node) = self.nth_parent_kind(2) else {
            return false;
        };

        if let Node::ArrowFunction(_) = parent_node {
            return matches!(self.nth_parent_kind(3), Some(Node::Pipe(_)));
        }

        self.is_unary_or_binary_or_ternary(parent_node) || matches!(parent_node, Node::VariadicArrayElement(_))
    }

    fn pipe_node_needs_parens(&self, node: Node<'arena, 'arena>) -> bool {
        let Node::Pipe(_) = node else {
            return false;
        };

        match self.nth_parent_kind(2) {
            Some(Node::Binary(e)) => {
                let precedence = e.operator.precedence();

                if precedence >= Precedence::Pipe {
                    return true;
                }

                false
            }
            Some(Node::Assignment(_)) => false,
            Some(Node::UnaryPrefix(_) | Node::UnaryPostfix(_)) => true,
            Some(Node::VariadicArrayElement(_)) => true,
            Some(Node::ArrayAppend(_)) => true,
            Some(Node::Conditional(_)) => true,
            _ => false,
        }
    }

    fn binary_node_needs_parens(&self, node: Node<'arena, 'arena>) -> bool {
        let operator = match node {
            Node::Binary(e) => &e.operator,
            _ => return false,
        };

        let precedence = operator.precedence();
        let parent_precedence = match self.nth_parent_kind(2) {
            Some(Node::Clone(_) | Node::ArrayAppend(_) | Node::VariadicArrayElement(_) | Node::UnaryPostfix(_)) => {
                return true;
            }
            Some(Node::UnaryPrefix(u)) => u.operator.precedence(),
            Some(Node::Binary(e)) => {
                let parent_precedence = e.operator.precedence();

                if parent_precedence == precedence {
                    if parent_precedence.is_non_associative() {
                        return true;
                    }

                    if parent_precedence.is_right_associative() && node.end_position() < e.operator.start_position() {
                        return true;
                    }

                    if parent_precedence.is_left_associative() && node.start_position() > e.operator.end_position() {
                        return true;
                    }
                }

                if (operator.is_arithmetic() && !e.operator.is_arithmetic())
                    || (operator.is_multiplicative() || e.operator.is_multiplicative())
                    || (operator.is_bit_shift() && !e.operator.is_bit_shift())
                    || (operator.is_bitwise() && e.operator.is_bitwise() && !e.operator.is_same_as(operator))
                {
                    return true;
                }

                parent_precedence
            }
            Some(Node::Pipe(_)) => Precedence::Pipe,
            Some(Node::ArrowFunction(_)) => {
                let grand_parent_node = self.nth_parent_kind(3);
                if let Some(Node::Pipe(_)) = grand_parent_node {
                    return true;
                }

                return false;
            }
            Some(Node::Conditional(_)) => Precedence::ElvisOrConditional,
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
                return access.left_bracket.start > node.span().start;
            }
            Some(Node::Assignment(_)) => Precedence::Assignment,
            _ => {
                let grand_parent_node = self.nth_parent_kind(3);

                if let Some(Node::Access(_)) = grand_parent_node {
                    return true;
                } else {
                    return false;
                }
            }
        };

        precedence < parent_precedence
    }

    fn unary_node_needs_parens(&self, node: Node<'arena, 'arena>) -> bool {
        if let Node::UnaryPrefix(unary) = node
            && unary.operator.is_error_control()
        {
            let is_include_like = matches!(
                unary.operand,
                Expression::Construct(
                    Construct::Include(_)
                        | Construct::IncludeOnce(_)
                        | Construct::Require(_)
                        | Construct::RequireOnce(_)
                )
            );

            if is_include_like && let Some(Node::Binary(parent_bin)) = self.nth_parent_kind(2) {
                let is_lhs = node.end_position() < parent_bin.operator.start_position();
                if is_lhs {
                    return true;
                }
            }
        }

        let current_precedence = match node {
            Node::UnaryPrefix(u) => u.operator.precedence(),
            Node::UnaryPostfix(u) => u.operator.precedence(),
            _ => return false,
        };

        let Some(parent) = self.nth_parent_kind(2) else {
            return false;
        };

        if matches!(parent, Node::VariadicArrayElement(_)) {
            return true;
        }

        let (parent_precedence, parent_associativity, parent_op_start) = match parent {
            Node::Binary(bin) => {
                let precedence = bin.operator.precedence();
                let associativity = match precedence.associativity() {
                    Some(assoc) => assoc,
                    None => return false,
                };

                (precedence, associativity, bin.operator.start_position())
            }
            Node::Conditional(cond) => {
                (Precedence::ElvisOrConditional, Associativity::Left, cond.question_mark.start_position())
            }
            Node::Pipe(pipe) => (Precedence::Pipe, Associativity::Left, pipe.operator.start_position()),
            _ => return false, // Other parent types don't require parens on their children.
        };

        if current_precedence < parent_precedence {
            return true;
        }

        if current_precedence == parent_precedence {
            let is_lhs = node.end_position() < parent_op_start;

            match parent_associativity {
                Associativity::Left => !is_lhs,
                Associativity::Right => is_lhs,
                Associativity::NonAssociative => true,
            }
        } else {
            false
        }
    }

    fn called_or_accessed_node_needs_parens(&self, node: Node<'arena, 'arena>) -> bool {
        let Node::Expression(expression) = node else {
            return false;
        };

        if let Some(Node::ClosureCreation(closure)) = self.grandparent_node() {
            if let ClosureCreation::Function(_) = closure {
                return self.function_callee_expression_need_parenthesis(expression);
            }

            return self.callee_expression_need_parenthesis(expression, false);
        }

        if let Node::Call(call) = self.parent_node() {
            if let Call::Function(_) = call {
                return self.function_callee_expression_need_parenthesis(expression);
            }

            if let Expression::Instantiation(instantiation) = expression {
                return self.instantiation_needs_parens(instantiation);
            } else {
                return self.callee_expression_need_parenthesis(expression, false);
            }
        }

        if let Node::Instantiation(_) = self.parent_node() {
            return self.callee_expression_need_parenthesis(expression, true);
        }

        if let Node::ArrayAccess(access) = self.parent_node() {
            return if expression.span().end.offset < access.left_bracket.start.offset {
                self.callee_expression_need_parenthesis(expression, false)
            } else {
                false
            };
        }

        if let Some(Node::Access(access)) = self.grandparent_node() {
            let offset = match access {
                Access::Property(property_access) => property_access.arrow.start.offset,
                Access::NullSafeProperty(null_safe_property_access) => {
                    null_safe_property_access.question_mark_arrow.start.offset
                }
                Access::StaticProperty(static_property_access) => static_property_access.double_colon.start.offset,
                Access::ClassConstant(class_constant_access) => class_constant_access.double_colon.start.offset,
            };

            return if expression.span().end.offset < offset {
                self.callee_expression_need_parenthesis(expression, false)
            } else {
                false
            };
        }

        false
    }

    const fn callee_expression_need_parenthesis(
        &self,
        expression: &'arena Expression<'arena>,
        instantiation: bool,
    ) -> bool {
        if instantiation && matches!(expression, Expression::Call(_)) {
            return true;
        }

        if let Expression::Construct(construct) = expression {
            return !construct.has_bounds();
        }

        !matches!(
            expression,
            Expression::Literal(_)
                | Expression::Array(_)
                | Expression::LegacyArray(_)
                | Expression::ArrayAccess(_)
                | Expression::Variable(_)
                | Expression::Identifier(_)
                | Expression::ConstantAccess(_)
                | Expression::Call(_)
                | Expression::Access(_)
                | Expression::ClosureCreation(_)
                | Expression::Static(_)
                | Expression::Self_(_)
                | Expression::Parent(_)
        )
    }

    const fn function_callee_expression_need_parenthesis(&self, expression: &'arena Expression<'arena>) -> bool {
        !matches!(
            expression,
            Expression::Literal(_)
                | Expression::Array(_)
                | Expression::LegacyArray(_)
                | Expression::ArrayAccess(_)
                | Expression::Variable(_)
                | Expression::Identifier(_)
                | Expression::Construct(_)
                | Expression::Call(_)
                | Expression::ClosureCreation(_)
                | Expression::Static(_)
                | Expression::Self_(_)
                | Expression::Parent(_)
        )
    }

    pub(crate) fn instantiation_needs_parens(&self, i: &'arena Instantiation<'arena>) -> bool {
        if self.php_version.is_supported(Feature::NewWithoutParentheses) {
            if i.argument_list.as_ref().is_none_or(|list| list.arguments.is_empty()) {
                if self.settings.parentheses_in_new_expression {
                    self.settings.parentheses_around_new_in_member_access
                } else {
                    true
                }
            } else {
                self.settings.parentheses_around_new_in_member_access
            }
        } else {
            true
        }
    }

    const fn is_unary_or_binary_or_ternary(&self, node: Node<'arena, 'arena>) -> bool {
        self.is_unary(node) || self.is_binaryish(node) || self.is_ternary_conditional(node)
    }

    const fn is_binaryish(&self, node: Node<'arena, 'arena>) -> bool {
        match node {
            Node::Binary(_) => true,
            Node::Conditional(conditional) => conditional.then.is_none(),
            Node::Pipe(_) => true,
            _ => false,
        }
    }

    const fn is_unary(&self, node: Node<'arena, 'arena>) -> bool {
        matches!(node, Node::UnaryPrefix(_) | Node::UnaryPostfix(_))
    }

    const fn is_ternary_conditional(&self, node: Node<'arena, 'arena>) -> bool {
        if let Node::Conditional(op) = node { op.then.is_some() } else { false }
    }
}
