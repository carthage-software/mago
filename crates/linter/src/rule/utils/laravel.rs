use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::scope::ClassLikeScope;

pub fn is_within_controller<'ctx, 'arena>(context: &LintContext<'ctx, 'arena>) -> bool {
    let Some(ClassLikeScope::Class(classname)) = context.scope.get_class_like_scope() else {
        return false;
    };

    classname.ends_with("Controller")
}

pub fn is_this<'ast, 'arena>(expression: &'ast Expression<'arena>) -> bool {
    if let Expression::Variable(Variable::Direct(var)) = expression {
        var.name.eq_ignore_ascii_case("$this")
    } else {
        false
    }
}

pub fn is_method_named<'ast, 'arena>(member: &'ast ClassLikeMemberSelector<'arena>, name: &str) -> bool {
    match member {
        ClassLikeMemberSelector::Identifier(method) => method.value.eq_ignore_ascii_case(name),
        _ => false,
    }
}
