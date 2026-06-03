use bumpalo::collections::Vec;

use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::tag::TagValue;
use mago_phpdoc_syntax::cst::tag::TagVendor;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use crate::lower::Lowering;

pub(crate) struct VariableBindings<'arena> {
    pub(crate) unnamed: Option<&'arena TypeAnnotation<'arena>>,
    pub(crate) named: Vec<'arena, (DirectVariable<'arena>, &'arena TypeAnnotation<'arena>, Option<TagVendor>, Span)>,
}

impl<'arena> VariableBindings<'arena> {
    pub(crate) fn is_empty(&self) -> bool {
        self.unnamed.is_none() && self.named.is_empty()
    }

    pub(crate) fn take_unnamed(&mut self) -> Option<&'arena TypeAnnotation<'arena>> {
        self.unnamed.take()
    }

    pub(crate) fn take_named(&mut self, name: &[u8]) -> Option<&'arena TypeAnnotation<'arena>> {
        let position = self.named.iter().position(|entry| entry.0.name == name)?;

        Some(self.named.remove(position).1)
    }
}

impl<'arena> Lowering<'arena> {
    pub(crate) fn collect_var_bindings(&self, document: Option<&Document<'arena>>) -> VariableBindings<'arena> {
        let mut named = Vec::new_in(self.arena);
        let mut unnamed = None;
        let mut unnamed_vendor = None;

        let Some(document) = document else {
            return VariableBindings { unnamed, named };
        };

        for element in document.elements.iter() {
            let Element::Tag(tag) = element else { continue };
            let tag = *tag;
            let TagValue::Var(value) = &tag.value else { continue };

            match value.variable {
                Some(variable) => {
                    let variable = DirectVariable { span: variable.span, name: variable.value };
                    if let Some(entry) = named.iter_mut().find(|entry| entry.0.name == variable.name) {
                        if tag.vendor > entry.2 {
                            entry.1 = self.lower_type_annotation(value.r#type);
                            entry.2 = tag.vendor;
                        }
                    } else {
                        named.push((variable, self.lower_type_annotation(value.r#type), tag.vendor, tag.span()));
                    }
                }
                None => {
                    if unnamed.is_none() || tag.vendor > unnamed_vendor {
                        unnamed_vendor = tag.vendor;
                        unnamed = Some(self.lower_type_annotation(value.r#type));
                    }
                }
            }
        }

        VariableBindings { unnamed, named }
    }
}
