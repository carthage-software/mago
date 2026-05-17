use bumpalo::vec;

use mago_span::HasSpan;
use mago_syntax::ast::GenericArgumentList;
use mago_syntax::ast::GenericParameter;
use mago_syntax::ast::GenericParameterBound;
use mago_syntax::ast::GenericParameterDefault;
use mago_syntax::ast::GenericParameterList;
use mago_syntax::ast::GenericVariance;
use mago_syntax::ast::Turbofish;

use crate::document::Document;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::wrap;

impl<'arena> Format<'arena> for GenericParameterList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, GenericParameterList, {
            let mut parts = vec![in f.arena; Document::String("<")];
            let mut first = true;
            for parameter in self.parameters.iter() {
                if !first {
                    parts.push(Document::String(", "));
                }
                first = false;
                parts.push(parameter.format(f));
            }
            parts.push(Document::String(">"));
            Document::Array(parts)
        })
    }
}

impl<'arena> Format<'arena> for GenericParameter<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, GenericParameter, {
            let mut parts = vec![in f.arena];
            if let Some(variance) = &self.variance {
                parts.push(Document::String(match variance {
                    GenericVariance::Covariant(_) => "+",
                    GenericVariance::Contravariant(_) => "-",
                }));
            }
            parts.push(self.name.format(f));
            if let Some(bound) = &self.bound {
                parts.push(bound.format(f));
            }
            if let Some(default) = &self.default {
                parts.push(default.format(f));
            }
            Document::Array(parts)
        })
    }
}

impl<'arena> Format<'arena> for GenericParameterBound<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, GenericParameterBound, {
            Document::Array(vec![in f.arena; Document::String(": "), self.hint.format(f)])
        })
    }
}

impl<'arena> Format<'arena> for GenericParameterDefault<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, GenericParameterDefault, {
            Document::Array(vec![in f.arena; Document::String(" = "), self.hint.format(f)])
        })
    }
}

impl<'arena> Format<'arena> for GenericArgumentList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, GenericArgumentList, {
            let mut parts = vec![in f.arena; Document::String("<")];
            let mut first = true;
            for argument in self.arguments.iter() {
                if !first {
                    parts.push(Document::String(", "));
                }
                first = false;
                parts.push(argument.format(f));
            }
            parts.push(Document::String(">"));
            Document::Array(parts)
        })
    }
}

impl<'arena> Format<'arena> for Turbofish<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Turbofish, {
            let mut parts = vec![in f.arena; Document::String("::<")];
            let mut first = true;
            for argument in self.arguments.iter() {
                if !first {
                    parts.push(Document::String(", "));
                }
                first = false;
                parts.push(argument.format(f));
            }
            parts.push(Document::String(">"));
            Document::Array(parts)
        })
    }
}
