use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::document::*;
use crate::internal::FormatterState;
use crate::internal::format::assignment::AssignmentLikeNode;
use crate::internal::format::assignment::print_assignment;
use crate::internal::format::block::block_is_empty;
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
use crate::internal::format::string::print_lowercase_keyword;
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

pub trait Format<'arena> {
    #[must_use]
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena>;
}

impl<'arena, T> Format<'arena> for Box<T>
where
    T: Format<'arena>,
{
    fn format(&'arena self, p: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        (**self).format(p)
    }
}

impl<'arena, T> Format<'arena> for &'arena T
where
    T: Format<'arena>,
{
    fn format(&self, p: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        (**self).format(p)
    }
}

impl<'arena> Format<'arena> for Program<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        f.enter_node(Node::Program(self));
        let mut parts = vec![in f.arena];
        if let Some(doc) = block::print_block_body(f, &self.statements) {
            parts.push(doc);
        }

        f.leave_node();

        if !f.halted_compilation {
            parts.push(Document::Trim(Trim::Newlines));
            parts.push(Document::Line(Line::hard()));

            if f.scripting_mode
                && let Some(last_span) = self.trivia.last_span().or_else(|| self.statements.last_span())
            {
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

impl<'arena> Format<'arena> for Statement<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        let was_in_script_terminating_statement = f.in_script_terminating_statement;

        f.in_script_terminating_statement = !self.is_closing_tag() && self.terminates_scripting();

        let result = wrap!(f, self, Statement, {
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
        });

        f.in_script_terminating_statement = was_in_script_terminating_statement;

        result
    }
}

impl<'arena> Format<'arena> for OpeningTag<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

impl<'arena> Format<'arena> for FullOpeningTag<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, FullOpeningTag, { Document::String("<?php") })
    }
}

impl<'arena> Format<'arena> for ShortOpeningTag {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ShortOpeningTag, { Document::String("<?") })
    }
}

impl<'arena> Format<'arena> for EchoOpeningTag {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EchoOpeningTag, { Document::String("<?=") })
    }
}

impl<'arena> Format<'arena> for ClosingTag {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        f.scripting_mode = false;

        wrap!(f, self, ClosingTag, {
            if f.settings.remove_trailing_close_tag
                && !f.in_script_terminating_statement
                && f.skip_spaces_and_new_lines(Some(self.span.end.offset), /* backwards */ false).is_none()
            {
                f.scripting_mode = true;

                Document::Trim(Trim::Newlines)
            } else {
                Document::Array(vec![
                    in f.arena;
                    Document::LineSuffixBoundary,
                    if f.is_at_start_of_line(self.span) { Document::empty() } else { Document::soft_space() },
                    Document::String("?>"),
                ])
            }
        })
    }
}

impl<'arena> Format<'arena> for Inline<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        f.scripting_mode = false;

        wrap!(f, self, Inline, {
            utils::replace_end_of_line(f, Document::String(self.value), Separator::LiteralLine, f.halted_compilation)
        })
    }
}

impl<'arena> Format<'arena> for Declare<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Declare, {
            let mut contents = vec![in f.arena; self.declare.format(f)];

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

impl<'arena> Format<'arena> for DeclareItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, DeclareItem, {
            Document::Array(vec![
                in f.arena;
                self.name.format(f),
                if f.settings.space_around_assignment_in_declare { Document::space() } else { Document::empty() },
                Document::String("="),
                if f.settings.space_around_assignment_in_declare { Document::space() } else { Document::empty() },
                self.value.format(f),
            ])
        })
    }
}

impl<'arena> Format<'arena> for DeclareBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

impl<'arena> Format<'arena> for DeclareColonDelimitedBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, DeclareColonDelimitedBody, {
            print_colon_delimited_body(f, &self.colon, &self.statements, &self.end_declare, &self.terminator)
        })
    }
}

impl<'arena> Format<'arena> for Namespace<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Namespace, {
            let mut parts = vec![in f.arena; self.namespace.format(f)];

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

impl<'arena> Format<'arena> for Identifier<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Identifier, {
            match self {
                Identifier::Local(i) => i.format(f),
                Identifier::Qualified(i) => i.format(f),
                Identifier::FullyQualified(i) => i.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for LocalIdentifier<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, LocalIdentifier, { Document::String(self.value) })
    }
}

impl<'arena> Format<'arena> for QualifiedIdentifier<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, QualifiedIdentifier, { Document::String(self.value) })
    }
}

