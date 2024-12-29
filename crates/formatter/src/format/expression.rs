use mago_ast::*;
use mago_span::HasSpan;

use crate::document::Document;
use crate::document::Line;
use crate::format::array::print_array_like;
use crate::format::array::ArrayLike;
use crate::format::assignment::print_assignment;
use crate::format::assignment::AssignmentLikeNode;
use crate::format::binaryish;
use crate::format::call::collect_method_call_chain;
use crate::format::call::print_method_call_chain;
use crate::format::call_arguments::print_argument_list;
use crate::format::call_node::print_call_like_node;
use crate::format::call_node::CallLikeNode;
use crate::format::class_like::print_class_like_body;
use crate::format::misc;
use crate::format::misc::print_attribute_list_sequence;
use crate::format::misc::print_condition;
use crate::format::misc::print_modifiers;
use crate::format::string::print_string;
use crate::format::Group;
use crate::format::IfBreak;
use crate::format::IndentIfBreak;
use crate::format::Separator;
use crate::settings::*;
use crate::wrap;
use crate::Formatter;

use crate::format::Format;

impl<'a> Format<'a> for Expression {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        if let Expression::Parenthesized(parenthesized) = self {
            return parenthesized.expression.format(f);
        }

        wrap!(f, self, Expression, {
            match self {
                Expression::Binary(op) => op.format(f),
                Expression::UnaryPrefix(op) => op.format(f),
                Expression::UnaryPostfix(op) => op.format(f),
                Expression::Literal(literal) => literal.format(f),
                Expression::CompositeString(c) => c.format(f),
                Expression::AssignmentOperation(op) => op.format(f),
                Expression::Conditional(op) => op.format(f),
                Expression::Array(array) => array.format(f),
                Expression::LegacyArray(legacy_array) => legacy_array.format(f),
                Expression::List(list) => list.format(f),
                Expression::ArrayAccess(a) => a.format(f),
                Expression::ArrayAppend(a) => a.format(f),
                Expression::AnonymousClass(c) => c.format(f),
                Expression::Closure(c) => c.format(f),
                Expression::ArrowFunction(a) => a.format(f),
                Expression::Variable(v) => v.format(f),
                Expression::Identifier(i) => i.format(f),
                Expression::Match(m) => m.format(f),
                Expression::Yield(y) => y.format(f),
                Expression::Construct(construct) => construct.format(f),
                Expression::Throw(t) => t.format(f),
                Expression::Clone(c) => c.format(f),
                Expression::Call(c) => {
                    if let Some(method_chain) = collect_method_call_chain(self) {
                        let chain_length = method_chain.calls.len();
                        if chain_length >= f.settings.method_chain_break_threshold {
                            // Chain is longer than threshold; format with line breaks
                            print_method_call_chain(&method_chain, f)
                        } else {
                            // Regular formatting
                            c.format(f)
                        }
                    } else {
                        c.format(f)
                    }
                }
                Expression::Access(a) => a.format(f),
                Expression::ConstantAccess(a) => a.format(f),
                Expression::ClosureCreation(c) => c.format(f),
                Expression::Parent(k) => k.format(f),
                Expression::Static(k) => k.format(f),
                Expression::Self_(k) => k.format(f),
                Expression::Instantiation(i) => i.format(f),
                Expression::MagicConstant(c) => c.format(f),
                _ => unreachable!(),
            }
        })
    }
}

impl<'a> Format<'a> for Binary {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Binary, { binaryish::print_binaryish_expression(f, &self.lhs, &self.operator, &self.rhs) })
    }
}

impl<'a> Format<'a> for UnaryPrefix {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UnaryPrefix, {
            if self.operator.is_cast() {
                Document::Group(Group::new(vec![self.operator.format(f), Document::space(), self.operand.format(f)]))
            } else {
                Document::Group(Group::new(vec![self.operator.format(f), self.operand.format(f)]))
            }
        })
    }
}

impl<'a> Format<'a> for UnaryPrefixOperator {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UnaryPrefixOperator, {
            let value = self.as_str(f.interner);

            Document::String(match f.settings.keyword_case {
                CasingStyle::Lowercase => f.as_str(value.to_lowercase()),
                CasingStyle::Uppercase => f.as_str(value.to_uppercase()),
            })
        })
    }
}

