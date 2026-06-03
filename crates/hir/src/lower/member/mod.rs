use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::flags::Flags;
use crate::ir::member::ClassLikeConstant;
use crate::ir::member::ClassLikeConstantItem;
use crate::ir::member::EnumCase;
use crate::ir::member::HookedProperty;
use crate::ir::member::Method;
use crate::ir::member::Property;
use crate::ir::member::PropertyItem;
use crate::ir::member::TraitUse;
use crate::ir::member::TraitUseAdaptation;
use crate::ir::member::TraitUseAliasAdaptation;
use crate::ir::member::TraitUsePrecedenceAdaptation;
use crate::lower::Lowering;
use crate::lower::resolution::kind::ResolutionKind;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_method(&mut self, method: &'arena cst::Method<'arena>) -> Method<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&method.attribute_lists);
        let modifiers = self.lower_modifiers(&method.modifiers);
        let name = self.lower_name(&method.name);
        let parameters = self.lower_parameter_list(&method.parameter_list);
        let return_type = method.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));
        let body = match &method.body {
            cst::MethodBody::Abstract(_) => None,
            cst::MethodBody::Concrete(block) => {
                Some(self.statements_to_statement(block.statements.as_slice(), block.span()))
            }
        };

        Method {
            span: method.span(),
            attributes,
            flags: Flags::new(),
            modifiers,
            name,
            type_parameter_annotations: &[],
            parameters,
            where_constraint_annotations: &[],
            return_by_reference: method.ampersand.is_some(),
            return_type,
            return_type_annotation: None,
            throws: &[],
            asserts: &[],
            asserts_if_true: &[],
            asserts_if_false: &[],
            body,
        }
    }

    pub(crate) fn lower_plain_property(
        &mut self,
        property: &'arena cst::PlainProperty<'arena>,
    ) -> Property<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&property.attribute_lists);
        let modifiers = self.lower_modifiers(&property.modifiers);
        let r#type = property.hint.as_ref().map(|hint| self.lower_type(hint));
        let items = self.arena.alloc_slice_fill_iter(property.items.iter().map(|item| self.lower_property_item(item)));

        Property {
            span: property.span(),
            attributes,
            flags: Flags::new(),
            modifiers,
            r#type,
            type_annotation: None,
            items,
        }
    }

    pub(crate) fn lower_hooked_property(
        &mut self,
        property: &'arena cst::HookedProperty<'arena>,
    ) -> HookedProperty<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&property.attribute_lists);
        let modifiers = self.lower_modifiers(&property.modifiers);
        let r#type = property.hint.as_ref().map(|hint| self.lower_type(hint));
        let item = self.lower_property_item(&property.item);
        let hooks = self.lower_property_hooks(&property.hook_list);

        HookedProperty {
            span: property.span(),
            attributes,
            flags: Flags::new(),
            modifiers,
            r#type,
            type_annotation: None,
            item,
            hooks,
        }
    }

    fn lower_property_item(&mut self, item: &'arena cst::PropertyItem<'arena>) -> PropertyItem<'arena, (), (), ()> {
        match item {
            cst::PropertyItem::Abstract(item) => {
                PropertyItem { variable: self.lower_direct_variable(&item.variable), default_value: None }
            }
            cst::PropertyItem::Concrete(item) => PropertyItem {
                variable: self.lower_direct_variable(&item.variable),
                default_value: Some(self.arena.alloc(self.lower_expression(item.value))),
            },
        }
    }

    pub(crate) fn lower_class_like_constant(
        &mut self,
        constant: &'arena cst::ClassLikeConstant<'arena>,
    ) -> ClassLikeConstant<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&constant.attribute_lists);
        let modifiers = self.lower_modifiers(&constant.modifiers);
        let r#type = constant.hint.as_ref().map(|hint| self.lower_type(hint));
        let items = self
            .arena
            .alloc_slice_fill_iter(constant.items.iter().map(|item| self.lower_class_like_constant_item(item)));

        ClassLikeConstant { span: constant.span(), attributes, modifiers, r#type, type_annotation: None, items }
    }

    fn lower_class_like_constant_item(
        &mut self,
        item: &'arena cst::ClassLikeConstantItem<'arena>,
    ) -> ClassLikeConstantItem<'arena, (), (), ()> {
        ClassLikeConstantItem {
            name: self.lower_name(&item.name),
            value: self.arena.alloc(self.lower_expression(item.value)),
        }
    }

    pub(crate) fn lower_enum_case(&mut self, enum_case: &'arena cst::EnumCase<'arena>) -> EnumCase<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&enum_case.attribute_lists);
        let span = enum_case.span();

        match &enum_case.item {
            cst::EnumCaseItem::Unit(unit) => {
                EnumCase { span, attributes, name: self.lower_name(&unit.name), value: None }
            }
            cst::EnumCaseItem::Backed(backed) => EnumCase {
                span,
                attributes,
                name: self.lower_name(&backed.name),
                value: Some(self.arena.alloc(self.lower_expression(backed.value))),
            },
        }
    }

    pub(crate) fn lower_trait_use(&self, trait_use: &'arena cst::TraitUse<'arena>) -> TraitUse<'arena> {
        let traits = self.lower_class_reference_list(&trait_use.trait_names);
        let adaptations = match &trait_use.specification {
            cst::TraitUseSpecification::Abstract(_) => &[],
            cst::TraitUseSpecification::Concrete(concrete) => &*self.arena.alloc_slice_fill_iter(
                concrete.adaptations.iter().map(|adaptation| self.lower_trait_use_adaptation(adaptation)),
            ),
        };

        TraitUse { span: trait_use.span(), use_annotation: &[], traits, adaptations }
    }

    fn lower_trait_use_adaptation(
        &self,
        adaptation: &'arena cst::TraitUseAdaptation<'arena>,
    ) -> TraitUseAdaptation<'arena> {
        match adaptation {
            cst::TraitUseAdaptation::Precedence(precedence) => {
                let r#trait =
                    self.lower_identifier(&precedence.method_reference.trait_name, Some(ResolutionKind::Default));
                let method = self.lower_name(&precedence.method_reference.method_name);
                let instead_of = self.lower_class_reference_list(&precedence.trait_names);

                TraitUseAdaptation::Precedence(TraitUsePrecedenceAdaptation { r#trait, method, instead_of })
            }
            cst::TraitUseAdaptation::Alias(alias) => {
                let (r#trait, method) = match &alias.method_reference {
                    cst::TraitUseMethodReference::Identifier(identifier) => (None, self.lower_name(identifier)),
                    cst::TraitUseMethodReference::Absolute(absolute) => (
                        Some(self.lower_identifier(&absolute.trait_name, Some(ResolutionKind::Default))),
                        self.lower_name(&absolute.method_name),
                    ),
                };
                let visibility = alias.visibility.as_ref().map(|modifier| self.lower_modifier(modifier));
                let new_alias = match &alias.alias {
                    Some(identifier) => self.lower_name(identifier),
                    None => method,
                };

                TraitUseAdaptation::Alias(TraitUseAliasAdaptation { r#trait, method, visibility, alias: new_alias })
            }
        }
    }
}
