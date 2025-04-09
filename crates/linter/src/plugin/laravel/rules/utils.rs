use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::scope::ClassLikeScope;

pub fn is_within_controller(context: &LintContext<'_>) -> bool {
    let Some(ClassLikeScope::Class(classname_id)) = context.scope.get_class_like_scope() else {
        return false;
    };

    context.interner.lookup(classname_id).ends_with("Controller")
}

pub fn is_this<'a>(context: &LintContext<'a>, expression: &'a Expression) -> bool {
    if let Expression::Variable(Variable::Direct(var)) = expression {
        context.interner.lookup(&var.name).eq_ignore_ascii_case("$this")
    } else {
        false
    }
}

pub fn is_method_named<'a>(context: &LintContext<'a>, member: &'a ClassLikeMemberSelector, name: &str) -> bool {
    match member {
        ClassLikeMemberSelector::Identifier(method) => {
            context.interner.lookup(&method.value).eq_ignore_ascii_case(name)
        }
        _ => false,
    }
}

pub fn is_function_call_to<'a>(context: &LintContext<'a>, expression: &'a Expression, function_name: &str) -> bool {
    let Expression::Call(Call::Function(FunctionCall { function, .. })) = expression else {
        return false;
    };

    let Expression::Identifier(function_identifier) = function.as_ref() else { return false };

    let called_function_name = context.resolve_function_name(function_identifier);

    called_function_name.eq_ignore_ascii_case(function_name)
}
