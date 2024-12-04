use fennec_ast::*;
use fennec_span::HasSpan;
use misc::print_colon_delimited_body;
use parameters::print_function_like_parameters;

use crate::array;
use crate::default_line;
use crate::document::*;
use crate::format::assignment::print_assignment;
use crate::format::assignment::AssignmentLikeNode;
use crate::format::block::print_block_of_nodes;
use crate::format::call_node::print_call_like_node;
use crate::format::call_node::CallLikeNode;
use crate::format::class_like::print_class_like_body;
use crate::format::delimited::Delimiter;
use crate::format::misc::print_attribute_list_sequence;
use crate::format::misc::print_modifiers;
use crate::format::sequence::TokenSeparatedSequenceFormatter;
use crate::format::statement::print_statement_sequence;
use crate::group;
use crate::hardline;
use crate::if_break;
use crate::indent;
use crate::indent_if_break;
use crate::settings::*;
use crate::static_str;
use crate::token;
use crate::wrap;
use crate::Formatter;

pub mod array;
pub mod assignment;
pub mod binaryish;
pub mod block;
pub mod call;
pub mod call_arguments;
pub mod call_node;
pub mod class_like;
pub mod control_structure;
pub mod delimited;
pub mod expression;
pub mod misc;
pub mod parameters;
pub mod sequence;
pub mod statement;

pub trait Format<'a> {
    #[must_use]
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a>;
}

impl<'a, T> Format<'a> for Box<T>
where
    T: Format<'a>,
{
    fn format(&'a self, p: &mut Formatter<'a>) -> Document<'a> {
        (**self).format(p)
    }
}

impl<'a, T> Format<'a> for &'a T
where
    T: Format<'a>,
{
    fn format(&'a self, p: &mut Formatter<'a>) -> Document<'a> {
        (**self).format(p)
    }
}

impl<'a> Format<'a> for Program {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        f.enter_node(Node::Program(self));
        let mut parts = vec![];
        if let Some(doc) = block::print_block_body(f, &self.statements) {
            parts.push(doc);
        }

        f.leave_node();

        if f.scripting_mode {
            parts.push(Document::Line(Line::hardline()));
            if f.settings.include_closing_tag {
                parts.push(Document::Line(Line::hardline()));
                parts.push(static_str!("?>"));
                parts.push(Document::Line(Line::hardline()));
            }
        }

        Document::Array(parts)
    }
}

impl<'a> Format<'a> for Statement {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        f.scripting_mode = true;

        wrap!(f, self, FullOpeningTag, {
            Document::String(match f.settings.keyword_case {
                CasingStyle::Lowercase => "<?php",
                CasingStyle::Uppercase => "<?PHP",
            })
        })
    }
}

impl<'a> Format<'a> for ShortOpeningTag {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        f.scripting_mode = true;

        wrap!(f, self, ShortOpeningTag, { Document::String("<?") })
    }
}

impl<'a> Format<'a> for EchoOpeningTag {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        f.scripting_mode = true;

        wrap!(f, self, EchoOpeningTag, { Document::String("<?=") })
    }
}

impl<'a> Format<'a> for ClosingTag {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        f.scripting_mode = false;

        wrap!(f, self, ClosingTag, {
            let last_index = self.span.end.offset;
            if let None = f.skip_spaces_and_new_lines(Some(last_index), false) {
                if !f.settings.include_closing_tag {
                    f.scripting_mode = true;

                    Document::empty()
                } else {
                    Document::String("?>")
                }
            } else {
                Document::String("?>")
            }
        })
    }
}

impl<'a> Format<'a> for Inline {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        f.scripting_mode = false;

        wrap!(f, self, Inline, { Document::String(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for Declare {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, DeclareItem, {
            Document::Array(vec![
                self.name.format(f),
                if f.settings.space_around_declare_equals { Document::space() } else { Document::empty() },
                Document::String("="),
                if f.settings.space_around_declare_equals { Document::space() } else { Document::empty() },
                self.value.format(f),
            ])
        })
    }
}

impl<'a> Format<'a> for DeclareBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, DeclareColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_declare, &self.terminator)
        })
    }
}

