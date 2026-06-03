use bumpalo::collections::Vec;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::flags::Flags;
use crate::ir::statement::definition::Class;
use crate::ir::statement::definition::Constant;
use crate::ir::statement::definition::ConstantItem;
use crate::ir::statement::definition::Enum;
use crate::ir::statement::definition::EnumBackingType;
use crate::ir::statement::definition::Function;
use crate::ir::statement::definition::Interface;
use crate::ir::statement::definition::Trait;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_class(&mut self, class: &'arena cst::Class<'arena>) -> &'arena Class<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&class.attribute_lists);
        let modifiers = self.lower_modifiers(&class.modifiers);
        let name = self.lower_declaration_name(&class.name);
        let extends = class.extends.as_ref().and_then(|extends| self.lower_extends_one(extends));
        let implements = class.implements.as_ref().map(|implements| self.lower_implements(implements));

        let mut trait_uses = Vec::new_in(self.arena);
        let mut constants = Vec::new_in(self.arena);
        let mut properties = Vec::new_in(self.arena);
        let mut hooked_properties = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);

        for member in class.members.iter() {
            match member {
                cst::ClassLikeMember::TraitUse(trait_use) => trait_uses.push(self.lower_trait_use(trait_use)),
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::Property(cst::Property::Plain(property)) => {
                    properties.push(self.lower_plain_property(property));
                }
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    hooked_properties.push(self.lower_hooked_property(property));
                }
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method)),
                cst::ClassLikeMember::EnumCase(_) => {}
            }
        }

        self.arena.alloc(Class {
            flags: Flags::new(),
            attributes,
            name,
            type_parameter_annotations: &[],
            modifiers,
            type_alias_annotations: &[],
            imported_type_alias_annotations: &[],
            extends,
            extends_annotations: &[],
            implements,
            implements_annotations: &[],
            sealed_annotation: None,
            mixin_annotations: &[],
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            properties: properties.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            property_annotations: &[],
            methods: methods.into_bump_slice(),
            method_annotations: &[],
        })
    }

    pub(crate) fn lower_interface(
        &mut self,
        interface: &'arena cst::Interface<'arena>,
    ) -> &'arena Interface<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&interface.attribute_lists);
        let name = self.lower_declaration_name(&interface.name);
        let extends = interface.extends.as_ref().map(|extends| self.lower_extends_one_or_more(extends));

        let mut constants = Vec::new_in(self.arena);
        let mut hooked_properties = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);

        for member in interface.members.iter() {
            match member {
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    hooked_properties.push(self.lower_hooked_property(property));
                }
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method)),
                _ => {}
            }
        }

        self.arena.alloc(Interface {
            flags: Flags::new(),
            attributes,
            name,
            type_parameter_annotations: &[],
            type_alias_annotations: &[],
            imported_type_alias_annotations: &[],
            extends,
            extends_annotations: &[],
            sealed_annotation: None,
            mixin_annotations: &[],
            constants: constants.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            methods: methods.into_bump_slice(),
            method_annotations: &[],
        })
    }

    pub(crate) fn lower_trait(&mut self, r#trait: &'arena cst::Trait<'arena>) -> &'arena Trait<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&r#trait.attribute_lists);
        let name = self.lower_declaration_name(&r#trait.name);

        let mut trait_uses = Vec::new_in(self.arena);
        let mut constants = Vec::new_in(self.arena);
        let mut properties = Vec::new_in(self.arena);
        let mut hooked_properties = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);

        for member in r#trait.members.iter() {
            match member {
                cst::ClassLikeMember::TraitUse(trait_use) => trait_uses.push(self.lower_trait_use(trait_use)),
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::Property(cst::Property::Plain(property)) => {
                    properties.push(self.lower_plain_property(property));
                }
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    hooked_properties.push(self.lower_hooked_property(property));
                }
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method)),
                cst::ClassLikeMember::EnumCase(_) => {}
            }
        }

        self.arena.alloc(Trait {
            flags: Flags::new(),
            attributes,
            name,
            type_parameter_annotations: &[],
            type_alias_annotations: &[],
            imported_type_alias_annotations: &[],
            require_extends_annotations: &[],
            require_implements_annotations: &[],
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            properties: properties.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            property_annotations: &[],
            methods: methods.into_bump_slice(),
            method_annotations: &[],
        })
    }

    pub(crate) fn lower_enum(&mut self, r#enum: &'arena cst::Enum<'arena>) -> &'arena Enum<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&r#enum.attribute_lists);
        let name = self.lower_declaration_name(&r#enum.name);
        let backing_type = r#enum
            .backing_type_hint
            .as_ref()
            .map(|hint| EnumBackingType { span: hint.span(), r#type: self.lower_type(&hint.hint) });
        let implements = r#enum.implements.as_ref().map(|implements| self.lower_implements(implements));

        let mut trait_uses = Vec::new_in(self.arena);
        let mut constants = Vec::new_in(self.arena);
        let mut enum_cases = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);
        for member in r#enum.members.iter() {
            match member {
                cst::ClassLikeMember::TraitUse(trait_use) => trait_uses.push(self.lower_trait_use(trait_use)),
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::EnumCase(enum_case) => enum_cases.push(self.lower_enum_case(enum_case)),
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method)),
                _ => {}
            }
        }

        self.arena.alloc(Enum {
            flags: Flags::new(),
            attributes,
            name,
            backing_type,
            type_alias_annotations: &[],
            imported_type_alias_annotations: &[],
            implements,
            implements_annotations: &[],
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            enum_cases: enum_cases.into_bump_slice(),
            methods: methods.into_bump_slice(),
            method_annotations: &[],
        })
    }

    pub(crate) fn lower_function(
        &mut self,
        function: &'arena cst::Function<'arena>,
    ) -> &'arena Function<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&function.attribute_lists);
        let name = self.lower_declaration_name(&function.name);
        let parameters = self.lower_parameter_list(&function.parameter_list);
        let return_type = function.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));
        let body = self.statements_to_statement(function.body.statements.as_slice(), function.body.span());

        self.arena.alloc(Function {
            attributes,
            flags: Flags::new(),
            name,
            type_parameter_annotations: &[],
            parameters,
            where_constraint_annotations: &[],
            return_by_reference: function.ampersand.is_some(),
            return_type,
            return_type_annotation: None,
            throws: &[],
            asserts: &[],
            asserts_if_true: &[],
            asserts_if_false: &[],
            body,
        })
    }

    pub(crate) fn lower_constant(
        &mut self,
        constant: &'arena cst::Constant<'arena>,
    ) -> &'arena Constant<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&constant.attribute_lists);
        let items = self.arena.alloc_slice_fill_iter(constant.items.iter().map(|item| ConstantItem {
            name: self.lower_declaration_name(&item.name),
            value: self.arena.alloc(self.lower_expression(item.value)),
        }));

        self.arena.alloc(Constant { flags: Flags::new(), attributes, type_annotation: None, items })
    }
}
