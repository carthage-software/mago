use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_names::scope::NamespaceScope;
use mago_source::Source;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::comments::docblock::get_docblock_for_node;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_anonymous_class_mut;
use mago_syntax::walker::walk_class_mut;
use mago_syntax::walker::walk_enum_mut;
use mago_syntax::walker::walk_interface_mut;
use mago_syntax::walker::walk_trait_mut;

use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::scanner::class_like::*;
use crate::scanner::constant::*;
use crate::scanner::function_like::*;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;

mod argument;
mod attribute;
mod class_like;
mod class_like_constant;
mod constant;
mod docblock;
mod enum_case;
mod function_like;
mod inference;
mod parameter;
mod property;
mod ttype;

#[inline]
pub fn scan_program(
    interner: &ThreadedInterner,
    source: &Source,
    program: &Program,
    resolved_names: &ResolvedNames,
) -> CodebaseMetadata {
    let mut context = Context::new(interner, source, program, resolved_names);
    let mut scanner = Scanner::new();

    scanner.walk_program(program, &mut context);
    scanner.codebase
}

#[derive(Clone, Debug)]
struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub source: &'a Source,
    pub program: &'a Program,
    pub resolved_names: &'a ResolvedNames,
}

impl<'a> Context<'a> {
    pub fn new(
        interner: &'a ThreadedInterner,
        source: &'a Source,
        program: &'a Program,
        resolved_names: &'a ResolvedNames,
    ) -> Self {
        Self { interner, source, program, resolved_names }
    }

    pub fn get_docblock(&self, node: impl HasSpan) -> Option<&'a Trivia> {
        get_docblock_for_node(self.program, self.interner, self.source, node)
    }
}

type TemplateConstraint = (String, Vec<(GenericParent, TUnion)>);
type TemplateConstraintList = Vec<TemplateConstraint>;

#[derive(Debug, Default)]
struct Scanner {
    codebase: CodebaseMetadata,
    stack: Vec<StringIdentifier>,
    template_constraints: Vec<TemplateConstraintList>,
    scope: NamespaceScope,
}

impl Scanner {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_current_type_resolution_context(&self) -> TypeResolutionContext {
        let mut context = TypeResolutionContext::new();
        for template_constraint_list in self.template_constraints.iter().rev() {
            for (name, constraints) in template_constraint_list {
                if !context.has_template_definition(name) {
                    context = context.with_template_definition(name.clone(), constraints.clone());
                }
            }
        }

        context
    }
}

