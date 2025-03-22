use block::block_is_empty;
use mago_ast::*;
use mago_span::HasSpan;

use crate::document::*;
use crate::internal::FormatterState;
use crate::internal::format::assignment::AssignmentLikeNode;
use crate::internal::format::assignment::print_assignment;
use crate::internal::format::block::print_block_of_nodes;
use crate::internal::format::call_node::CallLikeNode;
use crate::internal::format::call_node::print_call_like_node;
use crate::internal::format::class_like::print_class_like_body;
use crate::internal::format::misc::print_attribute_list_sequence;
use crate::internal::format::misc::print_colon_delimited_body;
use crate::internal::format::misc::print_modifiers;
use crate::internal::format::parameters::print_function_like_parameters;
use crate::internal::format::return_value::format_return_value;
use crate::internal::format::statement::print_statement_sequence;
use crate::internal::utils;
use crate::settings::*;
use crate::wrap;

pub mod array;
pub mod assignment;
pub mod binaryish;
pub mod block;
pub mod call_arguments;
pub mod call_node;
pub mod class_like;
pub mod control_structure;
pub mod expression;
pub mod member_access;
pub mod misc;
pub mod parameters;
pub mod return_value;
pub mod statement;
pub mod string;

pub trait Format<'a> {
    #[must_use]
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a>;
}

impl<'a, T> Format<'a> for Box<T>
where
    T: Format<'a>,
{
    fn format(&'a self, p: &mut FormatterState<'a>) -> Document<'a> {
        (**self).format(p)
    }
}

impl<'a, T> Format<'a> for &'a T
where
    T: Format<'a>,
{
    fn format(&'a self, p: &mut FormatterState<'a>) -> Document<'a> {
        (**self).format(p)
    }
}

impl<'a> Format<'a> for Program {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        f.comments = self.trivia.comments().copied().collect::<Vec<_>>().into_iter().peekable();
        f.enter_node(Node::Program(self));
        let mut parts = vec![];
        if let Some(doc) = block::print_block_body(f, &self.statements) {
            parts.push(doc);
        }

        f.leave_node();

        parts.push(Document::Trim(Trim::Newlines));
        parts.push(Document::Line(Line::hard()));

        if f.scripting_mode {
            if let Some(last_span) = self.trivia.last_span().or_else(|| self.statements.last_span()) {
                let first_span = self.trivia.first_span().or_else(|| self.statements.first_span()).unwrap_or(last_span);

                if let Some(comments) = f.print_dangling_comments(first_span.join(last_span), false) {
                    parts.push(Document::Line(Line::hard()));
                    parts.push(comments);
                    parts.push(Document::Trim(Trim::Newlines));
                    parts.push(Document::Line(Line::hard()));
                }
            }
        }

        Document::Array(parts)
    }
}

impl<'a> Format<'a> for Statement {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Statement, {
            match self {
                Statement::OpeningTag(t) => t.format(f),
                Statement::ClosingTag(t) => t.format(f),
                Statement::Inline(i) => i.format(f),
                Statement::Namespace(n) => n.format(f),
                Statement::Use(u) => u.format(f),
                Statement::Class(c) => c.format(f),
                Statement::Interface(i) => i.format(f),
                Statement::Trait(t) => t.format(f),
                Statement::Enum(e) => e.format(f),
                Statement::Block(b) => b.format(f),
                Statement::Constant(c) => c.format(f),
                Statement::Function(u) => u.format(f),
                Statement::Declare(d) => d.format(f),
                Statement::Goto(g) => g.format(f),
                Statement::Label(l) => l.format(f),
                Statement::Try(t) => t.format(f),
                Statement::Foreach(o) => o.format(f),
                Statement::For(o) => o.format(f),
                Statement::While(w) => w.format(f),
                Statement::DoWhile(d) => d.format(f),
                Statement::Continue(c) => c.format(f),
                Statement::Break(b) => b.format(f),
                Statement::Switch(s) => s.format(f),
                Statement::If(i) => i.format(f),
                Statement::Return(r) => r.format(f),
                Statement::Expression(e) => e.format(f),
                Statement::Echo(e) => e.format(f),
                Statement::Global(g) => g.format(f),
                Statement::Static(s) => s.format(f),
                Statement::HaltCompiler(h) => h.format(f),
                Statement::Unset(u) => u.format(f),
                Statement::Noop(_) => Document::String(";"),
            }
        })
    }
}

impl<'a> Format<'a> for OpeningTag {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        f.scripting_mode = true;

        wrap!(f, self, OpeningTag, {
            match &self {
                OpeningTag::Full(tag) => tag.format(f),
                OpeningTag::Short(tag) => tag.format(f),
                OpeningTag::Echo(tag) => tag.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for FullOpeningTag {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, FullOpeningTag, { Document::String("<?php") })
    }
}

impl<'a> Format<'a> for ShortOpeningTag {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ShortOpeningTag, { Document::String("<?") })
    }
}

impl<'a> Format<'a> for EchoOpeningTag {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, EchoOpeningTag, { Document::String("<?=") })
    }
}

impl<'a> Format<'a> for ClosingTag {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        f.scripting_mode = false;

        wrap!(f, self, ClosingTag, {
            if f.settings.remove_trailing_close_tag
                && f.skip_spaces_and_new_lines(Some(self.span.end.offset), false).is_none()
            {
                f.scripting_mode = true;

                Document::Trim(Trim::Newlines)
            } else {
                Document::Array(vec![Document::LineSuffixBoundary, Document::String("?>")])
            }
        })
    }
}

