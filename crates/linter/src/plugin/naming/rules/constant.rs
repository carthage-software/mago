use mago_ast::ast::*;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct ConstantRule;

impl Rule for ConstantRule {
    fn get_name(&self) -> &'static str {
        "constant"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Help)
    }
}

impl<'a> Walker<LintContext<'a>> for ConstantRule {
    fn walk_in_constant<'ast>(&self, constant: &'ast Constant, context: &mut LintContext<'a>) {
        for item in constant.items.iter() {
            let name = context.lookup(&item.name.value);
            if !mago_casing::is_constant_case(name) {
                context.report(
                    Issue::new(context.level(), format!("Constant name `{}` should be in constant case.", name))
                        .with_annotation(
                            Annotation::primary(item.name.span())
                                .with_message(format!("Constant item `{}` is declared here.", name)),
                        )
                        .with_note(format!("The constant name `{}` does not follow constant naming convention.", name))
                        .with_help(format!(
                            "Consider renaming it to `{}` to adhere to the naming convention.",
                            mago_casing::to_constant_case(name)
                        )),
                );
            }
        }
    }

    fn walk_in_class_like_constant<'ast>(
        &self,
        class_like_constant: &'ast ClassLikeConstant,
        context: &mut LintContext<'a>,
    ) {
        for item in class_like_constant.items.iter() {
            let name = context.lookup(&item.name.value);

            if !mago_casing::is_constant_case(name) {
                context.report(
                    Issue::new(context.level(), format!("Constant name `{}` should be in constant case.", name))
                        .with_annotation(
                            Annotation::primary(item.name.span())
                                .with_message(format!("Constant item `{}` is declared here.", name)),
                        )
                        .with_note(format!("The constant name `{}` does not follow constant naming convention.", name))
                        .with_help(format!(
                            "Consider renaming it to `{}` to adhere to the naming convention.",
                            mago_casing::to_constant_case(name)
                        )),
                );
            }
        }
    }
}