impl<'a> Format<'a> for Namespace {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Namespace, {
            let mut parts = vec![self.namespace.format(f)];

            if let Some(name) = &self.name {
                parts.push(Document::space());
                parts.push(name.format(f));
            }

            match &self.body {
                NamespaceBody::Implicit(namespace_implicit_body) => {
                    parts.push(namespace_implicit_body.terminator.format(f));
                    parts.extend(hardline!());
                    parts.extend(hardline!());

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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, LocalIdentifier, { static_str!(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for QualifiedIdentifier {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, QualifiedIdentifier, { static_str!(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for FullyQualifiedIdentifier {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, FullyQualifiedIdentifier, { static_str!(f.lookup(&self.value)) })
    }
}

impl<'a> Format<'a> for Use {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Use, {
            let mut parts = vec![self.r#use.format(f), Document::space()];

            match &self.items {
                UseItems::Sequence(s) => {
                    parts.push(s.format(f));
                }
                UseItems::TypedSequence(s) => {
                    parts.push(s.format(f));
                }
                UseItems::TypedList(t) => {
                    parts.push(t.format(f));
                }
                UseItems::MixedList(m) => {
                    parts.push(m.format(f));
                }
            }

            parts.push(self.terminator.format(f));

            array!(@parts)
        })
    }
}

impl<'a> Format<'a> for UseItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UseItem, {
            let mut parts = vec![self.name.format(f)];

            if let Some(alias) = &self.alias {
                parts.push(Document::space());
                parts.push(alias.format(f));
            }

            group!(@parts, with_break: false)
        })
    }
}

impl<'a> Format<'a> for UseItemSequence {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UseItemSequence, {
            TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.items)
        })
    }
}

impl<'a> Format<'a> for TypedUseItemList {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TypedUseItemList, {
            let mut parts = vec![
                self.r#type.format(f),
                Document::space(),
                self.namespace.format(f),
                token!(f, self.namespace_separator, "\\"),
                token!(f, self.left_brace, "{"),
            ];
            for item in self.items.iter() {
                parts.push(indent!(default_line!(), item.format(f), static_str!(",")));
            }

            parts.push(default_line!());
            parts.push(token!(f, self.right_brace, "}"));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for MixedUseItemList {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MixedUseItemList, {
            let mut parts = vec![
                self.namespace.format(f),
                token!(f, self.namespace_separator, "\\"),
                token!(f, self.left_brace, "{"),
            ];

            for item in self.items.iter() {
                parts.push(indent!(default_line!(), item.format(f), static_str!(",")));
            }

            parts.push(default_line!());
            parts.push(token!(f, self.right_brace, "}"));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for MaybeTypedUseItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MaybeTypedUseItem, {
            match &self.r#type {
                Some(t) => group!(t.format(f), Document::space(), self.item.format(f)),
                None => self.item.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TypedUseItemSequence {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TypedUseItemSequence, {
            array![
                self.r#type.format(f),
                Document::space(),
                TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.items),
            ]
        })
    }
}

impl<'a> Format<'a> for UseItemAlias {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UseItemAlias, {
            let mut parts = vec![];

