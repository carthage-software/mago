use mago_ast::*;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct UndefinedConstantRule;

impl Rule for UndefinedConstantRule {
    fn get_name(&self) -> &'static str {
        "undefined-constant"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Error)
    }
}

impl<'a> Walker<LintContext<'a>> for UndefinedConstantRule {
    fn walk_in_constant_access(&self, constant_access: &ConstantAccess, context: &mut LintContext<'a>) {
        let identifier = &constant_access.name;
        let constant_name = context.resolve_constant_name(identifier);
        let constant_name_id = context.interner.intern(constant_name);
        if context.codebase.constant_exists(&constant_name_id) {
            return;
        }

        let issue = Issue::error(format!("Use of undefined constant `{}`.", constant_name))
            .with_annotation(
                Annotation::primary(identifier.span())
                    .with_message(format!("Constant `{}` does not exist.", constant_name)),
            )
            .with_help(format!("Ensure the constant `{}` is defined or imported before using it.", constant_name));

        context.report(issue);
    }
}
