use mago_reflection::CodebaseReflection;
use mago_reflection::class_like::ClassLikeReflection;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::internal::context::Context;
use crate::internal::reflector::class_like::*;
use crate::internal::reflector::constant::*;
use crate::internal::reflector::function_like::*;

#[derive(Debug)]
pub struct ModuleReflectionWalker {
    pub reflection: CodebaseReflection,
    scope: Vec<ClassLikeReflection>,
}

impl ModuleReflectionWalker {
    pub fn new() -> Self {
        Self { reflection: CodebaseReflection::new(), scope: Vec::new() }
    }
}

impl<'a> MutWalker<Context<'a>> for ModuleReflectionWalker {
    #[inline]
    fn walk_in_function(&mut self, function: &Function, context: &mut Context<'_>) {
        if let Some(reflection) = reflect_function(function, context, self.scope.last()) {
            self.reflection.register_function_like(context.interner, reflection);
        }
    }

    #[inline]
    fn walk_in_anonymous_class(&mut self, anonymous_class: &AnonymousClass, context: &mut Context<'_>) {
        self.scope.push(reflect_anonymous_class(anonymous_class, context));
    }

    #[inline]
    fn walk_out_anonymous_class<'ast>(&mut self, _anonymous_class: &'ast AnonymousClass, context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(context.interner, reflection);
    }

    #[inline]
    fn walk_in_class(&mut self, class: &Class, context: &mut Context<'_>) {
        self.scope.push(reflect_class(class, context));
    }

    #[inline]
    fn walk_out_class<'ast>(&mut self, _class: &'ast Class, context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(context.interner, reflection);
    }

    #[inline]
    fn walk_in_trait(&mut self, r#trait: &Trait, context: &mut Context<'_>) {
        self.scope.push(reflect_trait(r#trait, context));
    }

    #[inline]
    fn walk_out_trait<'ast>(&mut self, _trait: &'ast Trait, context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(context.interner, reflection);
    }

    #[inline]
    fn walk_in_enum(&mut self, r#enum: &Enum, context: &mut Context<'_>) {
        self.scope.push(reflect_enum(r#enum, context));
    }

    #[inline]
    fn walk_out_enum<'ast>(&mut self, _enum: &'ast Enum, context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(context.interner, reflection);
    }

    #[inline]
    fn walk_in_interface(&mut self, interface: &Interface, context: &mut Context<'_>) {
        self.scope.push(reflect_interface(interface, context));
    }

    #[inline]
    fn walk_out_interface<'ast>(&mut self, _interface: &'ast Interface, context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(context.interner, reflection);
    }

    #[inline]
    fn walk_in_closure(&mut self, closure: &Closure, context: &mut Context<'_>) {
        let reflection = reflect_closure(closure, context, self.scope.last());

        self.reflection.register_function_like(context.interner, reflection);
    }

    #[inline]
    fn walk_in_arrow_function(&mut self, arrow_function: &ArrowFunction, context: &mut Context<'_>) {
        let reflection = reflect_arrow_function(arrow_function, context, self.scope.last());

        self.reflection.register_function_like(context.interner, reflection);
    }

    #[inline]
    fn walk_in_constant(&mut self, constant: &Constant, context: &mut Context<'_>) {
        let reflections = reflect_constant(constant, context);

        for reflection in reflections {
            self.reflection.register_constant(context.interner, reflection);
        }
    }

    #[inline]
    fn walk_in_function_call(&mut self, function_call: &FunctionCall, context: &mut Context<'a>) {
        if let Some(constant_reflection) = reflect_defined_constant(function_call, context) {
            self.reflection.register_constant(context.interner, constant_reflection);
        }
    }
}
