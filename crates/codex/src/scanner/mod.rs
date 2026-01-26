use bumpalo::Bump;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_atom::empty_atom;
use mago_atom::u32_atom;
use mago_atom::u64_atom;
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::ast::AnonymousClass;
use mago_syntax::ast::ArrowFunction;
use mago_syntax::ast::Class;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Constant;
use mago_syntax::ast::Enum;
use mago_syntax::ast::Function;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Interface;
use mago_syntax::ast::Method;
use mago_syntax::ast::Namespace;
use mago_syntax::ast::Program;
use mago_syntax::ast::Trait;
use mago_syntax::ast::Trivia;
use mago_syntax::ast::Use;
use mago_syntax::comments::docblock::get_docblock_for_node;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_anonymous_class_mut;
use mago_syntax::walker::walk_class_mut;
use mago_syntax::walker::walk_enum_mut;
use mago_syntax::walker::walk_interface_mut;
use mago_syntax::walker::walk_trait_mut;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeKind;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::misc::GenericParent;
use crate::scanner::class_like::register_anonymous_class;
use crate::scanner::class_like::register_class;
use crate::scanner::class_like::register_enum;
use crate::scanner::class_like::register_interface;
use crate::scanner::class_like::register_trait;
use crate::scanner::constant::scan_constant;
use crate::scanner::constant::scan_defined_constant;
use crate::scanner::function_like::scan_arrow_function;
use crate::scanner::function_like::scan_closure;
use crate::scanner::function_like::scan_function;
use crate::scanner::function_like::scan_method;
use crate::scanner::property::scan_promoted_property;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;

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
pub fn scan_program<'arena, 'ctx>(
    arena: &'arena Bump,
    file: &'ctx File,
    program: &'arena Program<'arena>,
    resolved_names: &'ctx ResolvedNames<'arena>,
) -> CodebaseMetadata {
    let mut context = Context::new(arena, file, program, resolved_names);
    let mut scanner = Scanner::new();

    scanner.walk_program(program, &mut context);

    scanner.codebase
}

#[derive(Clone, Debug)]
struct Context<'ctx, 'arena> {
    pub arena: &'arena Bump,
    pub file: &'ctx File,
    pub program: &'arena Program<'arena>,
    pub resolved_names: &'arena ResolvedNames<'arena>,
}

impl<'ctx, 'arena> Context<'ctx, 'arena> {
    pub fn new(
        arena: &'arena Bump,
        file: &'ctx File,
        program: &'arena Program<'arena>,
        resolved_names: &'arena ResolvedNames<'arena>,
    ) -> Self {
        Self { arena, file, program, resolved_names }
    }

    pub fn get_docblock(&self, node: impl HasSpan) -> Option<&'arena Trivia<'arena>> {
        get_docblock_for_node(self.program, self.file, node)
    }
}

type TemplateConstraint = (Atom, Vec<(GenericParent, TUnion)>);
type TemplateConstraintList = Vec<TemplateConstraint>;

#[derive(Debug, Default)]
struct Scanner {
    codebase: CodebaseMetadata,
    stack: Vec<Atom>,
    template_constraints: Vec<TemplateConstraintList>,
    scope: NamespaceScope,
    has_constructor: bool,
    file_type_aliases: AtomSet,
    file_imported_aliases: AtomMap<(Atom, Atom)>,
}

impl Scanner {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_current_type_resolution_context(&self) -> TypeResolutionContext {
        let mut context = TypeResolutionContext::new();
        context = context.with_type_aliases(self.file_type_aliases.clone());

        // Add imported aliases
        for (local_name, (source_class, original_name)) in &self.file_imported_aliases {
            context = context.with_imported_type_alias(*local_name, *source_class, *original_name);
        }

        for template_constraint_list in self.template_constraints.iter().rev() {
            for (name, constraints) in template_constraint_list {
                if !context.has_template_definition(name) {
                    context = context.with_template_definition(*name, constraints.clone());
                }
            }
        }

        context
    }
}