impl<'a> Format<'a> for Inline {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        f.scripting_mode = false;

        wrap!(f, self, Inline, {
            utils::replace_end_of_line(Document::String(f.interner.lookup(&self.value)), Separator::LiteralLine)
        })
    }
}

impl<'a> Format<'a> for Declare {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Declare, {
            let mut contents = vec![self.declare.format(f)];

            contents.push(Document::String("("));

            let len = self.items.len();
            for (i, item) in self.items.iter().enumerate() {
                contents.push(item.format(f));
                if i != len - 1 {
                    contents.push(Document::String(", "));
                }
            }

            contents.push(Document::String(")"));
            contents.push(self.body.format(f));

            Document::Group(Group::new(contents).with_break(true))
        })
    }
}

impl<'a> Format<'a> for DeclareItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, DeclareItem, {
            Document::Array(vec![
                self.name.format(f),
                if f.settings.space_around_assignment_in_declare { Document::space() } else { Document::empty() },
                Document::String("="),
                if f.settings.space_around_assignment_in_declare { Document::space() } else { Document::empty() },
                self.value.format(f),
            ])
        })
    }
}

impl<'a> Format<'a> for DeclareBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, DeclareBody, {
            match self {
                DeclareBody::Statement(s) => {
                    let body = s.format(f);

                    misc::adjust_clause(f, s, body, false)
                }
                DeclareBody::ColonDelimited(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for DeclareColonDelimitedBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, DeclareColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_declare, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for Namespace {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Namespace, {
            let mut parts = vec![self.namespace.format(f)];

            if let Some(name) = &self.name {
                parts.push(Document::space());
                parts.push(name.format(f));
            }

            match &self.body {
                NamespaceBody::Implicit(namespace_implicit_body) => {
                    parts.push(namespace_implicit_body.terminator.format(f));
                    parts.push(Document::Line(Line::hard()));
                    parts.push(Document::Line(Line::hard()));

                    parts.extend(print_statement_sequence(f, &namespace_implicit_body.statements));
                }
                NamespaceBody::BraceDelimited(block) => {
                    parts.push(Document::space());
                    parts.push(block.format(f));
                }
            }

            Document::Array(parts)
        })
    }
}

impl<'a> Format<'a> for Identifier {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Identifier, {
            match self {
                Identifier::Local(i) => i.format(f),
                Identifier::Qualified(i) => i.format(f),
                Identifier::FullyQualified(i) => i.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for LocalIdentifier {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, LocalIdentifier, { Document::String(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for QualifiedIdentifier {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, QualifiedIdentifier, { Document::String(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for FullyQualifiedIdentifier {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, FullyQualifiedIdentifier, { Document::String(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for Use {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Use, {
            Document::Group(Group::new(vec![
                self.r#use.format(f),
                Document::space(),
                match &self.items {
                    UseItems::Sequence(s) => s.format(f),
                    UseItems::TypedSequence(s) => s.format(f),
                    UseItems::TypedList(t) => t.format(f),
                    UseItems::MixedList(m) => m.format(f),
                },
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for UseItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, UseItem, {
            let mut parts = vec![self.name.format(f)];

            if let Some(alias) = &self.alias {
                parts.push(Document::space());
                parts.push(alias.format(f));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for UseItemSequence {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, UseItemSequence, {
            let items: Vec<_> = if f.settings.sort_uses {
                statement::sort_use_items(f, self.items.iter()).into_iter().map(|i| i.format(f)).collect()
            } else {
                self.items.iter().map(|i| i.format(f)).collect()
            };

            Document::Group(Group::new(vec![
                Document::Indent(Document::join(items, Separator::CommaLine)),
                Document::Line(Line::soft()),
            ]))
        })
    }
}

impl<'a> Format<'a> for TypedUseItemList {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TypedUseItemList, {
            let mut contents = vec![
                self.r#type.format(f),
                Document::space(),
                self.namespace.format(f),
                Document::String("\\"),
                Document::String("{"),
            ];

            if !self.items.is_empty() {
                let items: Vec<_> = if f.settings.sort_uses {
                    statement::sort_use_items(f, self.items.iter()).into_iter().map(|i| i.format(f)).collect()
                } else {
                    self.items.iter().map(|i| i.format(f)).collect()
                };

                let mut items = Document::join(items, Separator::CommaLine);
                items.insert(0, Document::Line(Line::soft()));

                contents.push(Document::Indent(items));
            }

            if let Some(comments) = f.print_dangling_comments(self.left_brace.join(self.right_brace), true) {
                contents.push(comments);
            } else {
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(Document::String("}"));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for MixedUseItemList {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, MixedUseItemList, {
            let mut contents = vec![self.namespace.format(f), Document::String("\\"), Document::String("{")];

            if !self.items.is_empty() {
                let mut items: Vec<_> = Document::join(
                    if f.settings.sort_uses {
                        statement::sort_maybe_typed_use_items(f, self.items.iter())
                            .into_iter()
                            .map(|i| i.format(f))
                            .collect()
                    } else {
                        self.items.iter().map(|i| i.format(f)).collect()
                    },
                    Separator::CommaLine,
                );

                items.insert(0, Document::Line(Line::soft()));

                contents.push(Document::Indent(items));
            }

            if let Some(comments) = f.print_dangling_comments(self.left_brace.join(self.right_brace), true) {
                contents.push(comments);
            } else {
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(Document::String("}"));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for MaybeTypedUseItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, MaybeTypedUseItem, {
            match &self.r#type {
                Some(t) => Document::Group(Group::new(vec![t.format(f), Document::space(), self.item.format(f)])),
                None => self.item.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TypedUseItemSequence {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TypedUseItemSequence, {
            let mut documents = vec![self.r#type.format(f), Document::space()];

            let items = if f.settings.sort_uses {
                statement::sort_use_items(f, self.items.iter()).into_iter().map(|i| i.format(f)).collect()
            } else {
                self.items.iter().map(|i| i.format(f)).collect()
            };

            documents.push(Document::Indent(Document::join(items, Separator::CommaLine)));
            documents.push(Document::Line(Line::soft()));

            Document::Group(Group::new(documents))
        })
    }
}

impl<'a> Format<'a> for UseItemAlias {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, UseItemAlias, {
            Document::Group(Group::new(vec![self.r#as.format(f), Document::space(), self.identifier.format(f)]))
        })
    }
}

impl<'a> Format<'a> for UseType {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, UseType, {
            match self {
                UseType::Function(keyword) => keyword.format(f),
                UseType::Const(keyword) => keyword.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUse {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUse, {
            let mut contents = vec![self.r#use.format(f), Document::space()];
            for (i, trait_name) in self.trait_names.iter().enumerate() {
                if i != 0 {
                    contents.push(Document::String(", "));
                }

                contents.push(trait_name.format(f));
            }

            contents.push(self.specification.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for TraitUseSpecification {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseSpecification, {
            match self {
                TraitUseSpecification::Abstract(s) => s.format(f),
                TraitUseSpecification::Concrete(s) => Document::Array(vec![Document::space(), s.format(f)]),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUseAbstractSpecification {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseAbstractSpecification, { self.0.format(f) })
    }
}

impl<'a> Format<'a> for TraitUseConcreteSpecification {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseConcreteSpecification, {
            print_block_of_nodes(f, &self.left_brace, &self.adaptations, &self.right_brace, false)
        })
    }
}

impl<'a> Format<'a> for TraitUseAdaptation {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseAdaptation, {
            match self {
                TraitUseAdaptation::Precedence(a) => a.format(f),
                TraitUseAdaptation::Alias(a) => a.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUseMethodReference {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseMethodReference, {
            match self {
                TraitUseMethodReference::Identifier(m) => m.format(f),
                TraitUseMethodReference::Absolute(m) => m.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUseAbsoluteMethodReference {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseAbsoluteMethodReference, {
            Document::Group(Group::new(vec![
                self.trait_name.format(f),
                Document::String("::"),
                self.method_name.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for TraitUsePrecedenceAdaptation {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUsePrecedenceAdaptation, {
            let mut contents =
                vec![self.method_reference.format(f), Document::space(), self.insteadof.format(f), Document::space()];

            for (i, trait_name) in self.trait_names.iter().enumerate() {
                if i != 0 {
                    contents.push(Document::String(", "));
                }

                contents.push(trait_name.format(f));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for TraitUseAliasAdaptation {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseAliasAdaptation, {
            let mut parts = vec![self.method_reference.format(f), Document::space(), self.r#as.format(f)];

            if let Some(v) = &self.visibility {
                parts.push(Document::space());
                parts.push(v.format(f));
            }

            if let Some(a) = &self.alias {
                parts.push(Document::space());
                parts.push(a.format(f));
            }

            parts.push(self.terminator.format(f));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for ClassLikeConstant {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeConstant, {
            let mut contents = vec![];
            if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(Line::hard()));
            };

            if !self.modifiers.is_empty() {
                contents.extend(print_modifiers(f, &self.modifiers));
                contents.push(Document::space());
            }

            contents.push(self.r#const.format(f));
            if let Some(h) = &self.hint {
                contents.push(Document::space());
                contents.push(h.format(f));
            }

            if self.items.len() == 1 {
                contents.push(Document::space());
                contents.push(self.items.as_slice()[0].format(f));
            } else if !self.items.is_empty() {
                contents.push(Document::Indent(vec![Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    self.items.iter().map(|v| v.format(f)).collect(),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for ClassLikeConstantItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeConstantItem, {
            let lhs = self.name.format(f);

            print_assignment(
                f,
                AssignmentLikeNode::ClassLikeConstantItem(self),
                lhs,
                Document::String("="),
                &self.value,
            )
        })
    }
}

impl<'a> Format<'a> for EnumCase {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, EnumCase, {
            let mut parts = vec![];
            for attribute_list in self.attribute_lists.iter() {
                parts.push(attribute_list.format(f));
                parts.push(Document::Line(Line::hard()));
            }

            parts.push(self.case.format(f));
            parts.push(Document::space());
            parts.push(self.item.format(f));
            parts.push(self.terminator.format(f));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for EnumCaseItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, EnumCaseItem, {
            match self {
                EnumCaseItem::Unit(c) => c.format(f),
                EnumCaseItem::Backed(c) => c.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for EnumCaseUnitItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, EnumCaseUnitItem, { self.name.format(f) })
    }
}

impl<'a> Format<'a> for EnumCaseBackedItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, EnumCaseBackedItem, {
            let lhs = self.name.format(f);
            let operator = Document::String("=");

            print_assignment(f, AssignmentLikeNode::EnumCaseBackedItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'a> Format<'a> for Property {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Property, {
            match self {
                Property::Plain(p) => p.format(f),
                Property::Hooked(p) => p.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PlainProperty {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PlainProperty, {
            let mut contents = vec![];
            if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(Line::hard()));
            }

            contents.extend(print_modifiers(f, &self.modifiers));
            let mut should_add_space = !self.modifiers.is_empty();
            if let Some(var) = &self.var {
                if should_add_space {
                    contents.push(Document::space());
                }

                contents.push(var.format(f));
                should_add_space = true;
            }

            if let Some(h) = &self.hint {
                if should_add_space {
                    contents.push(Document::space());
                }

                contents.push(h.format(f));
                should_add_space = true;
            }

            if self.items.len() == 1 {
                if should_add_space {
                    contents.push(Document::space());
                }

                contents.push(self.items.as_slice()[0].format(f));
            } else if !self.items.is_empty() {
                let mut items = Document::join(self.items.iter().map(|v| v.format(f)).collect(), Separator::CommaLine);

                if should_add_space {
                    items.insert(0, Document::Line(Line::default()));
                    contents.push(Document::Indent(items));
                    contents.push(Document::Line(Line::soft()));
                } else {
                    // we don't have any modifiers, so we don't need to indent, or add a line
                    contents.extend(items);
                }
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for HookedProperty {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, HookedProperty, {
            let mut contents = vec![];
            if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(Line::hard()));
            }

            contents.extend(print_modifiers(f, &self.modifiers));
            let mut should_add_space = !self.modifiers.is_empty();
            if let Some(var) = &self.var {
                if should_add_space {
                    contents.push(Document::space());
                }

                contents.push(var.format(f));
                should_add_space = true;
            }

            if let Some(h) = &self.hint {
                if should_add_space {
                    contents.push(Document::space());
                }

                contents.push(h.format(f));
                should_add_space = true;
            }

            if should_add_space {
                contents.push(Document::space());
            }

            contents.push(self.item.format(f));
            contents.push(Document::space());
            contents.push(self.hooks.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for PropertyItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyItem, {
            match self {
                PropertyItem::Abstract(p) => p.format(f),
                PropertyItem::Concrete(p) => p.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PropertyAbstractItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyAbstractItem, { self.variable.format(f) })
    }
}

impl<'a> Format<'a> for PropertyConcreteItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyConcreteItem, {
            let lhs = self.variable.format(f);
            let operator = Document::String("=");

            print_assignment(f, AssignmentLikeNode::PropertyConcreteItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'a> Format<'a> for Method {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Method, {
            let mut attributes = vec![];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let mut signature = print_modifiers(f, &self.modifiers);
            if !signature.is_empty() {
                signature.push(Document::space());
            }

            signature.push(self.function.format(f));
            signature.push(Document::space());
            if self.ampersand.is_some() {
                signature.push(Document::String("&"));
            }

            signature.push(self.name.format(f));
            let has_parameters_or_inner_parameter_comments =
                !self.parameter_list.parameters.is_empty() || f.has_inner_comment(self.parameter_list.span());

            if f.settings.space_before_method_parameter_list_parenthesis {
                signature.push(Document::space());
            }

            signature.push(self.parameter_list.format(f));
            if let Some(return_type) = &self.return_type_hint {
                signature.push(return_type.format(f));
            }

            let signature_id = f.next_id();
            let signature_document = Document::Group(Group::new(signature).with_id(signature_id));

            let mut body = vec![];
            if let MethodBody::Concrete(block) = &self.body {
                let is_constructor = f.interner.lookup(&self.name.value).eq_ignore_ascii_case("__construct");

                let inlined_braces = if is_constructor {
                    f.settings.inline_empty_constructor_braces
                } else {
                    f.settings.inline_empty_method_braces
                } && block_is_empty(f, &block.left_brace, &block.right_brace);

                body.push(if inlined_braces {
                    Document::space()
                } else {
                    match f.settings.method_brace_style {
                        BraceStyle::SameLine => Document::space(),
                        BraceStyle::NextLine => {
                            if !has_parameters_or_inner_parameter_comments {
                                Document::Line(Line::hard())
                            } else {
                                Document::IfBreak(
                                    IfBreak::new(Document::space(), Document::Line(Line::hard())).with_id(signature_id),
                                )
                            }
                        }
                    }
                });
            }

            body.push(self.body.format(f));

            Document::Group(Group::new(vec![
                Document::Group(Group::new(attributes)),
                signature_document,
                Document::Group(Group::new(body)),
            ]))
        })
    }
}

impl<'a> Format<'a> for MethodBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, MethodBody, {
            match self {
                MethodBody::Abstract(b) => b.format(f),
                MethodBody::Concrete(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for MethodAbstractBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, MethodAbstractBody, { Document::String(";") })
    }
}

impl<'a> Format<'a> for Keyword {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Keyword, {
            let value = f.lookup(&self.value);

            Document::String(f.as_str(value.to_ascii_lowercase()))
        })
    }
}

impl<'a> Format<'a> for Terminator {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Terminator, {
            match self {
                Terminator::Semicolon(_) | Terminator::TagPair(_, _) => Document::String(";"),
                Terminator::ClosingTag(t) => Document::Array(vec![Document::space(), t.format(f)]),
            }
        })
    }
}

impl<'a> Format<'a> for ExpressionStatement {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ExpressionStatement, {
            Document::Array(vec![self.expression.format(f), self.terminator.format(f)])
        })
    }
}

impl<'a> Format<'a> for Extends {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Extends, {
            Document::Group(Group::new(vec![
                self.extends.format(f),
                Document::Indent(vec![Document::Line(Line::default())]),
                Document::Indent(Document::join(
                    self.types.iter().map(|v| v.format(f)).collect(),
                    Separator::CommaLine,
                )),
            ]))
        })
    }
}

impl<'a> Format<'a> for Implements {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Implements, {
            Document::Group(Group::new(vec![
                self.implements.format(f),
                Document::Indent(vec![Document::Line(Line::default())]),
                Document::Indent(Document::join(
                    self.types.iter().map(|v| v.format(f)).collect(),
                    Separator::CommaLine,
                )),
            ]))
        })
    }
}

impl<'a> Format<'a> for ClassLikeMember {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeMember, {
            match self {
                ClassLikeMember::TraitUse(m) => m.format(f),
                ClassLikeMember::Constant(m) => m.format(f),
                ClassLikeMember::Property(m) => m.format(f),
                ClassLikeMember::EnumCase(m) => m.format(f),
                ClassLikeMember::Method(m) => m.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for Interface {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Interface, {
            let mut attributes = vec![];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let signature = vec![
                self.interface.format(f),
                Document::space(),
                self.name.format(f),
                if let Some(e) = &self.extends {
                    Document::Array(vec![Document::space(), e.format(f)])
                } else {
                    Document::empty()
                },
            ];

            Document::Group(Group::new(vec![
                Document::Group(Group::new(attributes)),
                Document::Group(Group::new(signature)),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]))
        })
    }
}

impl<'a> Format<'a> for EnumBackingTypeHint {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, EnumBackingTypeHint, {
            Document::Group(Group::new(vec![
                Document::String(":"),
                if f.settings.space_after_colon_in_enum_backing_type { Document::space() } else { Document::empty() },
                self.hint.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for Class {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Class, {
            let attributes = misc::print_attribute_list_sequence(f, &self.attribute_lists);
            let mut signature = print_modifiers(f, &self.modifiers);
            if !signature.is_empty() {
                signature.push(Document::space());
            }

            signature.push(self.class.format(f));
            signature.push(Document::space());
            signature.push(self.name.format(f));

            if let Some(e) = &self.extends {
                signature.push(Document::space());
                signature.push(e.format(f));
            }

            if let Some(i) = &self.implements {
                signature.push(Document::space());
                signature.push(i.format(f));
            }

            let class = Document::Group(Group::new(vec![
                Document::Group(Group::new(signature)),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]));

            if let Some(attributes) = attributes {
                Document::Array(vec![attributes, Document::Line(Line::hard()), class])
            } else {
                class
            }
        })
    }
}

impl<'a> Format<'a> for Trait {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Trait, {
            let mut attributes = vec![];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            Document::Group(Group::new(vec![
                Document::Group(Group::new(attributes)),
                Document::Group(Group::new(vec![self.r#trait.format(f), Document::space(), self.name.format(f)])),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]))
        })
    }
}

impl<'a> Format<'a> for Enum {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Enum, {
            let mut attributes = vec![];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let signature = vec![
                self.r#enum.format(f),
                Document::space(),
                self.name.format(f),
                if let Some(backing_type_hint) = &self.backing_type_hint {
                    if f.settings.space_before_colon_in_enum_backing_type {
                        Document::Array(vec![Document::space(), backing_type_hint.format(f)])
                    } else {
                        backing_type_hint.format(f)
                    }
                } else {
                    Document::empty()
                },
                if let Some(i) = &self.implements {
                    Document::Array(vec![Document::space(), i.format(f)])
                } else {
                    Document::empty()
                },
            ];

            Document::Group(Group::new(vec![
                Document::Group(Group::new(attributes)),
                Document::Group(Group::new(signature)),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]))
        })
    }
}

impl<'a> Format<'a> for Return {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Return, {
            let mut contents = vec![self.r#return.format(f)];

            if let Some(value) = &self.value {
                contents.push(Document::space());
                contents.push(format_return_value(f, value));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for Block {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Block, { block::print_block(f, &self.left_brace, &self.statements, &self.right_brace) })
    }
}

impl<'a> Format<'a> for Echo {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Echo, {
            let mut contents = vec![self.echo.format(f), Document::Indent(vec![Document::Line(Line::default())])];

            if !self.values.is_empty() {
                contents.push(Document::Indent(Document::join(
                    self.values.iter().map(|v| v.format(f)).collect(),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for ConstantItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, ConstantItem, {
            let lhs = self.name.format(f);
            let operator = Document::String("=");

            print_assignment(f, AssignmentLikeNode::ConstantItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'a> Format<'a> for Constant {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Constant, {
            let mut contents = vec![];

            if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(Line::hard()));
            }

            contents.push(self.r#const.format(f));
            if self.items.len() == 1 {
                contents.push(Document::space());
                contents.push(self.items.as_slice()[0].format(f));
            } else if !self.items.is_empty() {
                contents.push(Document::Indent(vec![Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    self.items.iter().map(|v| v.format(f)).collect(),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for Attribute {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Attribute, { print_call_like_node(f, CallLikeNode::Attribute(self)) })
    }
}

impl<'a> Format<'a> for Hint {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Hint, {
            match self {
                Hint::Identifier(identifier) => identifier.format(f),
                Hint::Parenthesized(parenthesized_hint) => Document::Group(Group::new(vec![
                    Document::String("("),
                    if f.settings.space_within_type_parenthesis { Document::space() } else { Document::empty() },
                    parenthesized_hint.hint.format(f),
                    if f.settings.space_within_type_parenthesis { Document::space() } else { Document::empty() },
                    Document::String(")"),
                ])),
                Hint::Nullable(nullable_hint) => {
                    // If the nullable type is nested inside another type hint,
                    // we cannot use `?` syntax.
                    let force_long_syntax = matches!(f.parent_node(), Node::Hint(_))
                        || (matches!(
                            nullable_hint.hint.as_ref(),
                            Hint::Nullable(_) | Hint::Union(_) | Hint::Intersection(_) | Hint::Parenthesized(_)
                        ));

                    if force_long_syntax {
                        return Document::Group(Group::new(vec![
                            Document::String("null"),
                            if f.settings.space_around_pipe_in_union_type {
                                Document::space()
                            } else {
                                Document::empty()
                            },
                            Document::String("|"),
                            if f.settings.space_around_pipe_in_union_type {
                                Document::space()
                            } else {
                                Document::empty()
                            },
                            nullable_hint.hint.format(f),
                        ]));
                    }

                    match f.settings.null_type_hint {
                        NullTypeHint::NullPipe => Document::Group(Group::new(vec![
                            Document::String("null"),
                            if f.settings.space_around_pipe_in_union_type {
                                Document::space()
                            } else {
                                Document::empty()
                            },
                            Document::String("|"),
                            if f.settings.space_around_pipe_in_union_type {
                                Document::space()
                            } else {
                                Document::empty()
                            },
                            nullable_hint.hint.format(f),
                        ])),
                        NullTypeHint::Question => Document::Group(Group::new(vec![
                            Document::String("?"),
                            if f.settings.space_after_nullable_type_question_mark {
                                Document::space()
                            } else {
                                Document::empty()
                            },
                            nullable_hint.hint.format(f),
                        ])),
                    }
                }
                Hint::Union(union_hint) => {
                    let force_long_syntax = matches!(f.parent_node(), Node::Hint(_))
                        || matches!(
                            union_hint.left.as_ref(),
                            Hint::Nullable(_) | Hint::Union(_) | Hint::Intersection(_) | Hint::Parenthesized(_)
                        )
                        || matches!(
                            union_hint.right.as_ref(),
                            Hint::Nullable(_) | Hint::Union(_) | Hint::Intersection(_) | Hint::Parenthesized(_)
                        );

                    if !force_long_syntax {
                        if let Hint::Null(_) = union_hint.left.as_ref() {
                            if f.settings.null_type_hint.is_question() {
                                return Document::Group(Group::new(vec![
                                    Document::String("?"),
                                    if f.settings.space_after_nullable_type_question_mark {
                                        Document::space()
                                    } else {
                                        Document::empty()
                                    },
                                    union_hint.right.format(f),
                                ]));
                            }
                        }

                        if let Hint::Null(_) = union_hint.right.as_ref() {
                            if f.settings.null_type_hint.is_question() {
                                return Document::Group(Group::new(vec![
                                    Document::String("?"),
                                    if f.settings.space_after_nullable_type_question_mark {
                                        Document::space()
                                    } else {
                                        Document::empty()
                                    },
                                    union_hint.left.format(f),
                                ]));
                            }
                        }
                    }

                    Document::Group(Group::new(vec![
                        union_hint.left.format(f),
                        if f.settings.space_around_pipe_in_union_type { Document::space() } else { Document::empty() },
                        Document::String("|"),
                        if f.settings.space_around_pipe_in_union_type { Document::space() } else { Document::empty() },
                        union_hint.right.format(f),
                    ]))
                }
                Hint::Intersection(intersection_hint) => Document::Group(Group::new(vec![
                    intersection_hint.left.format(f),
                    if f.settings.space_around_ampersand_in_intersection_type {
                        Document::space()
                    } else {
                        Document::empty()
                    },
                    Document::String("&"),
                    if f.settings.space_around_ampersand_in_intersection_type {
                        Document::space()
                    } else {
                        Document::empty()
                    },
                    intersection_hint.right.format(f),
                ])),
                Hint::Null(_) => Document::String("null"),
                Hint::True(_) => Document::String("true"),
                Hint::False(_) => Document::String("false"),
                Hint::Array(_) => Document::String("array"),
                Hint::Callable(_) => Document::String("callable"),
                Hint::Static(_) => Document::String("static"),
                Hint::Self_(_) => Document::String("self"),
                Hint::Parent(_) => Document::String("parent"),
                Hint::Void(_) => Document::String("void"),
                Hint::Never(_) => Document::String("never"),
                Hint::Float(_) => Document::String("float"),
                Hint::Bool(_) => Document::String("bool"),
                Hint::Integer(_) => Document::String("int"),
                Hint::String(_) => Document::String("string"),
                Hint::Object(_) => Document::String("object"),
                Hint::Mixed(_) => Document::String("mixed"),
                Hint::Iterable(_) => Document::String("iterable"),
            }
        })
    }
}

impl<'a> Format<'a> for Modifier {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Modifier, {
            match self {
                Modifier::Static(keyword) => keyword.format(f),
                Modifier::Final(keyword) => keyword.format(f),
                Modifier::Abstract(keyword) => keyword.format(f),
                Modifier::Readonly(keyword) => keyword.format(f),
                Modifier::Public(keyword) => keyword.format(f),
                Modifier::Protected(keyword) => keyword.format(f),
                Modifier::Private(keyword) => keyword.format(f),
                Modifier::PrivateSet(keyword) => keyword.format(f),
                Modifier::ProtectedSet(keyword) => keyword.format(f),
                Modifier::PublicSet(keyword) => keyword.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for AttributeList {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, AttributeList, {
            let attributes_count = self.attributes.len();
            let must_break = f.settings.preserve_breaking_attribute_list
                && attributes_count >= 1
                && misc::has_new_line_in_range(
                    f.source_text,
                    self.hash_left_bracket.end.offset,
                    self.attributes.as_slice()[0].span().start.offset,
                );
            let should_inline = !must_break && attributes_count == 1;

            let mut contents = vec![Document::String("#[")];
            if let Some(trailing_comments) = f.print_trailing_comments(self.hash_left_bracket) {
                contents.push(trailing_comments);
            }

            if should_inline {
                contents.push(self.attributes.as_slice()[0].format(f));
            } else {
                contents.push(Document::Indent({
                    let mut attributes = Document::join(
                        self.attributes
                            .iter()
                            .map(|a| Document::Group(Group::new(vec![a.format(f)])))
                            .collect::<Vec<_>>(),
                        Separator::CommaLine,
                    );

                    attributes.insert(0, Document::Line(Line::soft()));

                    attributes
                }));
            }

            if !should_inline {
                if f.settings.trailing_comma {
                    contents.push(Document::IfBreak(IfBreak::then(Document::String(","))));
                }

                contents.push(Document::Line(Line::soft()));
            }

            if let Some(leading_comments) = f.print_leading_comments(self.right_bracket) {
                contents.push(leading_comments);
            }

            contents.push(Document::String("]"));

            Document::Group(Group::new(contents).with_break(must_break))
        })
    }
}

impl<'a> Format<'a> for PropertyHookAbstractBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookAbstractBody, { Document::String(";") })
    }
}

impl<'a> Format<'a> for PropertyHookConcreteBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookConcreteBody, {
            Document::Group(Group::new(vec![
                Document::space(),
                match self {
                    PropertyHookConcreteBody::Block(b) => b.format(f),
                    PropertyHookConcreteBody::Expression(b) => b.format(f),
                },
            ]))
        })
    }
}

impl<'a> Format<'a> for PropertyHookConcreteExpressionBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookConcreteExpressionBody, {
            Document::Group(Group::new(vec![
                Document::String("=>"),
                Document::space(),
                self.expression.format(f),
                Document::String(";"),
            ]))
        })
    }
}

impl<'a> Format<'a> for PropertyHookBody {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookBody, {
            match self {
                PropertyHookBody::Abstract(b) => b.format(f),
                PropertyHookBody::Concrete(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PropertyHook {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHook, {
            let mut contents = vec![];
            if let Some(attributes) = print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(Line::hard()));
            }

            contents.extend(print_modifiers(f, &self.modifiers));
            if !self.modifiers.is_empty() {
                contents.push(Document::space());
            }

            if self.ampersand.is_some() {
                contents.push(Document::String("&"));
            }

            contents.push(self.name.format(f));
            if let Some(parameters) = &self.parameters {
                if f.settings.space_before_hook_parameter_list_parenthesis {
                    contents.push(Document::space());
                }

                contents.push(parameters.format(f));
            }

            contents.push(self.body.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for PropertyHookList {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookList, {
            Document::Group(Group::new(vec![
                Document::String("{"),
                f.print_trailing_comments(self.left_brace).unwrap_or_else(Document::empty),
                if self.hooks.is_empty() {
                    Document::empty()
                } else {
                    Document::Indent(vec![
                        Document::Line(Line::hard()),
                        Document::Array(Document::join(
                            self.hooks.iter().map(|hook| hook.format(f)).collect::<Vec<_>>(),
                            Separator::HardLine,
                        )),
                    ])
                },
                f.print_dangling_comments(self.span(), true).unwrap_or_else(|| {
                    if self.hooks.is_empty() { Document::empty() } else { Document::Line(Line::hard()) }
                }),
                Document::String("}"),
                f.print_trailing_comments(self.right_brace).unwrap_or_else(Document::empty),
            ]))
        })
    }
}

impl<'a> Format<'a> for FunctionLikeParameterDefaultValue {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeParameterDefaultValue, {
            Document::Group(Group::new(vec![Document::String("= "), self.value.format(f)]))
        })
    }
}

impl<'a> Format<'a> for FunctionLikeParameter {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeParameter, {
            let mut contents = vec![];
            if let Some(attributes) = print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(if f.parameter_state.force_break {
                    Line::hard()
                } else {
                    Line::default()
                }));
            }

            contents.extend(print_modifiers(f, &self.modifiers));
            if let Some(hint) = &self.hint {
                if !self.modifiers.is_empty() {
                    contents.push(Document::space());
                }

                contents.push(hint.format(f));
            }

            if !contents.is_empty() {
                contents.push(Document::space());
            }

            if self.ampersand.is_some() {
                contents.push(Document::String("&"));
            }

            if self.ellipsis.is_some() {
                contents.push(Document::String("..."));
            }

            contents.push(self.variable.format(f));
            if let Some(default_value) = &self.default_value {
                contents.push(Document::space());
                contents.push(default_value.format(f));
            }

            if let Some(hooks) = &self.hooks {
                contents.push(Document::space());
                contents.push(hooks.format(f));
            }

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for FunctionLikeParameterList {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeParameterList, { print_function_like_parameters(f, self) })
    }
}

impl<'a> Format<'a> for FunctionLikeReturnTypeHint {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeReturnTypeHint, {
            Document::Group(Group::new(vec![
                if f.settings.space_before_colon_in_return_type { Document::space() } else { Document::empty() },
                Document::String(":"),
                if f.settings.space_after_colon_in_return_type { Document::space() } else { Document::empty() },
                self.hint.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for Function {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Function, {
            let mut attributes = vec![];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let mut signature = vec![];
            signature.push(self.function.format(f));
            signature.push(Document::space());
            if self.ampersand.is_some() {
                signature.push(Document::String("&"));
            }

            signature.push(self.name.format(f));
            let has_parameters_or_inner_parameter_comments =
                !self.parameter_list.parameters.is_empty() || f.has_inner_comment(self.parameter_list.span());

            if f.settings.space_before_function_parameter_list_parenthesis {
                signature.push(Document::space());
            }

            signature.push(self.parameter_list.format(f));
            if let Some(return_type) = &self.return_type_hint {
                signature.push(return_type.format(f));
            }

            let signature_id = f.next_id();
            let signature_document = Document::Group(Group::new(signature).with_id(signature_id));

            let inlined_braces = f.settings.inline_empty_function_braces
                && block_is_empty(f, &self.body.left_brace, &self.body.right_brace);

            Document::Group(Group::new(vec![
                Document::Group(Group::new(attributes)),
                signature_document,
                Document::Group(Group::new(vec![
                    if inlined_braces {
                        Document::space()
                    } else {
                        match f.settings.function_brace_style {
                            BraceStyle::SameLine => Document::space(),
                            BraceStyle::NextLine => {
                                if !has_parameters_or_inner_parameter_comments {
                                    Document::Line(Line::hard())
                                } else {
                                    Document::IfBreak(
                                        IfBreak::new(
                                            Document::space(),
                                            Document::Array(vec![Document::Line(Line::hard()), Document::BreakParent]),
                                        )
                                        .with_id(signature_id),
                                    )
                                }
                            }
                        }
                    },
                    self.body.format(f),
                ])),
            ]))
        })
    }
}

impl<'a> Format<'a> for Try {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Try, {
            let mut parts = vec![self.r#try.format(f), Document::space(), self.block.format(f)];

            for clause in self.catch_clauses.iter() {
                parts.push(Document::space());
                parts.push(clause.format(f));
            }

            if let Some(clause) = &self.finally_clause {
                parts.push(Document::space());
                parts.push(clause.format(f));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for TryCatchClause {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TryCatchClause, {
            Document::Group(Group::new(vec![
                self.catch.format(f),
                if f.settings.space_before_catch_parenthesis { Document::space() } else { Document::empty() },
                f.print_leading_comments(self.left_parenthesis).unwrap_or(Document::empty()),
                Document::String("("),
                if f.settings.space_within_catch_parenthesis { Document::space() } else { Document::empty() },
                Document::Group(Group::new({
                    let mut context = vec![self.hint.format(f)];
                    if let Some(variable) = &self.variable {
                        context.push(Document::space());
                        context.push(variable.format(f));
                    }

                    context
                })),
                if f.settings.space_within_catch_parenthesis { Document::space() } else { Document::empty() },
                Document::String(")"),
                f.print_trailing_comments(self.right_parenthesis).unwrap_or(Document::empty()),
                Document::space(),
                self.block.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for TryFinallyClause {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, TryFinallyClause, {
            Document::Group(Group::new(vec![self.finally.format(f), Document::space(), self.block.format(f)]))
        })
    }
}

impl<'a> Format<'a> for Global {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Global, {
            let mut contents = vec![self.global.format(f)];

            if self.variables.len() == 1 {
                contents.push(Document::space());
                contents.push(self.variables.as_slice()[0].format(f));
            } else if !self.variables.is_empty() {
                contents.push(Document::Indent(vec![Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    self.variables.iter().map(|v| v.format(f)).collect(),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for StaticAbstractItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, StaticAbstractItem, { self.variable.format(f) })
    }
}

impl<'a> Format<'a> for StaticConcreteItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, StaticConcreteItem, {
            Document::Group(Group::new(vec![
                self.variable.format(f),
                Document::space(),
                Document::String("="),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for StaticItem {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, StaticItem, {
            match self {
                StaticItem::Abstract(i) => i.format(f),
                StaticItem::Concrete(i) => i.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for Static {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Static, {
            let mut contents = vec![self.r#static.format(f)];

            if self.items.len() == 1 {
                contents.push(Document::space());
                contents.push(self.items.as_slice()[0].format(f));
            } else if !self.items.is_empty() {
                contents.push(Document::Indent(vec![Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    self.items.iter().map(|v| v.format(f)).collect(),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for Unset {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Unset, {
            let mut contents = vec![self.unset.format(f), Document::String("(")];

            if !self.values.is_empty() {
                let mut values =
                    Document::join(self.values.iter().map(|v| v.format(f)).collect(), Separator::CommaLine);

                if f.settings.trailing_comma {
                    values.push(Document::IfBreak(IfBreak::then(Document::String(","))));
                }

                values.insert(0, Document::Line(Line::soft()));

                contents.push(Document::Indent(values));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(Document::String(")"));
            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'a> Format<'a> for Goto {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Goto, {
            Document::Group(Group::new(vec![
                self.goto.format(f),
                Document::space(),
                self.label.format(f),
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'a> Format<'a> for Label {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        wrap!(f, self, Label, { Document::Group(Group::new(vec![self.name.format(f), Document::String(":")])) })
    }
}

impl<'a> Format<'a> for HaltCompiler {
    fn format(&'a self, f: &mut FormatterState<'a>) -> Document<'a> {
        f.scripting_mode = false;

        wrap!(f, self, HaltCompiler, {
            Document::Group(Group::new(vec![
                self.halt_compiler.format(f),
                Document::String("("),
                Document::String(")"),
                self.terminator.format(f),
            ]))
        })
    }
}