impl<'a> Format<'a> for UnaryPostfix {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UnaryPostfix, {
            Document::Group(Group::new(vec![self.operand.format(f), self.operator.format(f)]))
        })
    }
}

impl<'a> Format<'a> for UnaryPostfixOperator {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UnaryPostfixOperator, { Document::String(self.as_str()) })
    }
}

impl<'a> Format<'a> for Literal {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Literal, {
            match self {
                Literal::String(literal) => literal.format(f),
                Literal::Integer(literal) => literal.format(f),
                Literal::Float(literal) => literal.format(f),
                Literal::True(keyword) => keyword.format(f),
                Literal::False(keyword) => keyword.format(f),
                Literal::Null(keyword) => keyword.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for LiteralString {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, LiteralString, { Document::String(print_string(f, &self.kind, &self.value)) })
    }
}

impl<'a> Format<'a> for LiteralInteger {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, LiteralInteger, {
            let value = f.lookup(&self.raw);

            Document::String(value)
        })
    }
}

impl<'a> Format<'a> for LiteralFloat {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, LiteralFloat, {
            let value = f.lookup(&self.raw);

            Document::String(value)
        })
    }
}

impl<'a> Format<'a> for Variable {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Variable, {
            match self {
                Variable::Direct(var) => var.format(f),
                Variable::Indirect(var) => var.format(f),
                Variable::Nested(var) => var.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for IndirectVariable {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IndirectVariable, {
            Document::Group(Group::new(vec![Document::String("${"), self.expression.format(f), Document::String("}")]))
        })
    }
}

impl<'a> Format<'a> for DirectVariable {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, DirectVariable, { Document::String(f.lookup(&self.name)) })
    }
}

impl<'a> Format<'a> for NestedVariable {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, NestedVariable, {
            Document::Group(Group::new(vec![Document::String("$"), self.variable.format(f)]))
        })
    }
}

impl<'a> Format<'a> for Array {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Array, { print_array_like(f, ArrayLike::Array(self)) })
    }
}

impl<'a> Format<'a> for LegacyArray {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, LegacyArray, { print_array_like(f, ArrayLike::LegacyArray(self)) })
    }
}

impl<'a> Format<'a> for List {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, List, { print_array_like(f, ArrayLike::List(self)) })
    }
}

impl<'a> Format<'a> for ArrayElement {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ArrayElement, {
            match self {
                ArrayElement::KeyValue(e) => e.format(f),
                ArrayElement::Value(e) => e.format(f),
                ArrayElement::Variadic(e) => e.format(f),
                ArrayElement::Missing(e) => e.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for KeyValueArrayElement {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, KeyValueArrayElement, {
            let lhs = self.key.format(f);
            let operator = Document::String("=>");

            Document::Group(Group::new(vec![print_assignment(
                f,
                AssignmentLikeNode::KeyValueArrayElement(self),
                lhs,
                operator,
                &self.value,
            )]))
        })
    }
}

impl<'a> Format<'a> for ValueArrayElement {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ValueArrayElement, { self.value.format(f) })
    }
}

impl<'a> Format<'a> for VariadicArrayElement {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, VariadicArrayElement, { Document::Array(vec![Document::String("..."), self.value.format(f)]) })
    }
}

impl<'a> Format<'a> for MissingArrayElement {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MissingArrayElement, { Document::empty() })
    }
}

impl<'a> Format<'a> for Construct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Construct, {
            match self {
                Construct::Isset(c) => c.format(f),
                Construct::Empty(c) => c.format(f),
                Construct::Eval(c) => c.format(f),
                Construct::Include(c) => c.format(f),
                Construct::IncludeOnce(c) => c.format(f),
                Construct::Require(c) => c.format(f),
                Construct::RequireOnce(c) => c.format(f),
                Construct::Print(c) => c.format(f),
                Construct::Exit(c) => c.format(f),
                Construct::Die(c) => c.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for IssetConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IssetConstruct, {
            let mut contents = vec![self.isset.format(f), Document::String("(")];

            if !self.values.is_empty() {
                let mut values =
                    Document::join(self.values.iter().map(|v| v.format(f)).collect(), Separator::CommaLine);

                if f.settings.trailing_comma {
                    values.push(Document::IfBreak(IfBreak::then(Document::String(","))));
                }

                values.insert(0, Document::Line(Line::softline()));

                contents.push(Document::Indent(values));
                contents.push(Document::Line(Line::softline()));
            }

            contents.push(Document::String(")"));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for EmptyConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, EmptyConstruct, {
            Document::Group(Group::new(vec![
                self.empty.format(f),
                Document::String("("),
                self.value.format(f),
                Document::String(")"),
            ]))
        })
    }
}

