use std::collections::HashMap;

use mago_allocator::LocalArena;
use mago_oracle::id::SymbolId;
use mago_oracle::path::Path;
use mago_oracle::symbol::class_like::part::constant::ClassLikeConstantMember;
use mago_oracle::symbol::class_like::part::enum_case::EnumCaseMember;
use mago_oracle::symbol::class_like::part::inheritance::InheritedType;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::refines;
use mago_oracle::ty::well_known;
use mago_oracle::world::ClassProperty;
use mago_oracle::world::EnumBacking;
use mago_oracle::world::TemplateParameter;
use mago_oracle::world::World;

struct StubCodebase<'world> {
    return_types: HashMap<SymbolId, Type<'world>>,
    edges: HashMap<SymbolId, SymbolId>,
}

fn class_id(text: &str) -> SymbolId {
    SymbolId::class_like(text.as_bytes())
}

impl<'world> StubCodebase<'world> {
    fn new() -> Self {
        Self { return_types: HashMap::new(), edges: HashMap::new() }
    }

    fn declare_function(&mut self, name: &str, return_type: Type<'world>) {
        self.return_types.insert(SymbolId::constant(name.as_bytes()), return_type);
    }

    fn declare_edge(&mut self, child: &str, parent: &str) {
        self.edges.insert(class_id(child), class_id(parent));
    }

    fn return_type(&self, name: &str) -> Option<Type<'world>> {
        self.return_types.get(&SymbolId::constant(name.as_bytes())).copied()
    }
}

impl<'world: 'arena, 'arena> World<'arena> for StubCodebase<'world> {
    fn descends_from(&self, child: SymbolId, ancestor: SymbolId) -> bool {
        if child == ancestor {
            return true;
        }

        let mut current = child;
        while let Some(parent) = self.edges.get(&current) {
            if *parent == ancestor {
                return true;
            }

            current = *parent;
        }

        false
    }

    fn uses_trait(&self, _class: SymbolId, _trait_name: SymbolId) -> bool {
        false
    }

    fn template_parameter_arity(&self, _class: SymbolId) -> usize {
        0
    }

    fn template_parameter_at(&self, _class: SymbolId, _position: usize) -> Option<TemplateParameter<'arena>> {
        None
    }

    fn template_parameter_index(&self, _class: SymbolId, _name: &[u8]) -> Option<usize> {
        None
    }

    fn inherited_template_argument(
        &self,
        _child: SymbolId,
        _ancestor: SymbolId,
        _position: usize,
    ) -> Option<Type<'arena>> {
        None
    }

    fn template_parameter_forwards_to(
        &self,
        from_class: SymbolId,
        from_parameter: &[u8],
        to_class: SymbolId,
        to_parameter: &[u8],
    ) -> bool {
        from_class == to_class && from_parameter == to_parameter
    }

    fn class_has_method(&self, _class: SymbolId, _method: &[u8]) -> bool {
        false
    }

    fn class_property_type(&self, _class: SymbolId, _property: &[u8]) -> Option<Type<'arena>> {
        None
    }

    fn class_has_property(&self, _class: SymbolId, _property: &[u8]) -> bool {
        false
    }

    fn enum_backing(&self, _enum_name: SymbolId) -> Option<EnumBacking<'arena>> {
        None
    }

    fn class_like_kind(&self, name: SymbolId) -> Option<ClassLikeKind> {
        if self.edges.contains_key(&name) {
            return Some(ClassLikeKind::Class);
        }

        None
    }

    fn is_final(&self, _name: SymbolId) -> bool {
        false
    }

    fn alias_body(&self, _class: SymbolId, _alias: &[u8]) -> Option<Type<'arena>> {
        None
    }

    fn class_constant_type(&self, _class: SymbolId, _constant: &[u8]) -> Option<Type<'arena>> {
        None
    }

    fn class_constants(&self, _class: SymbolId) -> &[ClassLikeConstantMember<'arena>] {
        &[]
    }

    fn enum_cases(&self, _enum_name: SymbolId) -> &[EnumCaseMember<'arena>] {
        &[]
    }

    fn global_constant_type(&self, name: SymbolId) -> Option<Type<'arena>> {
        self.return_types.get(&name).copied()
    }

    fn class_property_count(&self, _class: SymbolId) -> usize {
        0
    }

    fn class_property_at(&self, _class: SymbolId, _position: usize) -> Option<ClassProperty<'arena>> {
        None
    }

    fn sealed_direct_inheritors(&self, _class_like: SymbolId) -> Option<&[InheritedType<'arena>]> {
        None
    }

    fn sealed_parent_of(&self, _child: SymbolId) -> Option<Path<'arena>> {
        None
    }
}

#[test]
fn world_types_serve_shorter_file_lifetimes_covariantly() {
    let world_arena = LocalArena::new();
    let world_scratch = LocalArena::new();
    let mut world_builder = TypeBuilder::new(&world_arena, &world_scratch);

    let mut codebase = StubCodebase::new();
    let collection = world_builder.object_named(b"Collection");
    let nullable_collection = world_builder.union_of(&[well_known::NULL, collection]);
    codebase.declare_function("make_collection", nullable_collection);
    codebase.declare_edge("ArrayCollection", "Collection");

    for _file in 0..3 {
        let file_arena = LocalArena::new();
        let file_scratch = LocalArena::new();
        let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);

        let array_collection = file_builder.object_named(b"ArrayCollection");
        let inferred = file_builder.union_of(&[array_collection]);

        let Some(declared) = codebase.return_type("make_collection") else {
            panic!("the stub world must know make_collection");
        };

        let mut report = LatticeReport::new();
        assert!(refines(inferred, declared, &codebase, LatticeOptions::default(), &mut report, &mut file_builder));

        let mixed_union = file_builder.union_of(&[well_known::NULL, array_collection]);
        let mut report = LatticeReport::new();
        assert!(refines(mixed_union, declared, &codebase, LatticeOptions::default(), &mut report, &mut file_builder));

        let stranger = file_builder.object_named(b"Stranger");
        let stranger_type = file_builder.union_of(&[stranger]);
        let mut report = LatticeReport::new();
        assert!(!refines(
            stranger_type,
            declared,
            &codebase,
            LatticeOptions::default(),
            &mut report,
            &mut file_builder
        ));
    }
}

#[test]
fn world_atoms_embed_into_file_types_without_copying() {
    let world_arena = LocalArena::new();
    let world_scratch = LocalArena::new();
    let mut world_builder = TypeBuilder::new(&world_arena, &world_scratch);

    let collection = world_builder.object_named(b"Collection");
    let world_type = world_builder.union_of(&[collection]);

    let file_arena = LocalArena::new();
    let file_scratch = LocalArena::new();
    let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);

    let mut atoms = world_type.atoms.to_vec();
    atoms.push(well_known::NULL);
    let file_type = file_builder.union_of(&atoms);

    assert_eq!(file_type.to_string(), "Collection|null");
    assert!(file_type.atoms.contains(&collection));
}

#[test]
fn imported_world_types_are_consed_in_the_file_arena() {
    let world_arena = LocalArena::new();
    let world_scratch = LocalArena::new();
    let mut world_builder = TypeBuilder::new(&world_arena, &world_scratch);

    let collection = world_builder.object_named(b"Collection");
    let world_type = world_builder.union_of(&[well_known::NULL, collection]);

    let file_arena = LocalArena::new();
    let file_scratch = LocalArena::new();
    let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);

    let imported = file_builder.import(world_type);
    let again = file_builder.import(world_type);

    assert_eq!(imported, world_type);
    assert!(imported.ptr_eq(&again));
}
