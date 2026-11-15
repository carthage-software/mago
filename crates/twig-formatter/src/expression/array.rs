use crate::document::DocumentBuilder;
use crate::safety::ambiguous_name;
use mago_allocator::Arena;
use mago_formatter_core::document::Document;
use mago_span::HasSpan;
use mago_twig_syntax::cst::{Array, ArrayElement, Expression, HashMap};

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn array_expression(&self, array: &Array<'arena>) -> Document<'arena, A> {
        if array.elements.iter().any(|element| matches!(element, ArrayElement::Missing(_))) {
            return self.raw(array.span());
        }

        let contents = self.separated(&array.elements, |element| match element {
            ArrayElement::Value(element) => self.expression(element.value),
            ArrayElement::Variadic(element) => self.spread(element.ellipsis, element.value),
            ArrayElement::Missing(_) => Document::empty(),
        });

        let first =
            array.elements.iter().next().map_or(array.right_bracket.start_offset(), |element| element.start_offset());
        self.preserved_group(
            self.enclosed(self.raw(array.left_bracket), contents, self.raw(array.right_bracket)),
            self.breaks_between(array.left_bracket.end_offset(), first),
        )
    }

    pub(super) fn hash_map(&self, map: &HashMap<'arena>) -> Document<'arena, A> {
        let contents = self.separated(&map.entries, |entry| {
            if let Some(ellipsis) = entry.ellipsis {
                return self.spread(ellipsis, &entry.value);
            }

            if let Some(key) = &entry.key
                && let Some(colon) = entry.colon
            {
                let separator = if matches!(key, Expression::String(name)
                    if ambiguous_name(name.raw) && name.span.end < colon.start)
                {
                    b" : ".as_slice()
                } else {
                    b": ".as_slice()
                };

                self.array([self.expression(key), Document::String(separator), self.expression(&entry.value)])
            } else {
                self.expression(&entry.value)
            }
        });

        let first = map.entries.iter().next().map_or(map.right_brace.start_offset(), |entry| entry.start_offset());
        self.preserved_group(
            self.enclosed(self.raw(map.left_brace), contents, self.raw(map.right_brace)),
            self.breaks_between(map.left_brace.end_offset(), first),
        )
    }
}