impl<'ctx, 'arena> MutWalker<'arena, 'arena, Context<'ctx, 'arena>> for Scanner {
    #[inline]
    fn walk_in_namespace(&mut self, namespace: &'arena Namespace<'arena>, _context: &mut Context<'ctx, 'arena>) {
        self.scope = match &namespace.name {
            Some(name) => NamespaceScope::for_namespace(name.value()),
            None => NamespaceScope::global(),
        };
    }

    #[inline]
    fn walk_out_namespace(&mut self, _namespace: &'arena Namespace<'arena>, _context: &mut Context<'ctx, 'arena>) {
        self.scope = NamespaceScope::global();
    }

    #[inline]
    fn walk_in_use(&mut self, r#use: &'arena Use<'arena>, _context: &mut Context<'ctx, 'arena>) {
        self.scope.populate_from_use(r#use);
    }

    #[inline]
    fn walk_in_function(&mut self, function: &'arena Function<'arena>, context: &mut Context<'ctx, 'arena>) {
        let type_context = self.get_current_type_resolution_context();

        let name = ascii_lowercase_atom(context.resolved_names.get(&function.name));
        let identifier = (empty_atom(), name);
        let metadata =
            scan_function(identifier, function, self.stack.last().copied(), context, &mut self.scope, type_context);

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in &metadata.template_types {
                constraints.push((*template_name, template_constraints.clone()));
            }

            constraints
        });