            parts.push(self.r#as.format(f));
            parts.push(Document::space());
            parts.push(self.identifier.format(f));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for UseType {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, UseType, {
            match self {
                UseType::Function(keyword) => keyword.format(f),
                UseType::Const(keyword) => keyword.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUse {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUse, {
            group!(
                self.r#use.format(f),
                Document::space(),
                TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.trait_names),
                self.specification.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for TraitUseSpecification {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseSpecification, {
            match self {
                TraitUseSpecification::Abstract(s) => s.format(f),
                TraitUseSpecification::Concrete(s) => array!(Document::space(), s.format(f)),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUseAbstractSpecification {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseAbstractSpecification, { self.0.format(f) })
    }
}

impl<'a> Format<'a> for TraitUseConcreteSpecification {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseConcreteSpecification, {
            print_block_of_nodes(f, self.left_brace, &self.adaptations, self.right_brace)
        })
    }
}

impl<'a> Format<'a> for TraitUseAdaptation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseAdaptation, {
            match self {
                TraitUseAdaptation::Precedence(a) => a.format(f),
                TraitUseAdaptation::Alias(a) => a.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUseMethodReference {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseMethodReference, {
            match self {
                TraitUseMethodReference::Identifier(m) => m.format(f),
                TraitUseMethodReference::Absolute(m) => m.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for TraitUseAbsoluteMethodReference {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUseAbsoluteMethodReference, {
            group!(self.trait_name.format(f), token!(f, self.double_colon, "::"), self.method_name.format(f))
        })
    }
}

impl<'a> Format<'a> for TraitUsePrecedenceAdaptation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TraitUsePrecedenceAdaptation, {
            group!(
                self.method_reference.format(f),
                Document::space(),
                self.insteadof.format(f),
                Document::space(),
                TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.trait_names),
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for TraitUseAliasAdaptation {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for ClassLikeConstant {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeConstant, {
            let mut parts = vec![];
            for attribute_list in self.attributes.iter() {
                parts.push(attribute_list.format(f));
                parts.extend(hardline!());
            }

            parts.extend(print_modifiers(f, &self.modifiers));
            parts.push(self.r#const.format(f));
            parts.push(Document::space());
            if let Some(h) = &self.hint {
                parts.push(h.format(f));
                parts.push(Document::space());
            }

            let prefix = array!(@parts);

            if f.settings.split_multi_declare {
                let items = self.items.iter().map(|i| i.format(f)).collect::<Vec<_>>();
                let terminator = self.terminator.format(f);

                let mut constants = vec![];
                let last = items.len() - 1;
                for (i, item) in items.into_iter().enumerate() {
                    constants.push(group!(prefix.clone(), item, terminator.clone()));
                    if i != last {
                        constants.extend(hardline!());
                    }
                }

                array!(@constants)
            } else {
                group!(
                    prefix,
                    TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.items),
                    self.terminator.format(f),
                )
            }
        })
    }
}

impl<'a> Format<'a> for ClassLikeConstantItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ClassLikeConstantItem, {
            let lhs = self.name.format(f);
            let operator = token!(f, self.equals, "=");

            print_assignment(f, AssignmentLikeNode::ClassLikeConstantItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'a> Format<'a> for EnumCase {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, EnumCase, {
            let mut parts = vec![];
            for attribute_list in self.attributes.iter() {
                parts.push(attribute_list.format(f));
                parts.extend(hardline!());
            }

            parts.push(self.case.format(f));
            parts.push(Document::space());
            parts.push(self.item.format(f));
            parts.push(self.terminator.format(f));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for EnumCaseItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, EnumCaseItem, {
            match self {
                EnumCaseItem::Unit(c) => c.format(f),
                EnumCaseItem::Backed(c) => c.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for EnumCaseUnitItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, EnumCaseUnitItem, { self.name.format(f) })
    }
}

impl<'a> Format<'a> for EnumCaseBackedItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, EnumCaseBackedItem, {
            let lhs = self.name.format(f);
            let operator = token!(f, self.equals, "=");

            print_assignment(f, AssignmentLikeNode::EnumCaseBackedItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'a> Format<'a> for Property {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Property, {
            match self {
                Property::Plain(p) => p.format(f),
                Property::Hooked(p) => p.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PlainProperty {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PlainProperty, {
            let mut parts = vec![];
            for attribute_list in self.attributes.iter() {
                parts.push(attribute_list.format(f));
                parts.extend(hardline!());
            }

            if let Some(var) = &self.var {
                parts.push(var.format(f));
                parts.push(Document::space());
            }

            parts.extend(print_modifiers(f, &self.modifiers));

            if let Some(h) = &self.hint {
                parts.push(h.format(f));
                parts.push(Document::space());
            }

            let prefix = array!(@parts);
            if f.settings.split_multi_declare {
                let items = self.items.iter().map(|i| i.format(f)).collect::<Vec<_>>();
                let terminator = self.terminator.format(f);
                let mut properties = vec![];
                let last = items.len() - 1;
                for (i, item) in items.into_iter().enumerate() {
                    properties.push(group!(prefix.clone(), item, terminator.clone()));
                    if i != last {
                        properties.extend(hardline!());
                    }
                }

                array!(@properties)
            } else {
                group!(
                    prefix,
                    TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.items),
                    self.terminator.format(f),
                )
            }
        })
    }
}

impl<'a> Format<'a> for HookedProperty {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, HookedProperty, {
            let mut parts = vec![];
            for attribute_list in self.attributes.iter() {
                parts.push(attribute_list.format(f));
                parts.extend(hardline!());
            }

            if let Some(var) = &self.var {
                parts.push(var.format(f));
                parts.push(Document::space());
            }

            parts.extend(print_modifiers(f, &self.modifiers));

            if let Some(h) = &self.hint {
                parts.push(h.format(f));
                parts.push(Document::space());
            }

            parts.push(self.item.format(f));
            parts.push(Document::space());
            parts.push(self.hooks.format(f));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for PropertyItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyItem, {
            match self {
                PropertyItem::Abstract(p) => p.format(f),
                PropertyItem::Concrete(p) => p.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PropertyAbstractItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyAbstractItem, { self.variable.format(f) })
    }
}

impl<'a> Format<'a> for PropertyConcreteItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyConcreteItem, {
            let lhs = self.variable.format(f);
            let operator = token!(f, self.equals, "=");

            print_assignment(f, AssignmentLikeNode::PropertyConcreteItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'a> Format<'a> for Method {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Method, {
            let mut attributes = vec![];
            for attribute_list in self.attributes.iter() {
                attributes.push(attribute_list.format(f));
                attributes.extend(hardline!());
            }

            let mut signature = vec![];
            signature.extend(print_modifiers(f, &self.modifiers));
            signature.push(self.function.format(f));
            signature.push(Document::space());
            if let Some(ampersand) = self.ampersand {
                signature.push(token!(f, ampersand, "&"));
            }

            signature.push(self.name.format(f));
            signature.push(self.parameters.format(f));
            if let Some(return_type) = &self.return_type_hint {
                signature.push(return_type.format(f));
            }

            let (signature_id, signature_document) = group!(f, @signature);

            let mut body = vec![];
            if let MethodBody::Concrete(_) = self.body {
                body.push(match f.settings.method_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => {
                        if_break!(Document::space(), Document::Line(Line::hardline()), Some(signature_id))
                    }
                });
            }

            body.push(self.body.format(f));

            group!(group!(@attributes), signature_document, group!(@body), Document::BreakParent)
        })
    }
}

impl<'a> Format<'a> for MethodBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MethodBody, {
            match self {
                MethodBody::Abstract(b) => b.format(f),
                MethodBody::Concrete(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for MethodAbstractBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, MethodAbstractBody, { static_str!(";") })
    }
}

impl<'a> Format<'a> for Keyword {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Keyword, {
            let mut value = f.lookup(&self.value);

            value = match f.settings.keyword_case {
                CasingStyle::Lowercase => f.as_str(value.to_ascii_lowercase()),
                CasingStyle::Uppercase => f.as_str(value.to_ascii_uppercase()),
            };

            static_str!(value)
        })
    }
}

impl<'a> Format<'a> for Terminator {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Terminator, {
            match self {
                Terminator::Semicolon(_) | Terminator::TagPair(_, _) => static_str!(";"),
                Terminator::ClosingTag(t) => t.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for ExpressionStatement {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ExpressionStatement, { array![self.expression.format(f), self.terminator.format(f)] })
    }
}

impl<'a> Format<'a> for Extends {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Extends, {
            Document::Group(Group::new(vec![
                self.extends.format(f),
                Document::IndentIfBreak(IndentIfBreak::new(vec![
                    Document::IfBreak(IfBreak::new(Document::Line(Line::hardline()), Document::space())),
                    TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.types),
                ])),
            ]))
        })
    }
}

impl<'a> Format<'a> for Implements {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Implements, {
            Document::Group(Group::new(vec![
                self.implements.format(f),
                Document::IndentIfBreak(IndentIfBreak::new(vec![
                    Document::IfBreak(IfBreak::new(Document::Line(Line::hardline()), Document::space())),
                    TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.types),
                ])),
            ]))
        })
    }
}

impl<'a> Format<'a> for ClassLikeMember {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Interface, {
            let mut attributes = vec![];
            for attribute_list in self.attributes.iter() {
                attributes.push(attribute_list.format(f));
                attributes.extend(hardline!());
            }

            let signature = vec![
                self.interface.format(f),
                Document::space(),
                self.name.format(f),
                if let Some(e) = &self.extends { array!(Document::space(), e.format(f)) } else { Document::empty() },
            ];

            let body = vec![
                match f.settings.classlike_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => {
                        array!(@hardline!())
                    }
                },
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace),
            ];

            group!(group!(@attributes), group!( @signature), group!(@body), Document::BreakParent)
        })
    }
}

impl<'a> Format<'a> for EnumBackingTypeHint {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, EnumBackingTypeHint, {
            group!(token!(f, self.colon, ":"), Document::space(), self.hint.format(f),)
        })
    }
}

impl<'a> Format<'a> for Class {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Class, {
            let mut attributes = vec![];
            for attribute_list in self.attributes.iter() {
                attributes.push(attribute_list.format(f));
                attributes.extend(hardline!());
            }

