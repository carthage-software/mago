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
use mago_syntax::ast::Call;
use mago_syntax::ast::Class;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Constant;
use mago_syntax::ast::Enum;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Function;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::If;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::Interface;
use mago_syntax::ast::Method;
use mago_syntax::ast::Namespace;
use mago_syntax::ast::Program;
use mago_syntax::ast::Trait;
use mago_syntax::ast::Trivia;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
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
use crate::ttype::template::GenericTemplate;

mod attribute;
mod class_like;
mod class_like_constant;
mod constant;
mod docblock;
mod enum_case;
mod function_like;

pub mod inference;

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
        get_docblock_for_node(self.program, node)
    }
}

type TemplateConstraint = (Atom, GenericTemplate);
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
    polyfill_depth: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PolyfillGuardBranch {
    Then,
    Else,
    None,
}

const POLYFILL_GUARD_FUNCTIONS: &[&str] =
    &["class_exists", "interface_exists", "trait_exists", "enum_exists", "function_exists", "defined"];

fn classify_polyfill_guard(cond: &Expression<'_>) -> PolyfillGuardBranch {
    let cond = cond.unparenthesized();

    if is_polyfill_existence_check(cond) {
        return PolyfillGuardBranch::Else;
    }

    if let Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Not(_), operand }) = cond
        && is_polyfill_existence_check(operand.unparenthesized())
    {
        return PolyfillGuardBranch::Then;
    }

    PolyfillGuardBranch::None
}

/// Returns true if `expr` is a call to one of the recognized existence-check
/// functions (regardless of whether it's written as `class_exists` or
/// `\class_exists` — we just look at the trailing segment).
fn is_polyfill_existence_check(expr: &Expression<'_>) -> bool {
    let Expression::Call(Call::Function(FunctionCall { function, .. })) = expr.unparenthesized() else {
        return false;
    };
    let Expression::Identifier(identifier) = function.unparenthesized() else {
        return false;
    };
    let last = identifier.last_segment();
    POLYFILL_GUARD_FUNCTIONS.iter().any(|name| last.eq_ignore_ascii_case(name))
}

impl Scanner {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_current_type_resolution_context(&self) -> TypeResolutionContext {
        let mut context = TypeResolutionContext::new();
        context = context.with_type_aliases(self.file_type_aliases.clone());

        for (local_name, (source_class, original_name)) in &self.file_imported_aliases {
            context = context.with_imported_type_alias(*local_name, *source_class, *original_name);
        }

        for template_constraint_list in self.template_constraints.iter().rev() {
            for (name, constraint) in template_constraint_list {
                if !context.has_template_definition(*name) {
                    context = context.with_template_definition(*name, vec![constraint.clone()]);
                }
            }
        }

        context
    }

    fn apply_polyfill_flag_to_class_like(&mut self, id: Atom) {
        if self.polyfill_depth == 0 {
            return;
        }

        if let Some(metadata) = self.codebase.class_likes.get_mut(&id) {
            metadata.flags |= MetadataFlags::POLYFILL;
        }
    }
}

