use fennec_ast::ast::*;
use fennec_reflection::CodebaseReflection;
use fennec_walker::MutWalker;

use crate::internal::context::Context;
use crate::internal::reflect::class_like::*;
use crate::internal::reflect::constant::*;
use crate::internal::reflect::function_like::*;

#[derive(Debug)]
pub struct ReflectionWalker {
    pub reflections: CodebaseReflection,
}

impl ReflectionWalker {
    pub fn new() -> Self {
        Self { reflections: CodebaseReflection::new() }
    }
}

impl<'a> MutWalker<Context<'a>> for ReflectionWalker {
    fn walk_in_function<'ast>(&mut self, function: &'ast Function, context: &mut Context<'_>) {
        let reflection = reflect_function(function, context);

        self.reflections.register_function_like(reflection);
    }

    fn walk_in_anonymous_class<'ast>(&mut self, anonymous_class: &'ast AnonymousClass, context: &mut Context<'_>) {
        let reflection = reflect_anonymous_class(anonymous_class, context);

        context.enter_scope(reflection);
    }

    fn walk_out_anonymous_class<'ast>(&mut self, _anonymous_class: &'ast AnonymousClass, context: &mut Context<'_>) {
        let Some(reflection) = context.exit_scope() else {
            panic!("scope should be present when exiting anonymous class, this is a bug in fennec, please report it.");
        };

        self.reflections.register_class_like(reflection);
    }

    fn walk_in_class<'ast>(&mut self, class: &'ast Class, context: &mut Context<'_>) {
        let reflection = reflect_class(class, context);

        context.enter_scope(reflection);
    }

    fn walk_out_class<'ast>(&mut self, _class: &'ast Class, context: &mut Context<'_>) {
        let Some(reflection) = context.exit_scope() else {
            panic!("scope should be present when exiting class, this is a bug in fennec, please report it.");
        };

        self.reflections.register_class_like(reflection);
    }

    fn walk_in_trait<'ast>(&mut self, r#trait: &'ast Trait, context: &mut Context<'_>) {
        let symbol = reflect_trait(r#trait, context);

        context.enter_scope(symbol);
    }

    fn walk_out_trait<'ast>(&mut self, _trait: &'ast Trait, context: &mut Context<'_>) {
        let Some(reflection) = context.exit_scope() else {
            panic!("scope should be present when exiting class, this is a bug in fennec, please report it.");
        };

        self.reflections.register_class_like(reflection);
    }

    fn walk_in_enum<'ast>(&mut self, r#enum: &'ast Enum, context: &mut Context<'_>) {
        let symbol = reflect_enum(r#enum, context);

        context.enter_scope(symbol);
    }

    fn walk_out_enum<'ast>(&mut self, _enum: &'ast Enum, context: &mut Context<'_>) {
        let Some(reflection) = context.exit_scope() else {
            panic!("scope should be present when exiting class, this is a bug in fennec, please report it.");
        };

        self.reflections.register_class_like(reflection);
    }

    fn walk_in_interface<'ast>(&mut self, interface: &'ast Interface, context: &mut Context<'_>) {
        let symbol = reflect_interface(interface, context);

        context.enter_scope(symbol);
    }

    fn walk_out_interface<'ast>(&mut self, _interface: &'ast Interface, context: &mut Context<'_>) {
        let Some(reflection) = context.exit_scope() else {
            panic!("scope should be present when exiting class, this is a bug in fennec, please report it.");
        };

        self.reflections.register_class_like(reflection);
    }

    fn walk_in_closure<'ast>(&mut self, closure: &'ast Closure, context: &mut Context<'_>) {
        let reflection = reflect_closure(closure, context);

        self.reflections.register_function_like(reflection);
    }

    fn walk_in_arrow_function<'ast>(&mut self, arrow_function: &'ast ArrowFunction, context: &mut Context<'_>) {
        let reflection = reflect_arrow_function(arrow_function, context);

        self.reflections.register_function_like(reflection);
    }

    fn walk_constant<'ast>(&mut self, constant: &'ast Constant, context: &mut Context<'_>) {
        let reflections = reflect_constant(constant, context);

        for reflection in reflections {
            self.reflections.register_constant(reflection);
        }
    }
}