            let mut signature = print_modifiers(f, &self.modifiers);
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

            let body = vec![
                match f.settings.classlike_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => {
                        array!(@hardline!())
                    }
                },
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace),
            ];

            group!(group!(@attributes), group!(@signature), group!(@body), Document::BreakParent)
        })
    }
}

impl<'a> Format<'a> for Trait {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Trait, {
            let mut attributes = vec![];
            for attribute_list in self.attributes.iter() {
                attributes.push(attribute_list.format(f));
                attributes.extend(hardline!());
            }

            let signature = vec![self.r#trait.format(f), Document::space(), self.name.format(f)];
            let body = vec![
                match f.settings.classlike_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => {
                        array!(@hardline!())
                    }
                },
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace),
            ];

            group!(group!(@attributes), group!(@signature), group!(@body), Document::BreakParent)
        })
    }
}

impl<'a> Format<'a> for Enum {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Enum, {
            let mut attributes = vec![];
            for attribute_list in self.attributes.iter() {
                attributes.push(attribute_list.format(f));
                attributes.extend(hardline!());
            }

            let signature = vec![
                self.r#enum.format(f),
                Document::space(),
                self.name.format(f),
                if let Some(backing_type_hint) = &self.backing_type_hint {
                    // TODO: add an option to add a space before the colon
                    backing_type_hint.format(f)
                } else {
                    Document::empty()
                },
                if let Some(i) = &self.implements { array!(Document::space(), i.format(f)) } else { Document::empty() },
            ];

