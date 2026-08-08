use mago_syntax::cst::Class;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::Constant;
use mago_syntax::cst::Enum;
use mago_syntax::cst::Function;
use mago_syntax::cst::Interface;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Namespace;
use mago_syntax::cst::Trait;
use mago_syntax::walker::MutWalker;

use crate::context::GuardContext;
use crate::matcher;
use crate::report::flaw::FlawKind;
use crate::report::flaw::StructuralFlaw;
use crate::settings::StructuralInheritanceConstraint;
use crate::settings::StructuralRule;
use crate::settings::StructuralSymbolKind;

#[derive(Debug, Clone, Copy)]
pub struct StructuralGuardWalker;

impl StructuralGuardWalker {
    fn get_structural_rules<'ctx, 'arena>(
        context: &'ctx GuardContext<'ctx, 'arena>,
        fqn: &'arena [u8],
        kind: StructuralSymbolKind,
    ) -> Vec<&'ctx StructuralRule> {
        context.settings.structural.rules.iter().filter(|rule| Self::applies_to(rule, fqn, kind)).collect()
    }

    fn applies_to(rule: &StructuralRule, fqn: &[u8], kind: StructuralSymbolKind) -> bool {
        if let Some(rule_kind) = rule.target
            && rule_kind != kind
        {
            return false;
        }

        if !matcher::matches(fqn, rule.on.as_bytes(), kind.is_constant(), true) {
            return false;
        }

        if let Some(not_on) = &rule.not_on
            && matcher::matches(fqn, not_on.as_bytes(), kind.is_constant(), true)
        {
            return false;
        }

        true
    }

    fn satisfies_inheritance_constraint(
        implemented_fqns: &[&[u8]],
        constraint: &StructuralInheritanceConstraint,
    ) -> bool {
        match constraint {
            StructuralInheritanceConstraint::AnyOfAllOf(groups) => {
                for items in groups {
                    if items
                        .iter()
                        .all(|item| implemented_fqns.iter().any(|imp| imp.eq_ignore_ascii_case(item.as_bytes())))
                    {
                        return true;
                    }
                }

                false
            }
            StructuralInheritanceConstraint::AllOf(items) => {
                for item in items {
                    if !implemented_fqns.iter().any(|imp| imp.eq_ignore_ascii_case(item.as_bytes())) {
                        return false;
                    }
                }

                true
            }
            StructuralInheritanceConstraint::Single(item) => {
                implemented_fqns.iter().any(|imp| imp.eq_ignore_ascii_case(item.as_bytes()))
            }
            StructuralInheritanceConstraint::Nothing => implemented_fqns.is_empty(),
        }
    }
}

fn fqn_to_owned(fqn: &[u8]) -> Vec<u8> {
    fqn.to_vec()
}

