use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::delimited::Delimited;
use crate::ir::item::member::trait_use::TraitUse;
use crate::ir::item::member::trait_use::TraitUseAdaptation;
use crate::ir::item::member::trait_use::TraitUseAliasAdaptation;
use crate::ir::item::member::trait_use::TraitUsePrecedenceAdaptation;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_trait_use(
        &mut self,
        trait_use: &'scratch cst::TraitUse<'scratch>,
    ) -> TraitUse<'arena, (), (), ()> {
        let traits = self.lower_class_reference_list(&trait_use.trait_names);
        let adaptations = match &trait_use.specification {
            cst::TraitUseSpecification::Abstract(_) => None,
            cst::TraitUseSpecification::Concrete(concrete) => Some(Delimited {
                span: concrete.left_brace.join(concrete.right_brace),
                items: self.arena.alloc_slice_fill_iter(
                    concrete.adaptations.iter().map(|adaptation| self.lower_trait_use_adaptation(adaptation)),
                ),
            }),
        };

        let document = self.phpdoc_resolution.get(trait_use.span());
        let annotation = self.lower_item_annotation(document.as_ref(), None);

        TraitUse { span: trait_use.span(), annotation, traits, adaptations }
    }

    fn lower_trait_use_adaptation(
        &mut self,
        adaptation: &'scratch cst::TraitUseAdaptation<'scratch>,
    ) -> TraitUseAdaptation<'arena> {
        match adaptation {
            cst::TraitUseAdaptation::Precedence(precedence) => {
                let r#trait =
                    self.lower_identifier(&precedence.method_reference.trait_name, Some(NameResolutionKind::Default));
                let method = self.lower_name(&precedence.method_reference.method_name);
                let instead_of = self.lower_class_reference_list(&precedence.trait_names);

                TraitUseAdaptation::Precedence(TraitUsePrecedenceAdaptation {
                    span: precedence.span(),
                    r#trait,
                    method,
                    instead_of,
                })
            }
            cst::TraitUseAdaptation::Alias(alias) => {
                let (r#trait, method) = match &alias.method_reference {
                    cst::TraitUseMethodReference::Identifier(identifier) => (None, self.lower_name(identifier)),
                    cst::TraitUseMethodReference::Absolute(absolute) => (
                        Some(self.lower_identifier(&absolute.trait_name, Some(NameResolutionKind::Default))),
                        self.lower_name(&absolute.method_name),
                    ),
                };
                let modifier = alias.modifier.as_ref().map(|modifier| self.lower_modifier(modifier));
                let new_alias = match &alias.alias {
                    Some(identifier) => self.lower_name(identifier),
                    None => method,
                };

                TraitUseAdaptation::Alias(TraitUseAliasAdaptation {
                    span: alias.span(),
                    r#trait,
                    method,
                    modifier,
                    alias: new_alias,
                })
            }
        }
    }
}