            let body = vec![
                match f.settings.classlike_brace_style {
                    BraceStyle::SameLine => Document::space(),
                    BraceStyle::NextLine => {
                        array!(@hardline!())
                    }
                },
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace),
            ];

            group!(group!(@attributes), group!(@signature), group!(@body), Document::BreakParent)
        })
    }
}

impl<'a> Format<'a> for Return {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Return, {
            let mut parts = vec![];

            parts.push(self.r#return.format(f));
            if let Some(value) = &self.value {
                parts.push(Document::space());
                parts.push(value.format(f));
            }

            parts.push(self.terminator.format(f));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for Block {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Block, { block::print_block(f, &self.left_brace, &self.statements, &self.right_brace) })
    }
}

impl<'a> Format<'a> for Echo {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Echo, {
            group!(
                self.echo.format(f),
                Document::space(),
                TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.values),
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for ConstantItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, ConstantItem, {
            let lhs = self.name.format(f);
            let operator = token!(f, self.equals, "=");

            print_assignment(f, AssignmentLikeNode::ConstantItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'a> Format<'a> for Constant {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Constant, {
            group!(
                self.r#const.format(f),
                Document::space(),
                TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.items),
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for Attribute {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Attribute, { print_call_like_node(f, CallLikeNode::Attribute(self)) })
    }
}

impl<'a> Format<'a> for Hint {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Hint, {
            let k = |v: &str| match f.settings.keyword_case {
                CasingStyle::Lowercase => static_str!(f.as_str(v.to_ascii_lowercase())),
                CasingStyle::Uppercase => static_str!(f.as_str(v.to_ascii_uppercase())),
            };

            match self {
                Hint::Identifier(identifier) => identifier.format(f),
                Hint::Parenthesized(parenthesized_hint) => {
                    let spacing = if f.settings.type_spacing > 0 {
                        static_str!(f.as_str(" ".repeat(f.settings.type_spacing)))
                    } else {
                        Document::empty()
                    };

                    group!(
                        token!(f, parenthesized_hint.left_parenthesis, "("),
                        spacing.clone(),
                        parenthesized_hint.hint.format(f),
                        spacing,
                        token!(f, parenthesized_hint.right_parenthesis, ")")
                    )
                }
                Hint::Nullable(nullable_hint) => {
                    let spacing = if f.settings.type_spacing > 0 {
                        static_str!(f.as_str(" ".repeat(f.settings.type_spacing)))
                    } else {
                        Document::empty()
                    };

                    // If the nullable type is nested inside another type hint,
                    // we cannot use `?` syntax.
                    let force_long_syntax = matches!(f.parent_node(), Node::Hint(_))
                        || (matches!(
                            nullable_hint.hint.as_ref(),
                            Hint::Nullable(_) | Hint::Union(_) | Hint::Intersection(_) | Hint::Parenthesized(_)
                        ));

                    if force_long_syntax {
                        return group!(
                            k("null"),
                            spacing.clone(),
                            token!(f, nullable_hint.question_mark, "|"),
                            spacing,
                            nullable_hint.hint.format(f)
                        );
                    }

                    match f.settings.null_type_hint {
                        NullTypeHint::NullPipe => {
                            group!(
                                k("null"),
                                spacing.clone(),
                                token!(f, nullable_hint.question_mark, "|"),
                                spacing,
                                nullable_hint.hint.format(f)
                            )
                        }
                        NullTypeHint::Question => {
                            group!(token!(f, nullable_hint.question_mark, "?"), spacing, nullable_hint.hint.format(f))
                        }
                    }
                }
                Hint::Union(union_hint) => {
                    let spacing = if f.settings.type_spacing > 0 {
                        static_str!(f.as_str(" ".repeat(f.settings.type_spacing)))
                    } else {
                        Document::empty()
                    };

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
                                return group!(token!(f, union_hint.pipe, "?"), spacing, union_hint.right.format(f));
                            }
                        }

                        if let Hint::Null(_) = union_hint.right.as_ref() {
                            if f.settings.null_type_hint.is_question() {
                                return group!(token!(f, union_hint.pipe, "?"), spacing, union_hint.left.format(f));
                            }
                        }
                    }

                    group!(
                        union_hint.left.format(f),
                        spacing.clone(),
                        token!(f, union_hint.pipe, "|"),
                        spacing,
                        union_hint.right.format(f),
                    )
                }
                Hint::Intersection(intersection_hint) => {
                    let spacing = if f.settings.type_spacing > 0 {
                        static_str!(f.as_str(" ".repeat(f.settings.type_spacing)))
                    } else {
                        Document::empty()
                    };

                    group!(
                        intersection_hint.left.format(f),
                        spacing.clone(),
                        token!(f, intersection_hint.ampersand, "&"),
                        spacing,
                        intersection_hint.right.format(f),
                    )
                }
                Hint::Null(_) => k("null"),
                Hint::True(_) => k("true"),
                Hint::False(_) => k("false"),
                Hint::Array(_) => k("array"),
                Hint::Callable(_) => k("callable"),
                Hint::Static(_) => k("static"),
                Hint::Self_(_) => k("self"),
                Hint::Parent(_) => k("parent"),
                Hint::Void(_) => k("void"),
                Hint::Never(_) => k("never"),
                Hint::Float(_) => k("float"),
                Hint::Bool(_) => k("bool"),
                Hint::Integer(_) => k("int"),
                Hint::String(_) => k("string"),
                Hint::Object(_) => k("object"),
                Hint::Mixed(_) => k("mixed"),
                Hint::Iterable(_) => k("iterable"),
            }
        })
    }
}