        self.codebase.function_likes.entry(identifier).or_insert(metadata);
    }

    #[inline]
    fn walk_out_function(&mut self, _function: &'arena Function<'arena>, _context: &mut Context<'ctx, 'arena>) {
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_in_closure(&mut self, closure: &'arena Closure<'arena>, context: &mut Context<'ctx, 'arena>) {
        let span = closure.span();

        let file_ref = u64_atom(span.file_id.as_u64());
        let closure_ref = u32_atom(span.start.offset);
        let identifier = (file_ref, closure_ref);

        let type_resolution_context = self.get_current_type_resolution_context();
        let metadata = scan_closure(
            identifier,
            closure,
            self.stack.last().copied(),
            context,
            &mut self.scope,
            type_resolution_context,
        );

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in &metadata.template_types {
                constraints.push((*template_name, template_constraints.clone()));
            }

            constraints
        });

        self.codebase.function_likes.entry(identifier).or_insert(metadata);
    }

    #[inline]
    fn walk_out_closure(&mut self, _closure: &'arena Closure<'arena>, _context: &mut Context<'ctx, 'arena>) {
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_in_arrow_function(
        &mut self,
        arrow_function: &'arena ArrowFunction<'arena>,
        context: &mut Context<'ctx, 'arena>,
    ) {
        let span = arrow_function.span();

        let file_ref = u64_atom(span.file_id.as_u64());
        let closure_ref = u32_atom(span.start.offset);
        let identifier = (file_ref, closure_ref);

        let type_resolution_context = self.get_current_type_resolution_context();

        let metadata = scan_arrow_function(
            identifier,
            arrow_function,
            self.stack.last().copied(),
            context,
            &mut self.scope,
            type_resolution_context,
        );

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in &metadata.template_types {
                constraints.push((*template_name, template_constraints.clone()));
            }

            constraints
        });
        self.codebase.function_likes.entry(identifier).or_insert(metadata);
    }

    #[inline]
    fn walk_out_arrow_function(
        &mut self,
        _arrow_function: &'arena ArrowFunction<'arena>,
        _context: &mut Context<'ctx, 'arena>,
    ) {
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_in_constant(&mut self, constant: &'arena Constant<'arena>, context: &mut Context<'ctx, 'arena>) {
        let constants = scan_constant(constant, context, &self.get_current_type_resolution_context(), &self.scope);

        for constant_metadata in constants {
            let constant_name = constant_metadata.name;
            self.codebase.constants.entry(constant_name).or_insert(constant_metadata);
        }
    }

    #[inline]
    fn walk_in_function_call(
        &mut self,
        function_call: &'arena FunctionCall<'arena>,
        context: &mut Context<'ctx, 'arena>,
    ) {
        let Some(constant_metadata) =
            scan_defined_constant(function_call, context, &self.get_current_type_resolution_context(), &self.scope)
        else {
            return;
        };

        self.codebase.constants.entry(constant_metadata.name).or_insert(constant_metadata);
    }

    #[inline]
    fn walk_anonymous_class(
        &mut self,
        anonymous_class: &'arena AnonymousClass<'arena>,
        context: &mut Context<'ctx, 'arena>,
    ) {
        if let Some((id, template_definition, type_aliases, imported_aliases)) =
            register_anonymous_class(&mut self.codebase, anonymous_class, context, &mut self.scope)
        {
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(template_definition);

            walk_anonymous_class_mut(self, anonymous_class, context);
        } else {
            // We don't need to walk the anonymous class if it's already been registered
        }
    }

    #[inline]
    fn walk_class(&mut self, class: &'arena Class<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_class(&mut self.codebase, class, context, &mut self.scope)
        {
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(templates);

            walk_class_mut(self, class, context);
        } else {
            // We don't need to walk the class if it's already been registered
        }
    }

    #[inline]
    fn walk_trait(&mut self, r#trait: &'arena Trait<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_trait(&mut self.codebase, r#trait, context, &mut self.scope)
        {
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(templates);

            walk_trait_mut(self, r#trait, context);
        } else {
            // We don't need to walk the trait if it's already been registered
        }
    }

    #[inline]
    fn walk_enum(&mut self, r#enum: &'arena Enum<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_enum(&mut self.codebase, r#enum, context, &mut self.scope)
        {
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(templates);

            walk_enum_mut(self, r#enum, context);
        } else {
            // We don't need to walk the enum if it's already been registered
        }
    }

    #[inline]
    fn walk_interface(&mut self, interface: &'arena Interface<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_interface(&mut self.codebase, interface, context, &mut self.scope)
        {
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(templates);

            walk_interface_mut(self, interface, context);
        }
    }

    #[inline]
    fn walk_in_method(&mut self, method: &'arena Method<'arena>, context: &mut Context<'ctx, 'arena>) {
        let current_class = self.stack.last().copied().expect("Expected class-like stack to be non-empty");
        let mut class_like_metadata =
            self.codebase.class_likes.remove(&current_class).expect("Expected class-like metadata to be present");

        let name = ascii_lowercase_atom(method.name.value);

        if class_like_metadata.methods.contains(&name) {
            if class_like_metadata.pseudo_methods.contains(&name)
                && let Some(existing_method) = self.codebase.function_likes.get_mut(&(class_like_metadata.name, name))
            {
                class_like_metadata.pseudo_methods.remove(&name);
                existing_method.flags.remove(MetadataFlags::MAGIC_METHOD);
            }

            self.codebase.class_likes.insert(current_class, class_like_metadata);
            self.template_constraints.push(vec![]);

            return;
        }

        let method_id = (class_like_metadata.name, name);
        let type_resolution_context = {
            let mut context = self.get_current_type_resolution_context();

            for alias_name in class_like_metadata.type_aliases.keys() {
                context = context.with_type_alias(*alias_name);
            }

            for (alias_name, (source_class, original_name, _span)) in &class_like_metadata.imported_type_aliases {
                context = context.with_imported_type_alias(*alias_name, *source_class, *original_name);
            }

            context
        };

        let mut function_like_metadata = scan_method(
            method_id,
            method,
            &class_like_metadata,
            context,
            &mut self.scope,
            Some(type_resolution_context),
        );

        let Some(method_metadata) = &function_like_metadata.method_metadata else {
            unreachable!("Method info should be present for method.",);
        };

        let mut is_constructor = false;
        let mut is_clone = false;
        if method_metadata.is_constructor {
            is_constructor = true;
            self.has_constructor = true;

            let type_context = self.get_current_type_resolution_context();
            for (index, param) in method.parameter_list.parameters.iter().enumerate() {
                if !param.is_promoted_property() {
                    continue;
                }

                let Some(parameter_metadata) = function_like_metadata.parameters.get_mut(index) else {
                    continue;
                };

                let property_metadata = scan_promoted_property(
                    param,
                    parameter_metadata,
                    &mut class_like_metadata,
                    current_class,
                    &type_context,
                    context,
                    &self.scope,
                );

                class_like_metadata.add_property_metadata(property_metadata);
            }
        } else {
            is_clone = name == atom("__clone");
        }

        class_like_metadata.methods.insert(name);
        let method_identifier = MethodIdentifier::new(class_like_metadata.name, name);
        class_like_metadata.add_declaring_method_id(name, method_identifier);
        if !method_metadata.visibility.is_private() || is_constructor || is_clone || class_like_metadata.kind.is_trait()
        {
            class_like_metadata.inheritable_method_ids.insert(name, method_identifier);
        }

        if method_metadata.is_final && is_constructor {
            class_like_metadata.flags |= MetadataFlags::CONSISTENT_CONSTRUCTOR;
        }

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in &function_like_metadata.template_types {
                constraints.push((*template_name, template_constraints.clone()));
            }

            constraints
        });

        self.codebase.class_likes.entry(current_class).or_insert(class_like_metadata);
        self.codebase.function_likes.entry(method_id).or_insert(function_like_metadata);
    }

    #[inline]
    fn walk_out_method(&mut self, _method: &'arena Method<'arena>, _context: &mut Context<'ctx, 'arena>) {
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_out_anonymous_class(
        &mut self,
        _anonymous_class: &'arena AnonymousClass<'arena>,
        _context: &mut Context<'ctx, 'arena>,
    ) {
        self.stack.pop().expect("Expected class stack to be non-empty");
        self.template_constraints.pop().expect("Expected template stack to be non-empty");
    }

    #[inline]
    fn walk_out_class(&mut self, _class: &'arena Class<'arena>, context: &mut Context<'ctx, 'arena>) {
        finalize_class_like(self, context);
    }

    #[inline]
    fn walk_out_trait(&mut self, _trait: &'arena Trait<'arena>, context: &mut Context<'ctx, 'arena>) {
        finalize_class_like(self, context);
    }

    #[inline]
    fn walk_out_enum(&mut self, _enum: &'arena Enum<'arena>, context: &mut Context<'ctx, 'arena>) {
        finalize_class_like(self, context);
    }

    #[inline]
    fn walk_out_interface(&mut self, _interface: &'arena Interface<'arena>, context: &mut Context<'ctx, 'arena>) {
        finalize_class_like(self, context);
    }
}

fn finalize_class_like(scanner: &mut Scanner, context: &mut Context<'_, '_>) {
    let has_constructor = scanner.has_constructor;
    scanner.has_constructor = false;

    let class_like_id = scanner.stack.pop().expect("Expected class stack to be non-empty");
    scanner.template_constraints.pop().expect("Expected template stack to be non-empty");

    if has_constructor {
        return;
    }

    let Some(mut class_like_metadata) = scanner.codebase.class_likes.remove(&class_like_id) else {
        return;
    };

    if class_like_metadata.flags.has_consistent_constructor() {
        let constructor_name = atom("__construct");

        class_like_metadata.methods.insert(constructor_name);
        let constructor_method_id = MethodIdentifier::new(class_like_metadata.name, constructor_name);
        class_like_metadata.add_declaring_method_id(constructor_name, constructor_method_id);
        class_like_metadata.inheritable_method_ids.insert(constructor_name, constructor_method_id);

        let mut flags = MetadataFlags::PURE;
        if context.file.file_type.is_host() {
            flags |= MetadataFlags::USER_DEFINED;
        } else if context.file.file_type.is_builtin() {
            flags |= MetadataFlags::BUILTIN;
        }

        scanner.codebase.function_likes.insert(
            (class_like_metadata.name, constructor_name),
            FunctionLikeMetadata::new(FunctionLikeKind::Method, class_like_metadata.span, flags),
        );
    }

    scanner.codebase.class_likes.insert(class_like_id, class_like_metadata);
}
