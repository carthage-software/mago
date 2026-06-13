use std::collections::HashMap;

use mago_allocator::LocalArena;
use mago_oracle::name::Name;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::refines;
use mago_oracle::ty::well_known;
use mago_oracle::world::ClassConstant;
use mago_oracle::world::ClassProperty;
use mago_oracle::world::EnumBacking;
use mago_oracle::world::TemplateParameter;
use mago_oracle::world::World;

struct StubCodebase<'world> {
    return_types: HashMap<Vec<u8>, Type<'world>>,
    edges: HashMap<Vec<u8>, Vec<u8>>,
}

impl<'world> StubCodebase<'world> {
    fn new() -> Self {
        Self { return_types: HashMap::new(), edges: HashMap::new() }
    }

    fn declare_function(&mut self, name: &str, return_type: Type<'world>) {
        self.return_types.insert(name.as_bytes().to_vec(), return_type);
    }

    fn declare_edge(&mut self, child: &str, parent: &str) {
        self.edges.insert(child.as_bytes().to_vec(), parent.as_bytes().to_vec());
    }

    fn return_type(&self, name: &str) -> Option<Type<'world>> {
        self.return_types.get(name.as_bytes()).copied()
    }
}

impl<'world: 'arena, 'arena> World<'arena> for StubCodebase<'world> {
    fn descends_from(&self, child: Name<'_>, ancestor: Name<'_>) -> bool {
        if child.as_bytes() == ancestor.as_bytes() {
            return true;
        }

        let mut current = child.as_bytes().to_vec();
        while let Some(parent) = self.edges.get(&current) {
            if parent.as_slice() == ancestor.as_bytes() {
                return true;
            }

            current.clone_from(parent);
        }

        false
    }

    fn uses_trait(&self, _class: Name<'_>, _trait_name: Name<'_>) -> bool {
        false
    }

    fn template_parameter_arity(&self, _class: Name<'_>) -> usize {
        0
    }

    fn template_parameter_at(&self, _class: Name<'_>, _position: usize) -> Option<TemplateParameter<'arena>> {
        None
    }

    fn template_parameter_index(&self, _class: Name<'_>, _name: Name<'_>) -> Option<usize> {
        None
    }

    fn inherited_template_argument(
        &self,
        _child: Name<'_>,
        _ancestor: Name<'_>,
        _position: usize,
    ) -> Option<Type<'arena>> {
        None
    }

    fn template_parameter_forwards_to(
        &self,
        from_class: Name<'_>,
        from_parameter: Name<'_>,
        to_class: Name<'_>,
        to_parameter: Name<'_>,
    ) -> bool {
        from_class.as_bytes() == to_class.as_bytes() && from_parameter.as_bytes() == to_parameter.as_bytes()
    }

    fn class_has_method(&self, _class: Name<'_>, _method: Name<'_>) -> bool {
        false
    }

    fn class_property_type(&self, _class: Name<'_>, _property: Name<'_>) -> Option<Type<'arena>> {
        None
    }

    fn class_has_property(&self, _class: Name<'_>, _property: Name<'_>) -> bool {
        false
    }

    fn enum_backing(&self, _enum_name: Name<'_>) -> Option<EnumBacking<'arena>> {
        None
    }

    fn class_like_kind(&self, name: Name<'_>) -> Option<ClassLikeKind> {
        if self.edges.contains_key(name.as_bytes()) {
            return Some(ClassLikeKind::Class);
        }

        None
    }

    fn is_final(&self, _name: Name<'_>) -> bool {
        false
    }

    fn alias_body(&self, _class: Name<'_>, _alias: Name<'_>) -> Option<Type<'arena>> {
        None
    }

    fn class_constant_type(&self, _class: Name<'_>, _constant: Name<'_>) -> Option<Type<'arena>> {
        None
    }

    fn class_constants(&self, _class: Name<'_>) -> &[ClassConstant<'arena>] {
        &[]
    }

    fn enum_cases(&self, _enum_name: Name<'_>) -> &[Name<'arena>] {
        &[]
    }

    fn global_constant_type(&self, name: Name<'_>) -> Option<Type<'arena>> {
        self.return_types.get(name.as_bytes()).copied()
    }

    fn class_property_count(&self, _class: Name<'_>) -> usize {
        0
    }

    fn class_property_at(&self, _class: Name<'_>, _position: usize) -> Option<ClassProperty<'arena>> {
        None
    }

    fn sealed_direct_inheritors(&self, _class_like: Name<'_>) -> Option<&[Name<'arena>]> {
        None
    }

    fn sealed_parent_of(&self, _child: Name<'_>) -> Option<Name<'arena>> {
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
