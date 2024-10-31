use fennec_ast::ast::*;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_reflection::CodebaseReflection;
use fennec_walker::MutWalker;

use crate::internal::context::Context;
use crate::internal::reflect::class_like::*;
use crate::internal::reflect::constant::*;
use crate::internal::reflect::function_like::*;

#[derive(Debug)]
pub struct ReflectionWalker {
    pub reflection: CodebaseReflection,
    scope: Vec<ClassLikeReflection>,
}

impl ReflectionWalker {
    pub fn new() -> Self {
        Self { reflection: CodebaseReflection::new(), scope: Vec::new() }
    }
}

impl<'a> MutWalker<Context<'a>> for ReflectionWalker {
    fn walk_in_function<'ast>(&mut self, function: &'ast Function, context: &mut Context<'_>) {
        let reflection = reflect_function(function, context, self.scope.last());

        self.reflection.register_function_like(reflection);
    }

    fn walk_in_anonymous_class<'ast>(&mut self, anonymous_class: &'ast AnonymousClass, context: &mut Context<'_>) {
        self.scope.push(reflect_anonymous_class(anonymous_class, context));
    }

    fn walk_out_anonymous_class<'ast>(&mut self, _anonymous_class: &'ast AnonymousClass, _context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(reflection);
    }

    fn walk_in_class<'ast>(&mut self, class: &'ast Class, context: &mut Context<'_>) {
        self.scope.push(reflect_class(class, context));
    }

    fn walk_out_class<'ast>(&mut self, _class: &'ast Class, _context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(reflection);
    }

    fn walk_in_trait<'ast>(&mut self, r#trait: &'ast Trait, context: &mut Context<'_>) {
        self.scope.push(reflect_trait(r#trait, context));
    }

    fn walk_out_trait<'ast>(&mut self, _trait: &'ast Trait, _context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(reflection);
    }

    fn walk_in_enum<'ast>(&mut self, r#enum: &'ast Enum, context: &mut Context<'_>) {
        self.scope.push(reflect_enum(r#enum, context));
    }

    fn walk_out_enum<'ast>(&mut self, _enum: &'ast Enum, _context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(reflection);
    }

    fn walk_in_interface<'ast>(&mut self, interface: &'ast Interface, context: &mut Context<'_>) {
        self.scope.push(reflect_interface(interface, context));
    }

    fn walk_out_interface<'ast>(&mut self, _interface: &'ast Interface, _context: &mut Context<'a>) {
        let Some(reflection) = self.scope.pop() else {
            return;
        };

        self.reflection.register_class_like(reflection);
    }

    fn walk_in_closure<'ast>(&mut self, closure: &'ast Closure, context: &mut Context<'_>) {
        let reflection = reflect_closure(closure, context, self.scope.last());

        self.reflection.register_function_like(reflection);
    }

    fn walk_in_arrow_function<'ast>(&mut self, arrow_function: &'ast ArrowFunction, context: &mut Context<'_>) {
        let reflection = reflect_arrow_function(arrow_function, context, self.scope.last());

        self.reflection.register_function_like(reflection);
    }

    fn walk_constant<'ast>(&mut self, constant: &'ast Constant, context: &mut Context<'_>) {
        let reflections = reflect_constant(constant, context);

        for reflection in reflections {
            self.reflection.register_constant(reflection);
        }
    }
}