impl<'a> Format<'a> for Modifier {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Modifier, {
            match self {
                Modifier::Static(keyword) => keyword.format(f),
                Modifier::Final(keyword) => keyword.format(f),
                Modifier::Abstract(keyword) => keyword.format(f),
                Modifier::Readonly(keyword) => keyword.format(f),
                Modifier::Public(keyword) => keyword.format(f),
                Modifier::Protected(keyword) => keyword.format(f),
                Modifier::Private(keyword) => keyword.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for AttributeList {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, AttributeList, {
            let delimiter = Delimiter::Attributes(self.hash_left_bracket, self.right_bracket);
            let document = TokenSeparatedSequenceFormatter::new(",")
                .with_trailing_separator(true)
                .format_with_delimiter(f, &self.attributes, delimiter, false);

            Document::Group(Group::new(vec![document]))
        })
    }
}

impl<'a> Format<'a> for PropertyHookAbstractBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookAbstractBody, { token!(f, self.semicolon, ";") })
    }
}

impl<'a> Format<'a> for PropertyHookConcreteBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookConcreteBody, {
            group!(
                Document::space(),
                match self {
                    PropertyHookConcreteBody::Block(b) => b.format(f),
                    PropertyHookConcreteBody::Expression(b) => b.format(f),
                }
            )
        })
    }
}