impl<'arena> Format<'arena> for FullyQualifiedIdentifier<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, FullyQualifiedIdentifier, { Document::String(self.value) })
    }
}

impl<'arena> Format<'arena> for Use<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Use, {
            Document::Group(Group::new(vec![
                in f.arena;
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

impl<'arena> Format<'arena> for UseItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UseItem, {
            let mut parts = vec![in f.arena; self.name.format(f)];

            if let Some(alias) = &self.alias {
                parts.push(Document::space());
                parts.push(alias.format(f));
            }

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena> Format<'arena> for UseItemSequence<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UseItemSequence, {
            if f.settings.sort_uses {
                Document::Group(Group::new(vec![
                    in f.arena;
                    Document::Indent(Document::join(
                        f.arena,
                        statement::sort_use_items(self.items.iter()).into_iter().map(|i| i.format(f)),
                        Separator::CommaLine,
                    )),
                    Document::Line(Line::soft()),
                ]))
            } else {
                Document::Group(Group::new(vec![
                    in f.arena;
                    Document::Indent(Document::join(
                        f.arena,
                        self.items.iter().map(|i| i.format(f)),
                        Separator::CommaLine,
                    )),
                    Document::Line(Line::soft()),
                ]))
            }
        })
    }
}

impl<'arena> Format<'arena> for TypedUseItemList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TypedUseItemList, {
            let mut contents = vec![
                in f.arena;
                self.r#type.format(f),
                Document::space(),
                self.namespace.format(f),
                Document::String("\\"),
                Document::String("{"),
            ];

            if !self.items.is_empty() {
                let mut items: Vec<_> = if f.settings.sort_uses {
                    Document::join(
                        f.arena,
                        statement::sort_use_items(self.items.iter()).into_iter().map(|i| i.format(f)),
                        Separator::CommaLine,
                    )
                } else {
                    Document::join(f.arena, self.items.iter().map(|i| i.format(f)), Separator::CommaLine)
                };

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

impl<'arena> Format<'arena> for MixedUseItemList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MixedUseItemList, {
            let mut contents =
                vec![in f.arena; self.namespace.format(f), Document::String("\\"), Document::String("{")];

            if !self.items.is_empty() {
                let mut items = if f.settings.sort_uses {
                    Document::join(
                        f.arena,
                        statement::sort_maybe_typed_use_items(self.items.iter()).into_iter().map(|i| i.format(f)),
                        Separator::CommaLine,
                    )
                } else {
                    Document::join(f.arena, self.items.iter().map(|i| i.format(f)), Separator::CommaLine)
                };

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

impl<'arena> Format<'arena> for MaybeTypedUseItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MaybeTypedUseItem, {
            match &self.r#type {
                Some(t) => {
                    Document::Group(Group::new(vec![in f.arena; t.format(f), Document::space(), self.item.format(f)]))
                }
                None => self.item.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for TypedUseItemSequence<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TypedUseItemSequence, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.r#type.format(f),
                Document::space(),
                if f.settings.sort_uses {
                    Document::Indent(Document::join(
                        f.arena,
                        statement::sort_use_items(self.items.iter()).into_iter().map(|i| i.format(f)),
                        Separator::CommaLine,
                    ))
                } else {
                    Document::Indent(Document::join(f.arena, self.items.iter().map(|i| i.format(f)), Separator::CommaLine))
                },
                Document::Line(Line::soft())
            ]))
        })
    }
}

impl<'arena> Format<'arena> for UseItemAlias<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UseItemAlias, {
            Document::Group(Group::new(
                vec![in f.arena; self.r#as.format(f), Document::space(), self.identifier.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for UseType<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UseType, {
            match self {
                UseType::Function(keyword) => keyword.format(f),
                UseType::Const(keyword) => keyword.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for TraitUse<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUse, {
            let mut contents = vec![in f.arena; self.r#use.format(f), Document::space()];
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

impl<'arena> Format<'arena> for TraitUseSpecification<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUseSpecification, {
            match self {
                TraitUseSpecification::Abstract(s) => s.format(f),
                TraitUseSpecification::Concrete(s) => Document::Array(vec![in f.arena; Document::space(), s.format(f)]),
            }
        })
    }
}

impl<'arena> Format<'arena> for TraitUseAbstractSpecification<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUseAbstractSpecification, { self.0.format(f) })
    }
}

impl<'arena> Format<'arena> for TraitUseConcreteSpecification<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUseConcreteSpecification, {
            print_block_of_nodes(f, &self.left_brace, &self.adaptations, &self.right_brace, false)
        })
    }
}

