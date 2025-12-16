use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Variable;

use crate::context::LintContext;
use crate::scope::ClassLikeScope;

pub fn is_within_controller(context: &LintContext<'_, '_>) -> bool {
    let Some(ClassLikeScope::Class(classname)) = context.scope.get_class_like_scope() else {
        return false;
    };

    classname.ends_with("Controller")
}

pub fn is_this(expression: &Expression<'_>) -> bool {
    if let Expression::Variable(Variable::Direct(var)) = expression {
        var.name.eq_ignore_ascii_case("$this")
    } else {
        false
    }
}

pub fn is_method_named(member: &ClassLikeMemberSelector<'_>, name: &str) -> bool {
    match member {
        ClassLikeMemberSelector::Identifier(method) => method.value.eq_ignore_ascii_case(name),
        _ => false,
    }
}