impl<'a> Format<'a> for PropertyHookConcreteExpressionBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookConcreteExpressionBody, {
            group!(
                token!(f, self.arrow, "=>"),
                Document::space(),
                self.expression.format(f),
                token!(f, self.semicolon, ";")
            )
        })
    }
}

impl<'a> Format<'a> for PropertyHookBody {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookBody, {
            match self {
                PropertyHookBody::Abstract(b) => b.format(f),
                PropertyHookBody::Concrete(b) => b.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for PropertyHook {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHook, {
            let mut parts = vec![];
            for attribute_list in self.attributes.iter() {
                parts.push(attribute_list.format(f));
                parts.extend(hardline!());
            }

            parts.extend(print_modifiers(f, &self.modifiers));
            if let Some(ampersand) = self.ampersand {
                parts.push(token!(f, ampersand, "&"));
            }

            parts.push(self.name.format(f));
            if let Some(parameters) = &self.parameters {
                parts.push(parameters.format(f));
            }

            parts.push(self.body.format(f));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for PropertyHookList {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, PropertyHookList, {
            let mut parts = vec![token!(f, self.left_brace, "{")];
            for hook in self.hooks.iter() {
                parts.push(indent!(default_line!(), hook.format(f)));
            }

            parts.push(default_line!());
            parts.push(token!(f, self.right_brace, "}"));

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for FunctionLikeParameterDefaultValue {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeParameterDefaultValue, {
            group!(
                token!(f, self.equals, "="),
                if_break!(default_line!(), Document::space()),
                indent_if_break!(self.value.format(f)),
            )
        })
    }
}

impl<'a> Format<'a> for FunctionLikeParameter {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeParameter, {
            let mut parts = vec![];
            if let Some(attributes) = print_attribute_list_sequence(f, &self.attributes) {
                parts.push(attributes);
            }

            parts.extend(print_modifiers(f, &self.modifiers));
            if let Some(hint) = &self.hint {
                parts.push(hint.format(f));
                parts.push(Document::space());
            }

            if self.ampersand.is_some() {
                parts.push(Document::String("&"));
            }

            if self.ellipsis.is_some() {
                parts.push(Document::String("..."));
            }

            parts.push(self.variable.format(f));
            if let Some(default_value) = &self.default_value {
                parts.push(Document::space());
                parts.push(default_value.format(f));
            }

            if let Some(hooks) = &self.hooks {
                parts.push(Document::space());
                parts.push(hooks.format(f));
            }

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for FunctionLikeParameterList {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeParameterList, { print_function_like_parameters(f, self) })
    }
}

impl<'a> Format<'a> for FunctionLikeReturnTypeHint {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, FunctionLikeReturnTypeHint, {
            group!(token!(f, self.colon, ":"), Document::space(), self.hint.format(f))
        })
    }
}

impl<'a> Format<'a> for Function {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Function, {
            let mut attributes = vec![];
            for attribute_list in self.attributes.iter() {
                attributes.push(attribute_list.format(f));
                attributes.extend(hardline!());
            }

            let mut signature = vec![];
            signature.push(self.function.format(f));
            signature.push(Document::space());
            if let Some(ampersand) = self.ampersand {
                signature.push(token!(f, ampersand, "&"));
            }

            signature.push(self.name.format(f));
            signature.push(self.parameters.format(f));
            if let Some(return_type) = &self.return_type_hint {
                signature.push(return_type.format(f));
            }

            let (signature_id, signature_document) = group!(f, @signature);

            let mut body = vec![];
            body.push(match f.settings.function_brace_style {
                BraceStyle::SameLine => Document::space(),
                BraceStyle::NextLine => {
                    if_break!(Document::space(), array!(@hardline!()), Some(signature_id))
                }
            });
            body.push(self.body.format(f));

            group!(group!(@attributes), signature_document, group!(@body))
        })
    }
}

impl<'a> Format<'a> for Try {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
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