impl<'ast, 'ctx, 'arena> MutWalker<'ast, 'arena, GuardContext<'ctx, 'arena>> for StructuralGuardWalker {
    fn walk_in_namespace(&mut self, namespace: &'ast Namespace<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        context.set_current_namespace(namespace.name.as_ref().map(mago_syntax::cst::Identifier::value));
    }

    fn walk_out_namespace(&mut self, _namespace: &'ast Namespace<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        context.set_current_namespace(None);
    }

    fn walk_in_class(&mut self, class: &'ast Class<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        let fqn = context.lookup_name(&class.name);
        let structural_rules = Self::get_structural_rules(context, fqn, StructuralSymbolKind::Class);

        let mut structural_flaws = vec![];

        for structural_rule in structural_rules {
            if let Some(must_be_named) = &structural_rule.must_be_named
                && !matcher::matches(class.name.value, must_be_named.as_bytes(), false, false)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Class,
                    span: class.name.span,
                    kind: FlawKind::MustBeNamed { pattern: must_be_named.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(allowed_kinds) = &structural_rule.must_be
                && !allowed_kinds.contains(&StructuralSymbolKind::Class)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Class,
                    span: class.name.span,
                    kind: FlawKind::MustBe { allowed: allowed_kinds.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(must_be_final) = structural_rule.must_be_final {
                let is_final = context
                    .codebase
                    .get_class(fqn)
                    .map(|c| c.flags.is_final())
                    .unwrap_or(class.modifiers.contains_final());
                let is_abstract = class.modifiers.contains_abstract();

                match (must_be_final, is_final, is_abstract) {
                    (true, false, false) => {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Class,
                            span: class.name.span,
                            kind: FlawKind::MustBeFinal,
                            reason: structural_rule.reason.clone(),
                        });
                    }
                    (false, true, true) => {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Class,
                            span: class.name.span,
                            kind: FlawKind::MustNotBeFinal,
                            reason: structural_rule.reason.clone(),
                        });
                    }
                    _ => {}
                }
            }

            if let Some(must_be_abstract) = structural_rule.must_be_abstract {
                let is_abstract = class.modifiers.contains_abstract();

                match (must_be_abstract, is_abstract) {
                    (true, false) => {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Class,
                            span: class.name.span,
                            kind: FlawKind::MustBeAbstract,
                            reason: structural_rule.reason.clone(),
                        });
                    }
                    (false, true) => {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Class,
                            span: class.name.span,
                            kind: FlawKind::MustNotBeAbstract,
                            reason: structural_rule.reason.clone(),
                        });
                    }
                    _ => {}
                }
            }

            if let Some(must_be_readonly) = structural_rule.must_be_readonly {
                let is_readonly = class.modifiers.contains_readonly();

                match (must_be_readonly, is_readonly) {
                    (true, false) => {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Class,
                            span: class.name.span,
                            kind: FlawKind::MustBeReadonly,
                            reason: structural_rule.reason.clone(),
                        });
                    }
                    (false, true) => {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Class,
                            span: class.name.span,
                            kind: FlawKind::MustNotBeReadonly,
                            reason: structural_rule.reason.clone(),
                        });
                    }
                    _ => {}
                }
            }

            if let Some(must_extends) = &structural_rule.must_extend {
                let extended_fqns: Vec<&[u8]> = class
                    .extends
                    .as_ref()
                    .iter()
                    .flat_map(|ext| ext.types.iter())
                    .map(|ident| context.lookup_name(ident))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&extended_fqns, must_extends) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Class,
                        span: class.name.span,
                        kind: FlawKind::MustExtend { expected: must_extends.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }

            if let Some(must_implement) = &structural_rule.must_implement {
                let implemented_fqns: Vec<&[u8]> = class
                    .implements
                    .as_ref()
                    .iter()
                    .flat_map(|ext| ext.types.iter())
                    .map(|ident| context.lookup_name(ident))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&implemented_fqns, must_implement) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Class,
                        span: class.name.span,
                        kind: FlawKind::MustImplement { expected: must_implement.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }

            if let Some(must_use_traits) = &structural_rule.must_use_trait {
                let used_fqns: Vec<&[u8]> = class
                    .members
                    .iter()
                    .filter_map(|member| match member {
                        ClassLikeMember::TraitUse(trait_use) => Some(trait_use),
                        _ => None,
                    })
                    .flat_map(|trait_use| trait_use.trait_names.iter())
                    .map(|ident| context.lookup_name(ident))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&used_fqns, must_use_traits) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Class,
                        span: class.name.span,
                        kind: FlawKind::MustUseTrait { expected: must_use_traits.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }

            if let Some(must_use_attributes) = &structural_rule.must_use_attribute {
                let used_attributes: Vec<&[u8]> = class
                    .attribute_lists
                    .iter()
                    .flat_map(|attribute_list| attribute_list.attributes.iter())
                    .map(|attr| context.lookup_name(&attr.name))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&used_attributes, must_use_attributes) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Class,
                        span: class.name.span,
                        kind: FlawKind::MustUseAttribute { expected: must_use_attributes.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }

            if let Some(allowed_public_methods) = &structural_rule.only_public_methods {
                for method in class.members.iter().filter_map(|member| match member {
                    ClassLikeMember::Method(method) => Some(method),
                    _ => None,
                }) {
                    let is_public = !method.modifiers.contains_protected() && !method.modifiers.contains_private();
                    let is_allowed = allowed_public_methods
                        .iter()
                        .any(|allowed| method.name.value.eq_ignore_ascii_case(allowed.as_bytes()));

                    if is_public && !is_allowed {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Class,
                            span: method.name.span,
                            kind: FlawKind::PublicMethodNotAllowed {
                                method: method.name.value.to_vec(),
                                allowed: allowed_public_methods.clone(),
                            },
                            reason: structural_rule.reason.clone(),
                        });
                    }
                }
            }
        }

        context.structural_flaws.extend(structural_flaws);
    }

    fn walk_in_interface(&mut self, interface: &'ast Interface<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        let fqn = context.lookup_name(&interface.name);
        let structural_rules = Self::get_structural_rules(context, fqn, StructuralSymbolKind::Interface);

        let mut structural_flaws = vec![];
        for structural_rule in structural_rules {
            if let Some(must_be_named) = &structural_rule.must_be_named
                && !matcher::matches(interface.name.value, must_be_named.as_bytes(), false, false)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Interface,
                    span: interface.name.span,
                    kind: FlawKind::MustBeNamed { pattern: must_be_named.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(allowed_kinds) = &structural_rule.must_be
                && !allowed_kinds.contains(&StructuralSymbolKind::Interface)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Interface,
                    span: interface.name.span,
                    kind: FlawKind::MustBe { allowed: allowed_kinds.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(must_extends) = &structural_rule.must_extend {
                let extended_fqns: Vec<&[u8]> = interface
                    .extends
                    .as_ref()
                    .iter()
                    .flat_map(|ext| ext.types.iter())
                    .map(|ident| context.lookup_name(ident))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&extended_fqns, must_extends) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Interface,
                        span: interface.name.span,
                        kind: FlawKind::MustExtend { expected: must_extends.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }

            if let Some(must_use_attributes) = &structural_rule.must_use_attribute {
                let used_attributes: Vec<&[u8]> = interface
                    .attribute_lists
                    .iter()
                    .flat_map(|attribute_list| attribute_list.attributes.iter())
                    .map(|attr| context.lookup_name(&attr.name))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&used_attributes, must_use_attributes) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Interface,
                        span: interface.name.span,
                        kind: FlawKind::MustUseAttribute { expected: must_use_attributes.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }
        }

        context.structural_flaws.extend(structural_flaws);
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        let fqn = context.lookup_name(&r#enum.name);
        let structural_rules = Self::get_structural_rules(context, fqn, StructuralSymbolKind::Enum);

        let mut structural_flaws = vec![];
        for structural_rule in structural_rules {
            if let Some(must_be_named) = &structural_rule.must_be_named
                && !matcher::matches(r#enum.name.value, must_be_named.as_bytes(), false, false)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Enum,
                    span: r#enum.name.span,
                    kind: FlawKind::MustBeNamed { pattern: must_be_named.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(allowed_kinds) = &structural_rule.must_be
                && !allowed_kinds.contains(&StructuralSymbolKind::Enum)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Enum,
                    span: r#enum.name.span,
                    kind: FlawKind::MustBe { allowed: allowed_kinds.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(must_implement) = &structural_rule.must_implement {
                let implemented_fqns: Vec<&[u8]> = r#enum
                    .implements
                    .as_ref()
                    .iter()
                    .flat_map(|ext| ext.types.iter())
                    .map(|ident| context.lookup_name(ident))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&implemented_fqns, must_implement) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Enum,
                        span: r#enum.name.span,
                        kind: FlawKind::MustImplement { expected: must_implement.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }

            if let Some(must_use_attributes) = &structural_rule.must_use_attribute {
                let used_attributes: Vec<&[u8]> = r#enum
                    .attribute_lists
                    .iter()
                    .flat_map(|attribute_list| attribute_list.attributes.iter())
                    .map(|attr| context.lookup_name(&attr.name))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&used_attributes, must_use_attributes) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Enum,
                        span: r#enum.name.span,
                        kind: FlawKind::MustUseAttribute { expected: must_use_attributes.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }
        }

        context.structural_flaws.extend(structural_flaws);
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        let fqn = context.lookup_name(&r#trait.name);
        let structural_rules = Self::get_structural_rules(context, fqn, StructuralSymbolKind::Trait);

        let mut structural_flaws = vec![];
        for structural_rule in structural_rules {
            if let Some(must_be_named) = &structural_rule.must_be_named
                && !matcher::matches(r#trait.name.value, must_be_named.as_bytes(), false, false)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Trait,
                    span: r#trait.name.span,
                    kind: FlawKind::MustBeNamed { pattern: must_be_named.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(allowed_kinds) = &structural_rule.must_be
                && !allowed_kinds.contains(&StructuralSymbolKind::Trait)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Trait,
                    span: r#trait.name.span,
                    kind: FlawKind::MustBe { allowed: allowed_kinds.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(must_use_traits) = &structural_rule.must_use_trait {
                let used_fqns: Vec<&[u8]> = r#trait
                    .members
                    .iter()
                    .filter_map(|member| match member {
                        ClassLikeMember::TraitUse(trait_use) => Some(trait_use),
                        _ => None,
                    })
                    .flat_map(|trait_use| trait_use.trait_names.iter())
                    .map(|ident| context.lookup_name(ident))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&used_fqns, must_use_traits) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Trait,
                        span: r#trait.name.span,
                        kind: FlawKind::MustUseTrait { expected: must_use_traits.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }

            if let Some(must_use_attributes) = &structural_rule.must_use_attribute {
                let used_attributes: Vec<&[u8]> = r#trait
                    .attribute_lists
                    .iter()
                    .flat_map(|attribute_list| attribute_list.attributes.iter())
                    .map(|attr| context.lookup_name(&attr.name))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&used_attributes, must_use_attributes) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Trait,
                        span: r#trait.name.span,
                        kind: FlawKind::MustUseAttribute { expected: must_use_attributes.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }
        }

        context.structural_flaws.extend(structural_flaws);
    }

    fn walk_in_function(&mut self, function: &'ast Function<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        let fqn = context.lookup_name(&function.name);
        let structural_rules = Self::get_structural_rules(context, fqn, StructuralSymbolKind::Function);

        let mut structural_flaws = vec![];
        for structural_rule in structural_rules {
            if let Some(must_be_named) = &structural_rule.must_be_named
                && !matcher::matches(function.name.value, must_be_named.as_bytes(), false, false)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Function,
                    span: function.name.span,
                    kind: FlawKind::MustBeNamed { pattern: must_be_named.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(allowed_kinds) = &structural_rule.must_be
                && !allowed_kinds.contains(&StructuralSymbolKind::Function)
            {
                structural_flaws.push(StructuralFlaw {
                    symbol_fqn: fqn_to_owned(fqn),
                    symbol_kind: StructuralSymbolKind::Function,
                    span: function.name.span,
                    kind: FlawKind::MustBe { allowed: allowed_kinds.clone() },
                    reason: structural_rule.reason.clone(),
                });
            }

            if let Some(must_use_attribute) = &structural_rule.must_use_attribute {
                let used_attributes: Vec<&[u8]> = function
                    .attribute_lists
                    .iter()
                    .flat_map(|attribute_list| attribute_list.attributes.iter())
                    .map(|attr| context.lookup_name(&attr.name))
                    .collect();

                if !Self::satisfies_inheritance_constraint(&used_attributes, must_use_attribute) {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Function,
                        span: function.name.span,
                        kind: FlawKind::MustUseAttribute { expected: must_use_attribute.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }
            }
        }

        context.structural_flaws.extend(structural_flaws);
    }

    fn walk_in_constant(&mut self, constant: &'ast Constant<'arena>, context: &mut GuardContext<'ctx, 'arena>) {
        let mut structural_flaws = vec![];
        for constant_item in &constant.items {
            let fqn = context.lookup_name(&constant_item.name);
            let structural_rules = Self::get_structural_rules(context, fqn, StructuralSymbolKind::Constant);

            for structural_rule in structural_rules {
                if let Some(must_be_named) = &structural_rule.must_be_named
                    && !matcher::matches(constant_item.name.value, must_be_named.as_bytes(), true, false)
                {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Constant,
                        span: constant_item.name.span,
                        kind: FlawKind::MustBeNamed { pattern: must_be_named.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }

                if let Some(allowed_kinds) = &structural_rule.must_be
                    && !allowed_kinds.contains(&StructuralSymbolKind::Constant)
                {
                    structural_flaws.push(StructuralFlaw {
                        symbol_fqn: fqn_to_owned(fqn),
                        symbol_kind: StructuralSymbolKind::Constant,
                        span: constant_item.name.span,
                        kind: FlawKind::MustBe { allowed: allowed_kinds.clone() },
                        reason: structural_rule.reason.clone(),
                    });
                }

                if let Some(must_use_attributes) = &structural_rule.must_use_attribute {
                    let used_attributes: Vec<&[u8]> = constant
                        .attribute_lists
                        .iter()
                        .flat_map(|attribute_list| attribute_list.attributes.iter())
                        .map(|attr| context.lookup_name(&attr.name))
                        .collect();

                    if !Self::satisfies_inheritance_constraint(&used_attributes, must_use_attributes) {
                        structural_flaws.push(StructuralFlaw {
                            symbol_fqn: fqn_to_owned(fqn),
                            symbol_kind: StructuralSymbolKind::Constant,
                            span: constant_item.name.span,
                            kind: FlawKind::MustUseAttribute { expected: must_use_attributes.clone() },
                            reason: structural_rule.reason.clone(),
                        });
                    }
                }
            }
        }

        context.structural_flaws.extend(structural_flaws);
    }
}