#[allow(clippy::expect_used)]
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

    fn walk_if(&mut self, r#if: &'arena If<'arena>, context: &mut Context<'ctx, 'arena>) {
        self.walk_keyword(&r#if.r#if, context);
        self.walk_expression(r#if.condition, context);

        let guard = classify_polyfill_guard(r#if.condition);

        match &r#if.body {
            IfBody::Statement(body) => {
                let then_polyfill = matches!(guard, PolyfillGuardBranch::Then);
                if then_polyfill {
                    self.polyfill_depth = self.polyfill_depth.saturating_add(1);
                }
                self.walk_statement(body.statement, context);
                if then_polyfill {
                    self.polyfill_depth = self.polyfill_depth.saturating_sub(1);
                }

                for else_if_clause in &body.else_if_clauses {
                    self.walk_if_statement_body_else_if_clause(else_if_clause, context);
                }

                if let Some(else_clause) = &body.else_clause {
                    let else_polyfill = matches!(guard, PolyfillGuardBranch::Else);
                    if else_polyfill {
                        self.polyfill_depth = self.polyfill_depth.saturating_add(1);
                    }
                    self.walk_if_statement_body_else_clause(else_clause, context);
                    if else_polyfill {
                        self.polyfill_depth = self.polyfill_depth.saturating_sub(1);
                    }
                }
            }
            IfBody::ColonDelimited(body) => {
                let then_polyfill = matches!(guard, PolyfillGuardBranch::Then);
                if then_polyfill {
                    self.polyfill_depth = self.polyfill_depth.saturating_add(1);
                }
                for statement in &body.statements {
                    self.walk_statement(statement, context);
                }
                if then_polyfill {
                    self.polyfill_depth = self.polyfill_depth.saturating_sub(1);
                }

                for else_if_clause in &body.else_if_clauses {
                    self.walk_if_colon_delimited_body_else_if_clause(else_if_clause, context);
                }

                if let Some(else_clause) = &body.else_clause {
                    let else_polyfill = matches!(guard, PolyfillGuardBranch::Else);
                    if else_polyfill {
                        self.polyfill_depth = self.polyfill_depth.saturating_add(1);
                    }
                    self.walk_if_colon_delimited_body_else_clause(else_clause, context);
                    if else_polyfill {
                        self.polyfill_depth = self.polyfill_depth.saturating_sub(1);
                    }
                }

                self.walk_keyword(&body.endif, context);
                self.walk_terminator(&body.terminator, context);
            }
        }
    }

    #[inline]
    fn walk_in_function(&mut self, function: &'arena Function<'arena>, context: &mut Context<'ctx, 'arena>) {
        let type_context = self.get_current_type_resolution_context();

        let name = ascii_lowercase_atom(context.resolved_names.get(&function.name));
        let identifier = (empty_atom(), name);
        let mut metadata = scan_function(
            identifier,
            function,
            self.stack.last().copied(),
            context,
            &mut self.scope,
            type_context,
            Some(&self.codebase.constants),
        );

        self.template_constraints.push({
            let mut constraints: TemplateConstraintList = vec![];
            for (template_name, template_constraints) in &metadata.template_types {
                constraints.push((*template_name, template_constraints.clone()));
            }

            constraints
        });

        if self.polyfill_depth > 0 {
            metadata.flags |= MetadataFlags::POLYFILL;
        }

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

        for mut constant_metadata in constants {
            if self.polyfill_depth > 0 {
                constant_metadata.flags |= MetadataFlags::POLYFILL;
            }
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
        let Some(mut constant_metadata) =
            scan_defined_constant(function_call, context, &self.get_current_type_resolution_context(), &self.scope)
        else {
            return;
        };

        if self.polyfill_depth > 0 {
            constant_metadata.flags |= MetadataFlags::POLYFILL;
        }

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
            self.apply_polyfill_flag_to_class_like(id);
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(template_definition);

            walk_anonymous_class_mut(self, anonymous_class, context);
        }
    }

    #[inline]
    fn walk_class(&mut self, class: &'arena Class<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_class(&mut self.codebase, class, context, &mut self.scope)
        {
            self.apply_polyfill_flag_to_class_like(id);
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(templates);

            walk_class_mut(self, class, context);
        }
    }

    #[inline]
    fn walk_trait(&mut self, r#trait: &'arena Trait<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_trait(&mut self.codebase, r#trait, context, &mut self.scope)
        {
            self.apply_polyfill_flag_to_class_like(id);
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(templates);

            walk_trait_mut(self, r#trait, context);
        }
    }

    #[inline]
    fn walk_enum(&mut self, r#enum: &'arena Enum<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_enum(&mut self.codebase, r#enum, context, &mut self.scope)
        {
            self.apply_polyfill_flag_to_class_like(id);
            self.file_type_aliases.extend(type_aliases);
            self.file_imported_aliases.extend(imported_aliases);
            self.stack.push(id);
            self.template_constraints.push(templates);

            walk_enum_mut(self, r#enum, context);
        }
    }

    #[inline]
    fn walk_interface(&mut self, interface: &'arena Interface<'arena>, context: &mut Context<'ctx, 'arena>) {
        if let Some((id, templates, type_aliases, imported_aliases)) =
            register_interface(&mut self.codebase, interface, context, &mut self.scope)
        {
            self.apply_polyfill_flag_to_class_like(id);
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

        #[allow(clippy::unreachable)]
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

#[allow(clippy::expect_used)]
fn finalize_class_like(scanner: &mut Scanner, context: &Context<'_, '_>) {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod polyfill_tests {
    use std::borrow::Cow;

    use bumpalo::Bump;

    use mago_atom::ascii_lowercase_atom;
    use mago_atom::atom;
    use mago_atom::empty_atom;
    use mago_database::Database;
    use mago_database::DatabaseConfiguration;
    use mago_database::DatabaseReader;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::parser::parse_file;

    use crate::metadata::CodebaseMetadata;
    use crate::metadata::flags::MetadataFlags;
    use crate::scanner::scan_program;

    fn scan(code: &'static str) -> CodebaseMetadata {
        let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Borrowed(code));
        let config =
            DatabaseConfiguration::new(std::path::Path::new("/"), vec![], vec![], vec![], vec![]).into_static();
        let database = Database::single(file, config);

        let mut codebase = CodebaseMetadata::new();
        let arena = Bump::new();
        for file in database.files() {
            let program = parse_file(&arena, &file);
            assert!(!program.has_errors(), "parse failed: {:?}", program.errors);
            let resolved_names = NameResolver::new(&arena).resolve(program);
            codebase.extend(scan_program(&arena, &file, program, &resolved_names));
        }
        codebase
    }

    fn class_flags(codebase: &CodebaseMetadata, name: &str) -> MetadataFlags {
        codebase
            .class_likes
            .get(&ascii_lowercase_atom(name))
            .unwrap_or_else(|| panic!("class-like `{name}` not found; have {:?}", codebase.class_likes.keys()))
            .flags
    }

    fn function_flags(codebase: &CodebaseMetadata, name: &str) -> MetadataFlags {
        codebase
            .function_likes
            .get(&(empty_atom(), ascii_lowercase_atom(name)))
            .unwrap_or_else(|| panic!("function `{name}` not found"))
            .flags
    }

    fn constant_flags(codebase: &CodebaseMetadata, name: &str) -> MetadataFlags {
        codebase.constants.get(&atom(name)).unwrap_or_else(|| panic!("constant `{name}` not found")).flags
    }

    #[test]
    fn class_in_not_class_exists_is_polyfill() {
        let code = "<?php
            if (!class_exists('Foo')) {
                class Foo {}
            }
        ";
        assert!(class_flags(&scan(code), "Foo").is_polyfill());
    }

    #[test]
    fn interface_in_not_interface_exists_is_polyfill() {
        let code = "<?php
            if (!interface_exists('Bar')) {
                interface Bar {}
            }
        ";
        assert!(class_flags(&scan(code), "Bar").is_polyfill());
    }

    #[test]
    fn trait_in_not_trait_exists_is_polyfill() {
        let code = "<?php
            if (!trait_exists('Mix')) {
                trait Mix {}
            }
        ";
        assert!(class_flags(&scan(code), "Mix").is_polyfill());
    }

    #[test]
    fn enum_in_not_enum_exists_is_polyfill() {
        let code = "<?php
            if (!enum_exists('Kind')) {
                enum Kind { case A; }
            }
        ";
        assert!(class_flags(&scan(code), "Kind").is_polyfill());
    }

    #[test]
    fn function_in_not_function_exists_is_polyfill() {
        let code = "<?php
            if (!function_exists('foo')) {
                function foo(): void {}
            }
        ";
        assert!(function_flags(&scan(code), "foo").is_polyfill());
    }

    #[test]
    fn const_in_not_defined_is_polyfill() {
        let code = "<?php
            if (!defined('FOO')) {
                const FOO = 1;
            }
        ";
        assert!(constant_flags(&scan(code), "FOO").is_polyfill());
    }

    #[test]
    fn define_call_in_not_defined_is_polyfill() {
        let code = "<?php
            if (!defined('BAR')) {
                define('BAR', 1);
            }
        ";
        assert!(constant_flags(&scan(code), "BAR").is_polyfill());
    }

    #[test]
    fn class_in_else_branch_of_positive_check_is_polyfill() {
        let code = "<?php
            if (class_exists('Foo')) {
            } else {
                class Foo {}
            }
        ";
        assert!(class_flags(&scan(code), "Foo").is_polyfill());
    }

    #[test]
    fn class_in_then_branch_of_positive_check_is_not_polyfill() {
        let code = "<?php
            if (class_exists('Foo')) {
                class Bar {}
            }
        ";
        assert!(!class_flags(&scan(code), "Bar").is_polyfill());
    }

    #[test]
    fn top_level_class_is_not_polyfill() {
        let code = "<?php class Plain {}";
        assert!(!class_flags(&scan(code), "Plain").is_polyfill());
    }

    #[test]
    fn class_inside_unrelated_if_is_not_polyfill() {
        let code = "<?php
            if (PHP_VERSION_ID > 80000) {
                class Modern {}
            }
        ";
        assert!(!class_flags(&scan(code), "Modern").is_polyfill());
    }

    #[test]
    fn class_in_then_branch_when_condition_is_not_exists_check_is_not_polyfill() {
        let code = "<?php
            if (!some_other_check()) {
                class Other {}
            }
        ";
        assert!(!class_flags(&scan(code), "Other").is_polyfill());
    }

    #[test]
    fn polyfill_flag_does_not_leak_to_siblings() {
        let code = "<?php
            if (!class_exists('Polyfilled')) {
                class Polyfilled {}
            }

            class Real {}
        ";
        let codebase = scan(code);
        assert!(class_flags(&codebase, "Polyfilled").is_polyfill());
        assert!(!class_flags(&codebase, "Real").is_polyfill());
    }

    #[test]
    fn class_inside_else_does_not_leak_to_preceding_sibling() {
        let code = "<?php
            if (class_exists('Gate')) {
                class Sibling {}
            } else {
                class Gate {}
            }
        ";
        let codebase = scan(code);
        assert!(!class_flags(&codebase, "Sibling").is_polyfill());
        assert!(class_flags(&codebase, "Gate").is_polyfill());
    }

    #[test]
    fn class_nested_inside_polyfill_guard_is_still_polyfill() {
        let code = "<?php
            if (!class_exists('Wrapper')) {
                if (PHP_VERSION_ID >= 80000) {
                    class Wrapper {}
                }
            }
        ";
        assert!(class_flags(&scan(code), "Wrapper").is_polyfill());
    }

    #[test]
    fn nested_polyfill_guards_unwind_correctly() {
        let code = "<?php
            if (!class_exists('A')) {
                class A {}
            }
            class B {}
            if (!class_exists('C')) {
                class C {}
            }
            class D {}
        ";
        let codebase = scan(code);
        assert!(class_flags(&codebase, "A").is_polyfill());
        assert!(!class_flags(&codebase, "B").is_polyfill());
        assert!(class_flags(&codebase, "C").is_polyfill());
        assert!(!class_flags(&codebase, "D").is_polyfill());
    }

    #[test]
    fn polyfill_within_namespace_gets_full_fqn_flagged() {
        let code = r#"<?php
            namespace Pkg;
            if (!class_exists('Pkg\\Stub')) {
                class Stub {}
            }
        "#;
        assert!(class_flags(&scan(code), "Pkg\\Stub").is_polyfill());
    }

    #[test]
    fn class_in_alternative_syntax_then_branch_is_polyfill() {
        let code = "<?php
            if (!class_exists('Alt')):
                class Alt {}
            endif;
        ";
        assert!(class_flags(&scan(code), "Alt").is_polyfill());
    }

    #[test]
    fn class_in_alternative_syntax_else_branch_is_polyfill() {
        let code = "<?php
            if (class_exists('AltElse')):
            else:
                class AltElse {}
            endif;
        ";
        assert!(class_flags(&scan(code), "AltElse").is_polyfill());
    }

    #[test]
    fn leading_backslash_on_guard_function_is_recognized() {
        let code = r#"<?php
            if (!\class_exists('Qualified')) {
                class Qualified {}
            }
        "#;
        assert!(class_flags(&scan(code), "Qualified").is_polyfill());
    }

    #[test]
    fn guard_function_case_insensitive() {
        let code = "<?php
            if (!CLASS_EXISTS('Uppercase')) {
                class Uppercase {}
            }
        ";
        assert!(class_flags(&scan(code), "Uppercase").is_polyfill());
    }

    #[test]
    fn parenthesized_guard_expression_is_recognized() {
        let code = "<?php
            if (!(class_exists('Parenned'))) {
                class Parenned {}
            }
        ";
        assert!(class_flags(&scan(code), "Parenned").is_polyfill());
    }

    #[test]
    fn doubly_parenthesized_guard_is_recognized() {
        let code = "<?php
            if ((!((class_exists('DoubleParen'))))) {
                class DoubleParen {}
            }
        ";
        assert!(class_flags(&scan(code), "DoubleParen").is_polyfill());
    }

    #[test]
    fn class_in_elseif_branch_is_not_polyfill() {
        let code = "<?php
            if (false) {
            } elseif (!class_exists('Never')) {
                class Never {}
            }
        ";
        assert!(!class_flags(&scan(code), "Never").is_polyfill());
    }

    #[test]
    fn merge_non_polyfill_overrides_polyfill() {
        let mut stub = scan(
            "<?php
            if (!class_exists('Shared')) {
                class Shared {}
            }
        ",
        );
        let real = scan("<?php class Shared { public int $x = 1; }");
        stub.extend(real);
        let flags = class_flags(&stub, "Shared");
        assert!(!flags.is_polyfill(), "polyfill should have been replaced by real: flags = {flags:?}");
    }

    #[test]
    fn merge_polyfill_does_not_override_non_polyfill() {
        let mut real = scan("<?php class Shared { public int $x = 1; }");
        let stub = scan(
            "<?php
            if (!class_exists('Shared')) {
                class Shared {}
            }
        ",
        );
        real.extend(stub);
        assert!(!class_flags(&real, "Shared").is_polyfill());
    }

    #[test]
    fn merge_only_polyfill_is_kept() {
        let codebase = scan(
            "<?php
            if (!class_exists('OnlyStub')) {
                class OnlyStub {}
            }
        ",
        );
        assert!(class_flags(&codebase, "OnlyStub").is_polyfill());
    }

    #[test]
    fn merge_function_non_polyfill_overrides_polyfill() {
        let mut stub = scan(
            "<?php
            if (!function_exists('array_is_list')) {
                function array_is_list(array $arr): bool { return true; }
            }
        ",
        );
        let real = scan("<?php function array_is_list(array $arr): bool { return false; }");
        stub.extend(real);
        assert!(!function_flags(&stub, "array_is_list").is_polyfill());
    }

    #[test]
    fn merge_constant_non_polyfill_overrides_polyfill() {
        let mut stub = scan(
            "<?php
            if (!defined('MY_CONST')) {
                const MY_CONST = 1;
            }
        ",
        );
        let real = scan("<?php const MY_CONST = 2;");
        stub.extend(real);
        assert!(!constant_flags(&stub, "MY_CONST").is_polyfill());
    }

    #[test]
    fn phpunit_test_case_stub_scenario_prefers_real() {
        let mut codebase = scan(
            r#"<?php
            namespace PHPUnit\Framework;

            if (!class_exists('PHPUnit\\Framework\\TestCase')) {
                abstract class TestCase {}
            }
        "#,
        );
        let real = scan(
            r#"<?php
            namespace PHPUnit\Framework {
                abstract class Assert {}
                abstract class TestCase extends Assert {}
            }
        "#,
        );
        codebase.extend(real);

        let tc = codebase
            .class_likes
            .get(&ascii_lowercase_atom("PHPUnit\\Framework\\TestCase"))
            .expect("TestCase should be present in merged codebase");

        assert!(!tc.flags.is_polyfill(), "merged TestCase should be the real definition");
        assert_eq!(
            tc.direct_parent_class.map(|p| p.to_string()),
            Some("PHPUnit\\Framework\\Assert".to_ascii_lowercase()),
        );
    }
}