impl MutWalker<Context<'_>> for Scanner {
    #[inline]
    fn walk_in_namespace(&mut self, namespace: &Namespace, context: &mut Context<'_>) {
        self.scope = match &namespace.name {
            Some(name) => NamespaceScope::for_namespace(context.interner.lookup(name.value())),
            None => NamespaceScope::global(),
        };
    }

    #[inline]
    fn walk_out_namespace(&mut self, _namespace: &Namespace, _context: &mut Context<'_>) {
        self.scope = NamespaceScope::global();
    }

    #[inline]
    fn walk_in_use(&mut self, r#use: &Use, context: &mut Context<'_>) {
        self.scope.populate_from_use(context.interner, r#use);
    }

    #[inline]
    fn walk_in_function(&mut self, function: &Function, context: &mut Context<'_>) {
        let type_context = self.get_current_type_resolution_context();

        let name = context.interner.lowered(context.resolved_names.get(&function.name));
        let identifier = (StringIdentifier::empty(), name);
        let metadata = scan_function(identifier, function, self.stack.last(), context, &mut self.scope, type_context);

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in metadata.get_template_types() {
                constraints.push((context.interner.lookup(template_name).to_string(), template_constraints.to_vec()));
            }

            constraints
        });

        self.codebase.function_likes.insert(identifier, metadata);
    }

    #[inline]
    fn walk_out_function(&mut self, _function: &Function, _context: &mut Context<'_>) {
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_in_closure(&mut self, closure: &Closure, context: &mut Context<'_>) {
        let span = closure.span();

        let closure_ref = context.interner.intern(span.start.to_string());
        let identifier = (span.start.source.0, closure_ref);

        let type_resolution_context = self.get_current_type_resolution_context();
        let metadata =
            scan_closure(identifier, closure, self.stack.last(), context, &mut self.scope, type_resolution_context);

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in metadata.get_template_types() {
                constraints.push((context.interner.lookup(template_name).to_string(), template_constraints.to_vec()));
            }

            constraints
        });

        self.codebase.function_likes.insert(identifier, metadata);
        self.codebase.closure_files.entry(span.start.source).or_default().insert(closure_ref);
    }

    #[inline]
    fn walk_out_closure(&mut self, _closure: &Closure, _context: &mut Context<'_>) {
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_in_arrow_function(&mut self, arrow_function: &ArrowFunction, context: &mut Context<'_>) {
        let span = arrow_function.span();
        let closure_ref = context.interner.intern(span.start.to_string());
        let identifer = (span.start.source.0, closure_ref);

        let type_resolution_context = self.get_current_type_resolution_context();

        let metadata = scan_arrow_function(
            identifer,
            arrow_function,
            self.stack.last(),
            context,
            &mut self.scope,
            type_resolution_context,
        );

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in metadata.get_template_types() {
                constraints.push((context.interner.lookup(template_name).to_string(), template_constraints.to_vec()));
            }

            constraints
        });
        self.codebase.function_likes.insert(identifer, metadata);
        self.codebase.closure_files.entry(span.start.source).or_default().insert(closure_ref);
    }

    #[inline]
    fn walk_out_arrow_function(&mut self, _arrow_function: &ArrowFunction, _context: &mut Context<'_>) {
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_in_constant(&mut self, constant: &Constant, context: &mut Context<'_>) {
        for metadata in scan_constant(constant, context) {
            self.codebase.constants.insert(metadata.name, metadata);
        }
    }

    #[inline]
    fn walk_in_function_call(&mut self, function_call: &FunctionCall, context: &mut Context<'_>) {
        if let Some(metadata) = scan_defined_constant(function_call, context) {
            self.codebase.constants.insert(metadata.name, metadata);
        }
    }

    #[inline]
    fn walk_anonymous_class(&mut self, anonymous_class: &AnonymousClass, context: &mut Context<'_>) {
        if let Some((id, template_definition)) =
            register_anonymous_class(&mut self.codebase, anonymous_class, context, &mut self.scope)
        {
            self.stack.push(id);
            self.template_constraints.push(template_definition);

            walk_anonymous_class_mut(self, anonymous_class, context);
        } else {
            // We don't need to walk the anonymous class if it's already been registered
        }
    }

    #[inline]
    fn walk_class(&mut self, class: &Class, context: &mut Context<'_>) {
        if let Some((id, template_definition)) = register_class(&mut self.codebase, class, context, &mut self.scope) {
            self.stack.push(id);
            self.template_constraints.push(template_definition);

            walk_class_mut(self, class, context);
        } else {
            // We don't need to walk the class if it's already been registered
        }
    }

    #[inline]
    fn walk_trait(&mut self, r#trait: &Trait, context: &mut Context<'_>) {
        if let Some((id, template_definition)) = register_trait(&mut self.codebase, r#trait, context, &mut self.scope) {
            self.stack.push(id);
            self.template_constraints.push(template_definition);

            walk_trait_mut(self, r#trait, context);
        } else {
            // We don't need to walk the trait if it's already been registered
        }
    }

    #[inline]
    fn walk_enum(&mut self, r#enum: &Enum, context: &mut Context<'_>) {
        if let Some((id, template_definition)) = register_enum(&mut self.codebase, r#enum, context, &mut self.scope) {
            self.stack.push(id);
            self.template_constraints.push(template_definition);

            walk_enum_mut(self, r#enum, context);
        } else {
            // We don't need to walk the enum if it's already been registered
        }
    }

    #[inline]
    fn walk_interface(&mut self, interface: &Interface, context: &mut Context<'_>) {
        if let Some((id, template_definition)) =
            register_interface(&mut self.codebase, interface, context, &mut self.scope)
        {
            self.stack.push(id);
            self.template_constraints.push(template_definition);

            walk_interface_mut(self, interface, context);
        }
    }

    #[inline]
    fn walk_out_anonymous_class(&mut self, _anonymous_class: &AnonymousClass, _context: &mut Context<'_>) {
        self.stack.pop().expect("Expected class stack to be non-empty");
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_out_class(&mut self, _class: &Class, _context: &mut Context<'_>) {
        self.stack.pop().expect("Expected class stack to be non-empty");
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_out_trait(&mut self, _trait: &Trait, _context: &mut Context<'_>) {
        self.stack.pop().expect("Expected class stack to be non-empty");
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_out_enum(&mut self, _enum: &Enum, _context: &mut Context<'_>) {
        self.stack.pop().expect("Expected class stack to be non-empty");
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_out_interface(&mut self, _interface: &Interface, _context: &mut Context<'_>) {
        self.stack.pop().expect("Expected class stack to be non-empty");
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }
}