impl<'a> Format<'a> for EvalConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, EvalConstruct, {
            Document::Group(Group::new(vec![
                self.eval.format(f),
                Document::String("("),
                self.value.format(f),
                Document::String(")"),
            ]))
        })
    }
}

impl<'a> Format<'a> for IncludeConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IncludeConstruct, {
            Document::Group(Group::new(vec![
                self.include.format(f),
                Document::Indent(vec![Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'a> Format<'a> for IncludeOnceConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, IncludeOnceConstruct, {
            Document::Group(Group::new(vec![
                self.include_once.format(f),
                Document::Indent(vec![Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'a> Format<'a> for RequireConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, RequireConstruct, {
            Document::Group(Group::new(vec![
                self.require.format(f),
                Document::Indent(vec![Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'a> Format<'a> for RequireOnceConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, RequireOnceConstruct, {
            Document::Group(Group::new(vec![
                self.require_once.format(f),
                Document::Indent(vec![Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'a> Format<'a> for PrintConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PrintConstruct, {
            Document::Group(Group::new(vec![
                self.print.format(f),
                Document::Indent(vec![Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'a> Format<'a> for ExitConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        // TODO: add support to check what syntax to use `exit` or `die`
        // and whether to use parentheses or not if there are no arguments
        wrap!(f, self, ExitConstruct, { print_call_like_node(f, CallLikeNode::ExitConstruct(self)) })
    }
}

impl<'a> Format<'a> for DieConstruct {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        // TODO: add support to check what syntax to use `exit` or `die`
        // and whether to use parentheses or not if there are no arguments
        wrap!(f, self, DieConstruct, { print_call_like_node(f, CallLikeNode::DieConstruct(self)) })
    }
}

impl<'a> Format<'a> for ArgumentList {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ArgumentList, { print_argument_list(f, self) })
    }
}

impl<'a> Format<'a> for Argument {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Argument, {
            match self {
                Argument::Positional(a) => a.format(f),
                Argument::Named(a) => a.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PositionalArgument {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PositionalArgument, {
            match self.ellipsis {
                Some(_) => Document::Group(Group::new(vec![Document::String("..."), self.value.format(f)])),
                None => Document::Group(Group::new(vec![self.value.format(f)])),
            }
        })
    }
}

impl<'a> Format<'a> for NamedArgument {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, NamedArgument, {
            match self.ellipsis {
                Some(_) => Document::Group(Group::new(vec![
                    self.name.format(f),
                    Document::String(": ..."),
                    self.value.format(f),
                ])),
                None => {
                    Document::Group(Group::new(vec![self.name.format(f), Document::String(": "), self.value.format(f)]))
                }
            }
        })
    }
}

impl<'a> Format<'a> for Assignment {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, AssignmentOperation, {
            let lhs = self.lhs.format(f);

            let operator = match self.operator {
                AssignmentOperator::Assign(_) => Document::String("="),
                AssignmentOperator::Addition(_) => Document::String("+="),
                AssignmentOperator::Subtraction(_) => Document::String("-="),
                AssignmentOperator::Multiplication(_) => Document::String("*="),
                AssignmentOperator::Division(_) => Document::String("/="),
                AssignmentOperator::Modulo(_) => Document::String("%="),
                AssignmentOperator::Exponentiation(_) => Document::String("**="),
                AssignmentOperator::Concat(_) => Document::String(".="),
                AssignmentOperator::BitwiseAnd(_) => Document::String("&="),
                AssignmentOperator::BitwiseOr(_) => Document::String("|="),
                AssignmentOperator::BitwiseXor(_) => Document::String("^="),
                AssignmentOperator::LeftShift(_) => Document::String("<<="),
                AssignmentOperator::RightShift(_) => Document::String(">>="),
                AssignmentOperator::Coalesce(_) => Document::String("??="),
            };

            print_assignment(f, AssignmentLikeNode::AssignmentOperation(self), lhs, operator, &self.rhs)
        })
    }
}

impl<'a> Format<'a> for ClosureUseClauseVariable {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClosureUseClauseVariable, {
            if self.ampersand.is_some() {
                Document::Group(Group::new(vec![Document::String("&"), self.variable.format(f)]))
            } else {
                self.variable.format(f)
            }
        })
    }
}

impl<'a> Format<'a> for ClosureUseClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClosureUseClause, {
            let mut contents = vec![self.r#use.format(f)];
            if f.settings.space_after_closure_use {
                contents.push(Document::space());
            }

            contents.push(Document::String("("));

            let mut variables = vec![];
            for variable in self.variables.iter() {
                variables.push(variable.format(f));
            }

            let mut inner_conent = Document::join(variables, Separator::CommaLine);
            inner_conent.insert(0, Document::Line(Line::softline()));
            if f.settings.trailing_comma {
                inner_conent.push(Document::IfBreak(IfBreak::then(Document::String(","))));
            }

            contents.push(Document::Indent(inner_conent));
            if let Some(comments) = f.print_dangling_comments(self.left_parenthesis.join(self.right_parenthesis), true)
            {
                contents.push(comments);
            } else {
                contents.push(Document::Line(Line::softline()));
            }

            contents.push(Document::String(")"));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for Closure {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Closure, {
            let mut attributes = vec![];
            for attribute_list in self.attributes.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hardline()));
                attributes.push(Document::BreakParent);
            }

            let mut signature = vec![];
            if let Some(s) = &self.r#static {
                signature.push(s.format(f));
                signature.push(Document::space());
            }

            signature.push(self.function.format(f));
            if f.settings.space_before_closure_params {
                signature.push(Document::space());
            }

            if self.ampersand.is_some() {
                signature.push(Document::String("&"));
            }

            signature.push(self.parameters.format(f));
            if let Some(u) = &self.use_clause {
                signature.push(Document::space());
                signature.push(u.format(f));
            }

            if let Some(h) = &self.return_type_hint {
                signature.push(h.format(f));
            }

            let signature_id = f.next_id();
            let signature_document = Document::Group(Group::new(signature).with_id(signature_id));

            Document::Group(Group::new(vec![
                Document::Group(Group::new(attributes)),
                signature_document,
                Document::Group(Group::new(vec![
                    match f.settings.closure_brace_style {
                        BraceStyle::SameLine => Document::space(),
                        BraceStyle::NextLine => Document::IfBreak(
                            IfBreak::new(Document::space(), Document::Line(Line::hardline())).with_id(signature_id),
                        ),
                    },
                    self.body.format(f),
                ])),
            ]))
        })
    }
}

impl<'a> Format<'a> for ArrowFunction {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ArrowFunction, {
            let attributes = print_attribute_list_sequence(f, &self.attributes, true);

            let mut contents = vec![];
            if let Some(s) = &self.r#static {
                contents.push(s.format(f));
                contents.push(Document::space());
            }

            contents.push(self.r#fn.format(f));
            if f.settings.space_before_arrow_function_params {
                contents.push(Document::space());
            }

            if self.ampersand.is_some() {
                contents.push(Document::String("&"));
            }

            contents.push(self.parameters.format(f));
            if let Some(h) = &self.return_type_hint {
                contents.push(h.format(f));
            }

            contents.push(Document::String(" => "));
            contents.push(self.expression.format(f));

            if let Some(attributes) = attributes {
                Document::Group(Group::new(vec![attributes, Document::Group(Group::new(contents))]))
            } else {
                Document::Group(Group::new(contents))
            }
        })
    }
}

impl<'a> Format<'a> for ClassLikeMemberSelector {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeMemberSelector, {
            match self {
                ClassLikeMemberSelector::Identifier(s) => s.format(f),
                ClassLikeMemberSelector::Variable(s) => s.format(f),
                ClassLikeMemberSelector::Expression(s) => s.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ClassLikeMemberExpressionSelector {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeMemberExpressionSelector, {
            Document::Group(Group::new(vec![Document::String("{"), self.expression.format(f), Document::String("}")]))
        })
    }
}

impl<'a> Format<'a> for ClassLikeConstantSelector {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeConstantSelector, {
            match self {
                ClassLikeConstantSelector::Identifier(s) => s.format(f),
                ClassLikeConstantSelector::Expression(s) => s.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ConstantAccess {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ConstantAccess, { self.name.format(f) })
    }
}

impl<'a> Format<'a> for Access {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Access, {
            match self {
                Access::Property(a) => a.format(f),
                Access::NullSafeProperty(a) => a.format(f),
                Access::StaticProperty(a) => a.format(f),
                Access::ClassConstant(a) => a.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PropertyAccess {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyAccess, {
            Document::Group(Group::new(vec![self.object.format(f), Document::String("->"), self.property.format(f)]))
        })
    }
}

impl<'a> Format<'a> for NullSafePropertyAccess {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, NullSafePropertyAccess, {
            Document::Group(Group::new(vec![self.object.format(f), Document::String("?->"), self.property.format(f)]))
        })
    }
}

impl<'a> Format<'a> for StaticPropertyAccess {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, StaticPropertyAccess, {
            Document::Group(Group::new(vec![self.class.format(f), Document::String("::"), self.property.format(f)]))
        })
    }
}

impl<'a> Format<'a> for ClassConstantAccess {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClassConstantAccess, {
            Document::Group(Group::new(vec![self.class.format(f), Document::String("::"), self.constant.format(f)]))
        })
    }
}

impl<'a> Format<'a> for Call {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Call, { print_call_like_node(f, CallLikeNode::Call(self)) })
    }
}

impl<'a> Format<'a> for Throw {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Throw, {
            Document::Group(Group::new(vec![self.throw.format(f), Document::space(), self.exception.format(f)]))
        })
    }
}

impl<'a> Format<'a> for Instantiation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Instantiation, { print_call_like_node(f, CallLikeNode::Instantiation(self)) })
    }
}

impl<'a> Format<'a> for ArrayAccess {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ArrayAccess, {
            Document::Group(Group::new(vec![
                self.array.format(f),
                Document::String("["),
                self.index.format(f),
                Document::String("]"),
            ]))
        })
    }
}

impl<'a> Format<'a> for ArrayAppend {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ArrayAppend, { Document::Group(Group::new(vec![self.array.format(f), Document::String("[]")])) })
    }
}

