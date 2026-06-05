use mago_syntax::cst;

use crate::ir::attribute::AttributeTarget;
use crate::ir::flags::Flags;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_attribute_target(
        &self,
        class_name: &[u8],
        attribute_lists: &'arena cst::Sequence<'arena, cst::AttributeList<'arena>>,
    ) -> Option<Flags<AttributeTarget>> {
        if is_attribute_name(class_name) {
            let mut flags = Flags::new();
            flags.set(AttributeTarget::Class);
            return Some(flags);
        }

        for attribute in attribute_lists.iter().flat_map(|list| list.attributes.iter()) {
            let resolved = self.namespace_resolution.resolve_name(NameResolutionKind::Default, attribute.name.value());
            if !is_attribute_name(resolved) {
                continue;
            }

            let Some(first_argument) =
                attribute.argument_list.as_ref().and_then(|argument_list| argument_list.arguments.iter().next())
            else {
                return Some(all_targets());
            };

            return Some(self.fold_attribute_target(first_argument.value()).unwrap_or_else(all_targets_repeatable));
        }

        None
    }

    fn fold_attribute_target(&self, expression: &cst::Expression<'_>) -> Option<Flags<AttributeTarget>> {
        match expression.unparenthesized() {
            cst::Expression::Access(cst::Access::ClassConstant(access)) => self.attribute_constant_target(access),
            cst::Expression::Binary(cst::Binary { operator: cst::BinaryOperator::BitwiseOr(_), lhs, rhs }) => {
                Some(self.fold_attribute_target(lhs)? | self.fold_attribute_target(rhs)?)
            }
            _ => None,
        }
    }

    fn attribute_constant_target(&self, access: &cst::ClassConstantAccess<'_>) -> Option<Flags<AttributeTarget>> {
        let cst::ClassLikeConstantSelector::Identifier(constant) = &access.constant else {
            return None;
        };

        let cst::Expression::Identifier(class) = access.class else {
            return None;
        };

        let class_name = self.namespace_resolution.resolve_name(NameResolutionKind::Default, class.value());
        if !is_attribute_name(class_name) {
            return None;
        }

        let mut flags = Flags::new();
        match constant.value {
            b"TARGET_CLASS" => flags.set(AttributeTarget::Class),
            b"TARGET_FUNCTION" => flags.set(AttributeTarget::Function),
            b"TARGET_METHOD" => flags.set(AttributeTarget::Method),
            b"TARGET_PROPERTY" => flags.set(AttributeTarget::Property),
            b"TARGET_CLASS_CONSTANT" => flags.set(AttributeTarget::ClassConstant),
            b"TARGET_PARAMETER" => flags.set(AttributeTarget::Parameter),
            b"TARGET_CONSTANT" => flags.set(AttributeTarget::Constant),
            b"TARGET_ALL" => return Some(all_targets()),
            b"IS_REPEATABLE" => flags.set(AttributeTarget::Repeatable),
            _ => return None,
        }

        Some(flags)
    }
}

fn is_attribute_name(resolved: &[u8]) -> bool {
    let mut bytes = resolved;
    while let Some(rest) = bytes.strip_prefix(b"\\") {
        bytes = rest;
    }

    bytes.eq_ignore_ascii_case(b"Attribute")
}

fn all_targets() -> Flags<AttributeTarget> {
    let mut flags = Flags::new();
    flags.set(AttributeTarget::Class);
    flags.set(AttributeTarget::Function);
    flags.set(AttributeTarget::Method);
    flags.set(AttributeTarget::Property);
    flags.set(AttributeTarget::ClassConstant);
    flags.set(AttributeTarget::Parameter);
    flags.set(AttributeTarget::Constant);
    flags
}

fn all_targets_repeatable() -> Flags<AttributeTarget> {
    let mut flags = all_targets();
    flags.set(AttributeTarget::Repeatable);
    flags
}
