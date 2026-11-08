use crate::document::DocumentBuilder;
use crate::safety::ambiguous_name;
use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::document::Document;
use mago_formatter_core::document::{Group, Line};
use mago_span::HasSpan;
use mago_twig_syntax::cst::{
    Argument, ArgumentList, ArrowFunction, Filter, NamedArgument, NamedArgumentSeparator, Test, TestArguments,
};

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(crate) fn arguments(&self, arguments: &ArgumentList<'arena>) -> Document<'arena, A> {
        let contents = self.separated(&arguments.arguments, |argument| match argument {
            Argument::Positional(argument) => argument
                .ellipsis
                .map_or_else(|| self.expression(argument.value), |ellipsis| self.spread(ellipsis, argument.value)),
            Argument::Named(argument) => self.array([
                self.raw(argument.name.span()),
                Document::String(self.named_separator(argument)),
                self.expression(argument.value),
            ]),
        });

        self.enclosed(self.raw(arguments.left_parenthesis), contents, self.raw(arguments.right_parenthesis))
    }

    fn named_separator(&self, argument: &NamedArgument<'arena>) -> &'static [u8] {
        let space_before = if ambiguous_name(argument.name.value) {
            argument.name.span.end < argument.separator.span().start
        } else {
            matches!(argument.separator, NamedArgumentSeparator::Equal(_))
        };

        match (argument.separator, space_before) {
            (NamedArgumentSeparator::Equal(_), true) => b" = ",
            (NamedArgumentSeparator::Equal(_), false) => b"= ",
            (NamedArgumentSeparator::Colon(_), true) => b" : ",
            (NamedArgumentSeparator::Colon(_), false) => b": ",
        }
    }

    pub(super) fn filter(&self, filter: &Filter<'arena>) -> Document<'arena, A> {
        self.array([
            self.expression(filter.operand),
            self.raw(filter.pipe),
            self.raw(filter.name.span()),
            filter.argument_list.as_ref().map_or_else(Document::empty, |arguments| self.arguments(arguments)),
        ])
    }

    pub(super) fn test(&self, test: &Test<'arena>) -> Document<'arena, A> {
        let capacity = 3
            + usize::from(test.not_keyword.is_some())
            + usize::from(test.second_word.is_some()) * 2
            + match &test.arguments {
                TestArguments::None => 0,
                TestArguments::Parenthesised(_) => 1,
                TestArguments::Bare(_) => 2,
            };

        let mut documents = Vec::with_capacity_in(capacity, self.arena);
        documents.push(self.expression(test.operand));
        documents.push(Document::String(b" is "));
        if test.not_keyword.is_some() {
            documents.push(Document::String(b"not "));
        }

        documents.push(self.raw(test.name.span()));
        if let Some(second_word) = test.second_word {
            documents.push(Document::space());
            documents.push(self.raw(second_word.span()));
        }

        match &test.arguments {
            TestArguments::None => {}
            TestArguments::Parenthesised(arguments) => documents.push(self.arguments(arguments)),
            TestArguments::Bare(expression) => {
                documents.push(Document::space());
                documents.push(self.expression(expression));
            }
        }

        Document::Group(Group::new(documents))
    }

    pub(super) fn arrow(&self, arrow: &ArrowFunction<'arena>) -> Document<'arena, A> {
        let parameters = self.separated(&arrow.parameters, |parameter| self.raw(parameter.span()));
        let parameters = if let (Some(open), Some(close)) = (arrow.left_parenthesis, arrow.right_parenthesis) {
            self.enclosed(self.raw(open), parameters, self.raw(close))
        } else {
            parameters
        };

        self.group([
            parameters,
            Document::String(b" =>"),
            Document::Indent(self.list([Document::Line(Line::default()), self.expression(arrow.body)])),
        ])
    }
}
