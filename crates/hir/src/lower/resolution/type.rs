use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use crate::ir::identifier::Identifier;
use crate::ir::item::annotation::generics::InheritedTypeParameterAnnotation;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;

#[derive(Debug, Clone, Copy)]
struct TemplateParameter<'arena> {
    name: Name<'arena>,
    bound: Option<&'arena TypeAnnotation<'arena>>,
    default: Option<&'arena TypeAnnotation<'arena>>,
}

#[derive(Debug, Clone, Copy)]
struct TypeAlias<'arena> {
    local_name: &'arena [u8],
    source_class: Identifier<'arena>,
    original_name: Name<'arena>,
}

#[derive(Debug)]
struct Scope<'scratch, 'arena, S>
where
    S: Arena,
{
    defining_entity: TypeParameterDefiningEntity<'arena>,
    is_static_method: bool,
    templates: Vec<'scratch, TemplateParameter<'arena>, S>,
    aliases: Vec<'scratch, TypeAlias<'arena>, S>,
}

#[derive(Debug)]
pub struct TypeResolution<'scratch, 'arena, S>
where
    S: Arena,
{
    scratch: &'scratch S,
    scopes: Vec<'scratch, Scope<'scratch, 'arena, S>, S>,
    program_aliases: Vec<'scratch, TypeAlias<'arena>, S>,
}

impl<'scratch, 'arena, S> TypeResolution<'scratch, 'arena, S>
where
    S: Arena,
{
    pub fn new_in(scratch: &'scratch S) -> TypeResolution<'scratch, 'arena, S> {
        TypeResolution { scratch, scopes: Vec::new_in(scratch), program_aliases: Vec::new_in(scratch) }
    }

    pub fn enter_scope(&mut self, defining_entity: TypeParameterDefiningEntity<'arena>) {
        self.enter_scope_with(defining_entity, false);
    }

    pub fn enter_scope_with(&mut self, defining_entity: TypeParameterDefiningEntity<'arena>, is_static_method: bool) {
        self.scopes.push(Scope {
            defining_entity,
            is_static_method,
            templates: Vec::new_in(self.scratch),
            aliases: Vec::new_in(self.scratch),
        });
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn add_template(
        &mut self,
        name: Name<'arena>,
        bound: Option<&'arena TypeAnnotation<'arena>>,
        default: Option<&'arena TypeAnnotation<'arena>>,
    ) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.templates.push(TemplateParameter { name, bound, default });
        }
    }

    /// Collects the `@template` parameters that an inner function-like (closure/arrow) inherits from
    /// its enclosing scopes. The current (own) scope on top of the stack is excluded — its templates
    /// are recorded directly on the function-like. When `inherit_static_templates` is `false`, the
    /// templates declared on a class are dropped once a `static` method scope has been crossed,
    /// matching the lexical rule that a static method cannot see instance-level type parameters.
    #[must_use]
    pub fn inherited_templates<A>(
        &self,
        arena: &'arena A,
        inherit_static_templates: bool,
    ) -> &'arena [InheritedTypeParameterAnnotation<'arena>]
    where
        A: Arena,
    {
        let Some(enclosing) = self.scopes.len().checked_sub(1) else {
            return &[];
        };

        let mut collected = Vec::new_in(self.scratch);
        let mut crossed_static_method = false;
        for scope in self.scopes[..enclosing].iter().rev() {
            let is_class_scope = matches!(scope.defining_entity, TypeParameterDefiningEntity::ClassLike(_));
            if is_class_scope && crossed_static_method && !inherit_static_templates {
                continue;
            }

            for template in scope.templates.iter() {
                collected.push(InheritedTypeParameterAnnotation {
                    span: template.name.span,
                    defining_entity: scope.defining_entity,
                    name: template.name,
                    bound: template.bound,
                    default: template.default,
                });
            }

            if scope.is_static_method {
                crossed_static_method = true;
            }
        }

        arena.alloc_slice_copy(&collected)
    }

    pub fn add_alias(
        &mut self,
        local_name: &'arena [u8],
        source_class: Identifier<'arena>,
        original_name: Name<'arena>,
    ) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.aliases.push(TypeAlias { local_name, source_class, original_name });
        }
    }

    /// Registers a type alias that stays resolvable across the whole program rather than only within
    /// the scope that declares it. Used for the program-wide-alias and re-export-alias behaviors.
    pub fn add_program_alias(
        &mut self,
        local_name: &'arena [u8],
        source_class: Identifier<'arena>,
        original_name: Name<'arena>,
    ) {
        self.program_aliases.push(TypeAlias { local_name, source_class, original_name });
    }

    #[must_use]
    pub fn lookup_template(
        &self,
        name: &[u8],
    ) -> Option<(TypeParameterDefiningEntity<'arena>, Option<&'arena TypeAnnotation<'arena>>)> {
        self.scopes.iter().rev().find_map(|scope| {
            scope
                .templates
                .iter()
                .find(|template| template.name.value == name)
                .map(|template| (scope.defining_entity, template.bound))
        })
    }

    /// The class-like that lexically encloses the current scope, if any. Used to resolve the
    /// `self`/`static`/`$this` keywords to a concrete class while lowering types.
    #[must_use]
    pub fn enclosing_class(&self) -> Option<Identifier<'arena>> {
        self.scopes.iter().rev().find_map(|scope| match scope.defining_entity {
            TypeParameterDefiningEntity::ClassLike(identifier) => Some(identifier),
            TypeParameterDefiningEntity::Method(identifier, _) => Some(identifier),
            _ => None,
        })
    }

    #[must_use]
    pub fn lookup_alias(&self, name: &[u8]) -> Option<(Identifier<'arena>, Name<'arena>)> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.aliases.iter().find(|alias| alias.local_name == name))
            .or_else(|| self.program_aliases.iter().find(|alias| alias.local_name == name))
            .map(|alias| (alias.source_class, alias.original_name))
    }
}
