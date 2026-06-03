use bumpalo::Bump;
use bumpalo::collections::Vec;

use crate::ir::generics::TypeParameterDefiningEntity;
use crate::ir::identifier::Identifier;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;

#[derive(Debug, Clone, Copy)]
struct TemplateParameter<'arena> {
    name: &'arena [u8],
    bound: Option<&'arena TypeAnnotation<'arena>>,
}

#[derive(Debug, Clone, Copy)]
struct TypeAlias<'arena> {
    local_name: &'arena [u8],
    source_class: Identifier<'arena>,
    original_name: Name<'arena>,
}

#[derive(Debug)]
struct Scope<'arena> {
    defining_entity: TypeParameterDefiningEntity<'arena>,
    templates: Vec<'arena, TemplateParameter<'arena>>,
    aliases: Vec<'arena, TypeAlias<'arena>>,
}

#[derive(Debug)]
pub struct TypeResolution<'arena> {
    arena: &'arena Bump,
    scopes: Vec<'arena, Scope<'arena>>,
}

impl<'arena> TypeResolution<'arena> {
    pub fn new_in(arena: &'arena Bump) -> TypeResolution<'arena> {
        TypeResolution { arena, scopes: Vec::new_in(arena) }
    }

    pub fn enter_scope(&mut self, defining_entity: TypeParameterDefiningEntity<'arena>) {
        self.scopes.push(Scope {
            defining_entity,
            templates: Vec::new_in(self.arena),
            aliases: Vec::new_in(self.arena),
        });
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn add_template(&mut self, name: &'arena [u8], bound: Option<&'arena TypeAnnotation<'arena>>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.templates.push(TemplateParameter { name, bound });
        }
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

    #[must_use]
    pub fn lookup_template(
        &self,
        name: &[u8],
    ) -> Option<(TypeParameterDefiningEntity<'arena>, Option<&'arena TypeAnnotation<'arena>>)> {
        self.scopes.iter().rev().find_map(|scope| {
            scope
                .templates
                .iter()
                .find(|template| template.name == name)
                .map(|template| (scope.defining_entity, template.bound))
        })
    }

    #[must_use]
    pub fn lookup_alias(&self, name: &[u8]) -> Option<(Identifier<'arena>, Name<'arena>)> {
        self.scopes.iter().rev().find_map(|scope| {
            scope
                .aliases
                .iter()
                .find(|alias| alias.local_name == name)
                .map(|alias| (alias.source_class, alias.original_name))
        })
    }
}