impl<'a> Format<'a> for MatchArm {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MatchArm, {
            match self {
                MatchArm::Expression(a) => a.format(f),
                MatchArm::Default(a) => a.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for MatchDefaultArm {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MatchDefaultArm, {
            Document::Group(Group::new(vec![
                self.default.format(f),
                Document::IndentIfBreak(IndentIfBreak::new(vec![
                    Document::IfBreak(IfBreak::new(Document::Line(Line::default()), Document::space())),
                    Document::String("=> "),
                ])),
                self.expression.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for MatchExpressionArm {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MatchExpressionArm, {
            let len = self.conditions.len();
            let mut contents = vec![];
            for (i, condition) in self.conditions.iter().enumerate() {
                contents.push(condition.format(f));
                if i != (len - 1) {
                    contents.push(Document::String(","));
                    contents.push(Document::IfBreak(IfBreak::new(Document::Line(Line::default()), Document::space())));
                } else if f.settings.trailing_comma {
                    contents.push(Document::IfBreak(IfBreak::then(Document::String(","))));
                }
            }

            contents.push(Document::IndentIfBreak(IndentIfBreak::new(vec![
                Document::IfBreak(IfBreak::new(Document::Line(Line::default()), Document::space())),
                Document::String("=> "),
            ])));

            Document::Array(vec![
                Document::Group(Group::new(contents)),
                Document::Group(Group::new(vec![self.expression.format(f)])),
            ])
        })
    }
}

impl<'a> Format<'a> for Match {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Match, {
            let mut contents = vec![self.r#match.format(f), Document::space(), print_condition(f, &self.expression)];

            match f.settings.control_brace_style {
                BraceStyle::SameLine => {
                    contents.push(Document::space());
                }
                BraceStyle::NextLine => {
                    contents.push(Document::Line(Line::default()));
                }
            };

            contents.push(Document::String("{"));

            if !self.arms.is_empty() {
                let mut inner_contents =
                    Document::join(self.arms.iter().map(|arm| arm.format(f)).collect::<Vec<_>>(), Separator::CommaLine);

                if f.settings.trailing_comma {
                    inner_contents.push(Document::IfBreak(IfBreak::then(Document::String(","))));
                }

                contents.push(Document::Indent(vec![Document::Line(Line::default()), Document::Array(inner_contents)]));
            }

            if let Some(comments) = f.print_dangling_comments(self.left_brace.join(self.right_brace), true) {
                contents.push(comments);
            } else if !self.arms.is_empty() {
                contents.push(Document::Line(Line::default()));
            }

            contents.push(Document::String("}"));

            Document::Group(Group::new(contents).with_break(true))
        })
    }
}

impl<'a> Format<'a> for Conditional {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Conditional, {
            match &self.then {
                Some(then) => Document::Group(Group::new(vec![
                    self.condition.format(f),
                    Document::Indent(vec![
                        Document::Line(Line::default()),
                        Document::String("? "),
                        then.format(f),
                        Document::Line(Line::default()),
                        Document::String(": "),
                        self.r#else.format(f),
                    ]),
                ])),
                None => Document::Group(Group::new(vec![
                    self.condition.format(f),
                    Document::Indent(vec![
                        Document::space(),
                        Document::Group(Group::new(vec![
                            Document::String("?:"),
                            Document::Line(Line::default()),
                            self.r#else.format(f),
                        ])),
                    ]),
                ])),
            }
        })
    }
}

impl<'a> Format<'a> for CompositeString {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, CompositeString, {
            match self {
                CompositeString::ShellExecute(s) => s.format(f),
                CompositeString::Interpolated(s) => s.format(f),
                CompositeString::Document(s) => s.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for DocumentString {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, DocumentString, {
            let label = f.lookup(&self.label);

            let mut contents = vec![Document::String("<<<")];
            match self.kind {
                DocumentKind::Heredoc => {
                    contents.push(Document::String(label));
                }
                DocumentKind::Nowdoc => {
                    contents.push(Document::String("'"));
                    contents.push(Document::String(label));
                    contents.push(Document::String("'"));
                }
            }

            let indent = match self.indentation {
                DocumentIndentation::None => 0,
                DocumentIndentation::Whitespace(n) => n,
                DocumentIndentation::Tab(n) => n,
                DocumentIndentation::Mixed(t, w) => t + w,
            };

            contents.push(Document::Line(Line::hardline()));
            for part in self.parts.iter() {
                let formatted = match part {
                    StringPart::Literal(l) => {
                        let content = f.lookup(&l.value);
                        let mut part_contents = vec![];
                        for line in Formatter::split_lines(content) {
                            let line = Formatter::skip_leading_whitespace_up_to(line, indent);

                            part_contents.push(Document::String(line));
                        }

                        part_contents = Document::join(part_contents, Separator::Hardline);

                        // if ends with a newline, add a newline
                        if content.ends_with('\n') {
                            part_contents.push(Document::Line(Line::hardline()));
                        }

                        Document::Array(part_contents)
                    }
                    _ => part.format(f),
                };

                contents.push(formatted);
            }

            contents.push(Document::String(label));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for InterpolatedString {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, InterpolatedString, {
            let mut parts = vec![Document::String("\"")];

            for part in self.parts.iter() {
                parts.push(part.format(f));
            }

            parts.push(Document::String("\""));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for ShellExecuteString {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ShellExecuteString, {
            let mut parts = vec![Document::String("`")];

            for part in self.parts.iter() {
                parts.push(part.format(f));
            }

            parts.push(Document::String("`"));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for StringPart {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, StringPart, {
            match self {
                StringPart::Literal(s) => s.format(f),
                StringPart::Expression(s) => s.format(f),
                StringPart::BracedExpression(s) => s.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for LiteralStringPart {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, LiteralStringPart, { Document::String(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for BracedExpressionStringPart {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, BracedExpressionStringPart, {
            Document::Group(Group::new(vec![Document::String("{"), self.expression.format(f), Document::String("}")]))
        })
    }
}

impl<'a> Format<'a> for Yield {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Yield, {
            match self {
                Yield::Value(y) => y.format(f),
                Yield::Pair(y) => y.format(f),
                Yield::From(y) => y.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for YieldValue {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, YieldValue, {
            match &self.value {
                Some(v) => Document::Group(Group::new(vec![self.r#yield.format(f), Document::space(), v.format(f)])),
                None => self.r#yield.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for YieldPair {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, YieldPair, {
            Document::Group(Group::new(vec![
                self.r#yield.format(f),
                Document::space(),
                self.key.format(f),
                Document::space(),
                Document::String("=>"),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for YieldFrom {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, YieldFrom, {
            Document::Group(Group::new(vec![
                self.r#yield.format(f),
                Document::space(),
                self.from.format(f),
                Document::space(),
                self.iterator.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for Clone {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Clone, {
            Document::Group(Group::new(vec![self.clone.format(f), Document::space(), self.object.format(f)]))
        })
    }
}

impl<'a> Format<'a> for MagicConstant {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MagicConstant, {
            match &self {
                MagicConstant::Line(i) => i.format(f),
                MagicConstant::File(i) => i.format(f),
                MagicConstant::Directory(i) => i.format(f),
                MagicConstant::Trait(i) => i.format(f),
                MagicConstant::Method(i) => i.format(f),
                MagicConstant::Function(i) => i.format(f),
                MagicConstant::Property(i) => i.format(f),
                MagicConstant::Namespace(i) => i.format(f),
                MagicConstant::Class(i) => i.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ClosureCreation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClosureCreation, {
            match &self {
                ClosureCreation::Function(c) => c.format(f),
                ClosureCreation::Method(c) => c.format(f),
                ClosureCreation::StaticMethod(c) => c.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for FunctionClosureCreation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, FunctionClosureCreation, {
            Document::Group(Group::new(vec![self.function.format(f), Document::String("(...)")]))
        })
    }
}

impl<'a> Format<'a> for MethodClosureCreation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MethodClosureCreation, {
            Document::Group(Group::new(vec![
                self.object.format(f),
                Document::String("->"),
                self.method.format(f),
                Document::String("(...)"),
            ]))
        })
    }
}

impl<'a> Format<'a> for StaticMethodClosureCreation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, StaticMethodClosureCreation, {
            Document::Group(Group::new(vec![
                self.class.format(f),
                Document::String("::"),
                self.method.format(f),
                Document::String("(...)"),
            ]))
        })
    }
}

impl<'a> Format<'a> for AnonymousClass {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, AnonymousClass, {
            let initialization = {
                let mut contents = vec![Document::BreakParent, self.new.format(f)];
                if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attributes, true) {
                    contents.push(Document::Line(Line::default()));
                    contents.push(attributes);
                } else {
                    contents.push(Document::space());
                }

                Document::Group(Group::new(contents))
            };

            let mut signature = print_modifiers(f, &self.modifiers);
            if !signature.is_empty() {
                signature.push(Document::space());
            }

            signature.push(self.class.format(f));
            if let Some(arguments) = &self.arguments {
                signature.push(arguments.format(f));
            }

            if let Some(extends) = &self.extends {
                signature.push(Document::space());
                signature.push(extends.format(f));
            }

            if let Some(implements) = &self.implements {
                signature.push(Document::space());
                signature.push(implements.format(f));
            }

            let signature_id = f.next_id();
            let signature = Document::Group(Group::new(signature).with_id(signature_id));

            let body = Document::Group(Group::new(vec![
                // we follow the same brace style as closures, not classes
                match f.settings.closure_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => Document::IfBreak(
                        IfBreak::new(
                            Document::space(),
                            Document::Array(vec![Document::Line(Line::hardline()), Document::BreakParent]),
                        )
                        .with_id(signature_id),
                    ),
                },
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace),
            ]));

            Document::Group(Group::new(vec![initialization, signature, body]))
        })
    }
}