            group!(@parts)
        })
    }
}

impl<'a> Format<'a> for TryCatchClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TryCatchClause, {
            let mut context = vec![self.hint.format(f)];
            if let Some(variable) = &self.variable {
                context.push(Document::space());
                context.push(variable.format(f));
            }

            group!(
                self.catch.format(f),
                Document::space(),
                token!(f, self.left_parenthesis, "("),
                group!(@context),
                token!(f, self.right_parenthesis, ")"),
                Document::space(),
                self.block.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for TryFinallyClause {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, TryFinallyClause, { group!(self.finally.format(f), Document::space(), self.block.format(f)) })
    }
}

impl<'a> Format<'a> for Global {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Global, {
            group!(
                self.global.format(f),
                Document::space(),
                TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.variables),
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for StaticAbstractItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, StaticAbstractItem, { self.variable.format(f) })
    }
}

impl<'a> Format<'a> for StaticConcreteItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, StaticConcreteItem, {
            group!(
                self.variable.format(f),
                Document::space(),
                token!(f, self.equals, "="),
                Document::space(),
                self.value.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for StaticItem {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, StaticItem, {
            match self {
                StaticItem::Abstract(i) => i.format(f),
                StaticItem::Concrete(i) => i.format(f),
            }
        })
    }
}

impl<'a> Format<'a> for Static {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Static, {
            group!(
                self.r#static.format(f),
                Document::space(),
                TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false).format(f, &self.items),
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for Unset {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Unset, {
            let delimiter = Delimiter::Parentheses(self.left_parenthesis, self.right_parenthesis);
            let formatter = TokenSeparatedSequenceFormatter::new(",").with_trailing_separator(false);

            group!(
                self.unset.format(f),
                formatter.format_with_delimiter(f, &self.values, delimiter, false),
                self.terminator.format(f),
            )
        })
    }
}

impl<'a> Format<'a> for Goto {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Goto, {
            group!(self.goto.format(f), Document::space(), self.label.format(f), self.terminator.format(f))
        })
    }
}

impl<'a> Format<'a> for Label {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        wrap!(f, self, Label, { group!(self.name.format(f), token!(f, self.colon, ":")) })
    }
}

impl<'a> Format<'a> for HaltCompiler {
    fn format(&'a self, f: &mut Formatter<'a>) -> Document<'a> {
        f.scripting_mode = false;

        wrap!(f, self, HaltCompiler, {
            group!(
                self.halt_compiler.format(f),
                token!(f, self.left_parenthesis, "("),
                token!(f, self.right_parenthesis, ")"),
                self.terminator.format(f),
            )
        })
    }
}