impl<'arena> Format<'arena> for TraitUseAdaptation<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUseAdaptation, {
            match self {
                TraitUseAdaptation::Precedence(a) => a.format(f),
                TraitUseAdaptation::Alias(a) => a.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for TraitUseMethodReference<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUseMethodReference, {
            match self {
                TraitUseMethodReference::Identifier(m) => m.format(f),
                TraitUseMethodReference::Absolute(m) => m.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for TraitUseAbsoluteMethodReference<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUseAbsoluteMethodReference, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.trait_name.format(f),
                Document::String("::"),
                self.method_name.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for TraitUsePrecedenceAdaptation<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUsePrecedenceAdaptation, {
            let mut contents = vec![in f.arena; self.method_reference.format(f), Document::space(), self.insteadof.format(f), Document::space()];

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

impl<'arena> Format<'arena> for TraitUseAliasAdaptation<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TraitUseAliasAdaptation, {
            let mut parts = vec![in f.arena; self.method_reference.format(f), Document::space(), self.r#as.format(f)];

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

impl<'arena> Format<'arena> for ClassLikeConstant<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ClassLikeConstant, {
            let mut contents = vec![in f.arena];
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
                contents.push(Document::Indent(vec![in f.arena; Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    f.arena,
                    self.items.iter().map(|v| v.format(f)),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for ClassLikeConstantItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

impl<'arena> Format<'arena> for EnumCase<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EnumCase, {
            let mut parts = vec![in f.arena];
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

impl<'arena> Format<'arena> for EnumCaseItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EnumCaseItem, {
            match self {
                EnumCaseItem::Unit(c) => c.format(f),
                EnumCaseItem::Backed(c) => c.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for EnumCaseUnitItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EnumCaseUnitItem, { self.name.format(f) })
    }
}

impl<'arena> Format<'arena> for EnumCaseBackedItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EnumCaseBackedItem, {
            let lhs = self.name.format(f);
            let operator = Document::String("=");

            print_assignment(f, AssignmentLikeNode::EnumCaseBackedItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'arena> Format<'arena> for Property<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Property, {
            match self {
                Property::Plain(p) => p.format(f),
                Property::Hooked(p) => p.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for PlainProperty<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PlainProperty, {
            let mut contents = vec![in f.arena];
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
                let mut items = Document::join(f.arena, self.items.iter().map(|v| v.format(f)), Separator::CommaLine);

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

impl<'arena> Format<'arena> for HookedProperty<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, HookedProperty, {
            let mut contents = vec![in f.arena];
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
            contents.push(self.hook_list.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for PropertyItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyItem, {
            match self {
                PropertyItem::Abstract(p) => p.format(f),
                PropertyItem::Concrete(p) => p.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for PropertyAbstractItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyAbstractItem, { self.variable.format(f) })
    }
}

impl<'arena> Format<'arena> for PropertyConcreteItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyConcreteItem, {
            let lhs = self.variable.format(f);
            let operator = Document::String("=");

            print_assignment(f, AssignmentLikeNode::PropertyConcreteItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'arena> Format<'arena> for Method<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Method, {
            let mut attributes = vec![in f.arena];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let leading_comments = f.print_leading_comments(self.modifiers.first_span().unwrap_or(self.function.span));
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

            signature.push(self.parameter_list.format(f));
            if let Some(return_type) = &self.return_type_hint {
                signature.push(return_type.format(f));
            }

            let signature_id = f.next_id();
            let signature_document = Document::Group(Group::new(signature).with_id(signature_id));

            Document::Group(Group::new(vec![
                in f.arena;
                Document::Group(Group::new(attributes)),
                leading_comments.unwrap_or_else(Document::empty),
                signature_document,
                match &self.body {
                    MethodBody::Abstract(_) => self.body.format(f),
                    MethodBody::Concrete(block) => {
                        let is_constructor = self.name.value.eq_ignore_ascii_case("__construct");

                        let inlined_braces = if is_constructor {
                            f.settings.inline_empty_constructor_braces
                        } else {
                            f.settings.inline_empty_method_braces
                        } && block_is_empty(f, &block.left_brace, &block.right_brace);

                        Document::Group(Group::new(vec![
                            in f.arena;
                            if inlined_braces {
                                Document::space()
                            } else {
                                match f.settings.method_brace_style {
                                    BraceStyle::SameLine => Document::space(),
                                    BraceStyle::NextLine => {
                                        if !has_parameters_or_inner_parameter_comments {
                                            Document::Line(Line::hard())
                                        } else {
                                            Document::IfBreak(
                                                IfBreak::new(
                                                    f.arena,
                                                    Document::space(),
                                                    Document::Array(vec![
                                                        in f.arena;
                                                        Document::Line(Line::hard()),
                                                        Document::BreakParent,
                                                    ]),
                                                )
                                                .with_id(signature_id),
                                            )
                                        }
                                    }
                                }
                            },
                            self.body.format(f),
                        ]))
                    }
                },
            ]))
        })
    }
}

impl<'arena> Format<'arena> for MethodBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MethodBody, {
            match self {
                MethodBody::Abstract(b) => b.format(f),
                MethodBody::Concrete(b) => b.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for MethodAbstractBody {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MethodAbstractBody, { Document::String(";") })
    }
}

impl<'arena> Format<'arena> for Keyword<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Keyword, { Document::String(print_lowercase_keyword(f, self.value)) })
    }
}

impl<'arena> Format<'arena> for Terminator<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Terminator, {
            match self {
                Terminator::Semicolon(_) | Terminator::TagPair(_, _) => Document::String(";"),
                Terminator::ClosingTag(t) => Document::Array(vec![in f.arena; Document::space(), t.format(f)]),
            }
        })
    }
}

impl<'arena> Format<'arena> for ExpressionStatement<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ExpressionStatement, {
            Document::Array(vec![in f.arena; self.expression.format(f), self.terminator.format(f)])
        })
    }
}

impl<'arena> Format<'arena> for Extends<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Extends, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.extends.format(f),
                Document::Indent(vec![in f.arena; Document::Line(Line::default())]),
                Document::Indent(Document::join(
                    f.arena,
                    self.types.iter().map(|v| v.format(f)),
                    Separator::CommaLine,
                )),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Implements<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Implements, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.implements.format(f),
                Document::Indent(vec![in f.arena; Document::Line(Line::default())]),
                Document::Indent(Document::join(
                    f.arena,
                    self.types.iter().map(|v| v.format(f)),
                    Separator::CommaLine,
                )),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for ClassLikeMember<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

impl<'arena> Format<'arena> for Interface<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Interface, {
            let mut attributes = vec![in f.arena];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let signature = vec![
                in f.arena;
                self.interface.format(f),
                Document::space(),
                self.name.format(f),
                if let Some(e) = &self.extends {
                    Document::Array(vec![in f.arena; Document::space(), e.format(f)])
                } else {
                    Document::empty()
                },
            ];

            Document::Group(Group::new(vec![
                in f.arena;
                Document::Group(Group::new(attributes)),
                Document::Group(Group::new(signature)),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for EnumBackingTypeHint<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EnumBackingTypeHint, {
            Document::Group(Group::new(vec![
                in f.arena;
                format_token(f, self.colon, ":"),
                Document::space(),
                self.hint.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Class<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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
                in f.arena;
                Document::Group(Group::new(signature)),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]));

            if let Some(attributes) = attributes {
                Document::Array(vec![in f.arena; attributes, Document::Line(Line::hard()), class])
            } else {
                class
            }
        })
    }
}

impl<'arena> Format<'arena> for Trait<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Trait, {
            let mut attributes = vec![in f.arena];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            Document::Group(Group::new(vec![
                in f.arena;
                Document::Group(Group::new(attributes)),
                Document::Group(Group::new(vec![in f.arena; self.r#trait.format(f), Document::space(), self.name.format(f)])),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Enum<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Enum, {
            let mut attributes = vec![in f.arena];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let signature = vec![
                in f.arena;
                self.r#enum.format(f),
                Document::space(),
                self.name.format(f),
                if let Some(backing_type_hint) = &self.backing_type_hint {
                    backing_type_hint.format(f)
                } else {
                    Document::empty()
                },
                if let Some(i) = &self.implements {
                    Document::Array(vec![in f.arena; Document::space(), i.format(f)])
                } else {
                    Document::empty()
                },
            ];

            Document::Group(Group::new(vec![
                in f.arena;
                Document::Group(Group::new(attributes)),
                Document::Group(Group::new(signature)),
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, None),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Return<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Return, {
            let mut contents = vec![in f.arena; self.r#return.format(f)];

            if let Some(value) = &self.value {
                contents.push(Document::space());
                contents.push(format_return_value(f, value));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for Block<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Block, { block::print_block(f, &self.left_brace, &self.statements, &self.right_brace) })
    }
}

impl<'arena> Format<'arena> for Echo<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Echo, {
            let mut contents = vec![in f.arena; self.echo.format(f), Document::Indent(vec![in f.arena; Document::Line(Line::default())])];

            if !self.values.is_empty() {
                contents.push(Document::Indent(Document::join(
                    f.arena,
                    self.values.iter().map(|v| v.format(f)),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for ConstantItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ConstantItem, {
            let lhs = self.name.format(f);
            let operator = Document::String("=");

            print_assignment(f, AssignmentLikeNode::ConstantItem(self), lhs, operator, &self.value)
        })
    }
}

impl<'arena> Format<'arena> for Constant<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Constant, {
            let mut contents = vec![in f.arena];

            if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(Line::hard()));
            }

            contents.push(self.r#const.format(f));
            if self.items.len() == 1 {
                contents.push(Document::space());
                contents.push(self.items.as_slice()[0].format(f));
            } else if !self.items.is_empty() {
                contents.push(Document::Indent(vec![in f.arena; Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    f.arena,
                    self.items.iter().map(|v| v.format(f)),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for Attribute<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Attribute, { print_call_like_node(f, CallLikeNode::Attribute(self)) })
    }
}

impl<'arena> Format<'arena> for Hint<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Hint, {
            match self {
                Hint::Identifier(identifier) => identifier.format(f),
                Hint::Parenthesized(parenthesized_hint) => Document::Group(Group::new(vec![
                    in f.arena;
                    format_token(f, parenthesized_hint.left_parenthesis, "("),
                    parenthesized_hint.hint.format(f),
                    format_token(f, parenthesized_hint.right_parenthesis, ")"),
                ])),
                Hint::Nullable(nullable_hint) => {
                    // If the nullable type is nested inside another type hint,
                    // we cannot use `?` syntax.
                    let force_long_syntax = matches!(f.parent_node(), Node::Hint(_))
                        || (matches!(
                            nullable_hint.hint,
                            Hint::Nullable(_) | Hint::Union(_) | Hint::Intersection(_) | Hint::Parenthesized(_)
                        ));

                    if force_long_syntax {
                        return Document::Group(Group::new(vec![
                            in f.arena;
                            Document::String("null"),
                            Document::String("|"),
                            nullable_hint.hint.format(f),
                        ]));
                    }

                    match f.settings.null_type_hint {
                        NullTypeHint::NullPipe => Document::Group(Group::new(vec![
                            in f.arena;
                            Document::String("null"),
                            Document::String("|"),
                            nullable_hint.hint.format(f),
                        ])),
                        NullTypeHint::Question => Document::Group(Group::new(
                            vec![in f.arena; Document::String("?"), nullable_hint.hint.format(f)],
                        )),
                    }
                }
                Hint::Union(union_hint) => {
                    let force_long_syntax = matches!(f.parent_node(), Node::Hint(_))
                        || matches!(
                            union_hint.left,
                            Hint::Nullable(_) | Hint::Union(_) | Hint::Intersection(_) | Hint::Parenthesized(_)
                        )
                        || matches!(
                            union_hint.right,
                            Hint::Nullable(_) | Hint::Union(_) | Hint::Intersection(_) | Hint::Parenthesized(_)
                        );

                    if !force_long_syntax {
                        if let Hint::Null(_) = union_hint.left
                            && f.settings.null_type_hint.is_question()
                        {
                            return Document::Group(Group::new(vec![
                                in f.arena;
                                Document::String("?"),
                                union_hint.right.format(f),
                            ]));
                        }

                        if let Hint::Null(_) = union_hint.right
                            && f.settings.null_type_hint.is_question()
                        {
                            return Document::Group(Group::new(
                                vec![in f.arena; Document::String("?"), union_hint.left.format(f)],
                            ));
                        }
                    }

                    Document::Group(Group::new(vec![
                        in f.arena;
                        union_hint.left.format(f),
                        format_token(f, union_hint.pipe, "|"),
                        union_hint.right.format(f),
                    ]))
                }
                Hint::Intersection(intersection_hint) => Document::Group(Group::new(vec![
                    in f.arena;
                    intersection_hint.left.format(f),
                    format_token(f, intersection_hint.ampersand, "&"),
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

impl<'arena> Format<'arena> for Modifier<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

impl<'arena> Format<'arena> for AttributeList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
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

            let mut contents = vec![in f.arena; Document::String("#[")];
            if let Some(trailing_comments) = f.print_trailing_comments(self.hash_left_bracket) {
                contents.push(trailing_comments);
            }

            if should_inline {
                contents.push(self.attributes.as_slice()[0].format(f));
            } else {
                contents.push(Document::Indent({
                    let mut attributes = Document::join(
                        f.arena,
                        self.attributes.iter().map(|a| Document::Group(Group::new(vec![in f.arena; a.format(f)]))),
                        Separator::CommaLine,
                    );

                    attributes.insert(0, Document::Line(Line::soft()));

                    attributes
                }));
            }

            if !should_inline {
                if f.settings.trailing_comma {
                    contents.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
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

impl<'arena> Format<'arena> for PropertyHookAbstractBody {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyHookAbstractBody, { Document::String(";") })
    }
}

impl<'arena> Format<'arena> for PropertyHookConcreteBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyHookConcreteBody, {
            Document::Group(Group::new(vec![
                in f.arena;
                Document::space(),
                match self {
                    PropertyHookConcreteBody::Block(b) => b.format(f),
                    PropertyHookConcreteBody::Expression(b) => b.format(f),
                },
            ]))
        })
    }
}

impl<'arena> Format<'arena> for PropertyHookConcreteExpressionBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyHookConcreteExpressionBody, {
            Document::Group(Group::new(vec![
                in f.arena;
                Document::String("=>"),
                Document::space(),
                self.expression.format(f),
                Document::String(";"),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for PropertyHookBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyHookBody, {
            match self {
                PropertyHookBody::Abstract(b) => b.format(f),
                PropertyHookBody::Concrete(b) => b.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for PropertyHook<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyHook, {
            let mut contents = vec![in f.arena];
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

impl<'arena> Format<'arena> for PropertyHookList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyHookList, {
            Document::Group(Group::new(vec![
                in f.arena;
                Document::String("{"),
                f.print_trailing_comments(self.left_brace).unwrap_or_else(Document::empty),
                if self.hooks.is_empty() {
                    Document::empty()
                } else {
                    Document::Indent(vec![
                        in f.arena;
                        Document::Line(Line::hard()),
                        Document::Array(Document::join(
                            f.arena,
                            self.hooks.iter().map(|hook| hook.format(f)),
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

impl<'arena> Format<'arena> for FunctionLikeParameterDefaultValue<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, FunctionLikeParameterDefaultValue, {
            Document::Group(Group::new(vec![in f.arena; Document::String("= "), self.value.format(f)]))
        })
    }
}

impl<'arena> Format<'arena> for FunctionLikeParameter<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, FunctionLikeParameter, {
            let mut contents = vec![in f.arena];
            if let Some(attributes) = print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(if f.parameter_state.force_break {
                    Line::hard()
                } else {
                    Line::default()
                }));
            }

            contents.extend(print_modifiers(f, &self.modifiers));
            if !self.modifiers.is_empty() {
                contents.push(Document::space());
            }

            if let Some(hint) = &self.hint {
                contents.push(hint.format(f));
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

impl<'arena> Format<'arena> for FunctionLikeParameterList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, FunctionLikeParameterList, { print_function_like_parameters(f, self) })
    }
}

impl<'arena> Format<'arena> for FunctionLikeReturnTypeHint<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, FunctionLikeReturnTypeHint, {
            Document::Group(Group::new(vec![
                in f.arena;
                format_token(f, self.colon, ":"),
                Document::space(),
                self.hint.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Function<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Function, {
            let mut attributes = vec![in f.arena];
            for attribute_list in self.attribute_lists.iter() {
                attributes.push(attribute_list.format(f));
                attributes.push(Document::Line(Line::hard()));
            }

            let leading_comments = f.print_leading_comments(self.function.span);
            let mut signature = vec![in f.arena];
            signature.push(self.function.format(f));
            signature.push(Document::space());
            if self.ampersand.is_some() {
                signature.push(Document::String("&"));
            }

            signature.push(self.name.format(f));
            let has_parameters_or_inner_parameter_comments =
                !self.parameter_list.parameters.is_empty() || f.has_inner_comment(self.parameter_list.span());

            signature.push(self.parameter_list.format(f));
            if let Some(return_type) = &self.return_type_hint {
                signature.push(return_type.format(f));
            }

            let signature_id = f.next_id();
            let signature_document = Document::Group(Group::new(signature).with_id(signature_id));

            let inlined_braces = f.settings.inline_empty_function_braces
                && block_is_empty(f, &self.body.left_brace, &self.body.right_brace);

            Document::Group(Group::new(vec![
                in f.arena;
                Document::Group(Group::new(attributes)),
                leading_comments.unwrap_or_else(Document::empty),
                signature_document,
                Document::Group(Group::new(vec![
                    in f.arena;
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
                                            f.arena,
                                            Document::space(),
                                            Document::Array(vec![in f.arena; Document::Line(Line::hard()), Document::BreakParent]),
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

impl<'arena> Format<'arena> for Try<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Try, {
            let mut parts = vec![in f.arena; self.r#try.format(f), Document::space(), self.block.format(f)];

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

impl<'arena> Format<'arena> for TryCatchClause<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TryCatchClause, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.catch.format(f),
                Document::space(),
                format_token(f, self.left_parenthesis, "("),
                Document::Group(Group::new({
                    let mut context = vec![in f.arena; self.hint.format(f)];
                    if let Some(variable) = &self.variable {
                        context.push(Document::space());
                        context.push(variable.format(f));
                    }

                    context
                })),
                format_token(f, self.right_parenthesis, ")"),
                Document::space(),
                self.block.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for TryFinallyClause<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, TryFinallyClause, {
            Document::Group(Group::new(
                vec![in f.arena; self.finally.format(f), Document::space(), self.block.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for Global<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Global, {
            let mut contents = vec![in f.arena; self.global.format(f)];

            if self.variables.len() == 1 {
                contents.push(Document::space());
                contents.push(self.variables.as_slice()[0].format(f));
            } else if !self.variables.is_empty() {
                contents.push(Document::Indent(vec![in f.arena; Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    f.arena,
                    self.variables.iter().map(|v| v.format(f)),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for StaticAbstractItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, StaticAbstractItem, { self.variable.format(f) })
    }
}

impl<'arena> Format<'arena> for StaticConcreteItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, StaticConcreteItem, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.variable.format(f),
                Document::space(),
                Document::String("="),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for StaticItem<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, StaticItem, {
            match self {
                StaticItem::Abstract(i) => i.format(f),
                StaticItem::Concrete(i) => i.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for Static<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Static, {
            let mut contents = vec![in f.arena; self.r#static.format(f)];

            if self.items.len() == 1 {
                contents.push(Document::space());
                contents.push(self.items.as_slice()[0].format(f));
            } else if !self.items.is_empty() {
                contents.push(Document::Indent(vec![in f.arena; Document::Line(Line::default())]));

                contents.push(Document::Indent(Document::join(
                    f.arena,
                    self.items.iter().map(|v| v.format(f)),
                    Separator::CommaLine,
                )));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(self.terminator.format(f));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for Unset<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Unset, {
            let mut contents = vec![in f.arena; self.unset.format(f), Document::String("(")];

            if !self.values.is_empty() {
                let mut values = Document::join(f.arena, self.values.iter().map(|v| v.format(f)), Separator::CommaLine);

                if f.settings.trailing_comma {
                    values.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
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

impl<'arena> Format<'arena> for Goto<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Goto, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.goto.format(f),
                Document::space(),
                self.label.format(f),
                self.terminator.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Label<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Label, {
            Document::Group(Group::new(vec![in f.arena; self.name.format(f), Document::String(":")]))
        })
    }
}

impl<'arena> Format<'arena> for HaltCompiler<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        f.scripting_mode = false;
        f.halted_compilation = true;

        wrap!(f, self, HaltCompiler, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.halt_compiler.format(f),
                Document::String("("),
                Document::String(")"),
                self.terminator.format(f),
            ]))
        })
    }
}

fn format_token<'arena>(f: &mut FormatterState<'_, 'arena>, span: Span, token_value: &'arena str) -> Document<'arena> {
    let leading = f.print_leading_comments(span);
    let trailing = f.print_trailing_comments(span);

    f.print_comments(leading, Document::String(token_value), trailing)
}
