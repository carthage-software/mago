use fennec_interner::ThreadedInterner;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_semantics::Semantics;

#[derive(Debug)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub semantics: &'a Semantics,
    pub scope: Vec<ClassLikeReflection>,
}

impl<'a> Context<'a> {
    pub fn new(interner: &'a ThreadedInterner, semantics: &'a Semantics) -> Self {
        Self { interner, semantics, scope: Vec::new() }
    }

    pub fn enter_scope(&mut self, scope: ClassLikeReflection) {
        self.scope.push(scope);
    }

    pub fn exit_scope(&mut self) -> Option<ClassLikeReflection> {
        self.scope.pop()
    }

    pub fn get_scope(&self) -> Option<&ClassLikeReflection> {
        self.scope.last()
    }
}
