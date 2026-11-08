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

        self.enclosed(self.raw(array.left_bracket), contents, self.raw(array.right_bracket))
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

        self.enclosed(self.raw(map.left_brace), contents, self.raw(map.right_brace))
    }
}
