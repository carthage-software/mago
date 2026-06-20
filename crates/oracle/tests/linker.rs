#![allow(clippy::expect_used)]

use std::borrow::Cow;

use mago_allocator::Arena;
use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_hir::ir::IR;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_oracle::definition::DefinitionTable;
use mago_oracle::definition::binder::bind;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::link;
use mago_oracle::symbol::Symbol;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::symbol::part::origin::Origin;
use mago_syntax::parser::parse_file;

/// Parses, lowers, and binds one source file into a definition table living in
/// `arena`. The parse/lower scratch is local; the table borrows only `arena`.
fn define<'arena>(
    arena: &'arena LocalArena,
    origin: Origin,
    source: &str,
) -> DefinitionTable<'arena, LocalArena, (), ()> {
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"linker.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'arena, (), (), ()> = Lowering::new(arena, &scratch, &file, program, LowerSettings::default()).lower();
    let ir = arena.alloc(ir);

    let (_typed, table) = bind(arena, origin, ir);

    table
}

fn with_link(sources: &[(Origin, &str)], check: impl FnOnce(&SymbolTable<'_, LocalArena>)) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();

    let tables: std::vec::Vec<_> = sources.iter().map(|(origin, source)| define(&arena, *origin, source)).collect();

    let table = link(&arena, &scratch, &tables);

    check(&table);
}

#[test]
fn indexes_top_level_symbols_across_files() {
    let sources = [
        (Origin::Project, "<?php class A {}"),
        (Origin::Project, "<?php class B extends A {} function f(): void {} const C = 1;"),
    ];

    with_link(&sources, |table| {
        assert!(table.get_class_like(SymbolId::class_like(b"A")).is_some(), "class A is linked");
        assert!(table.get_class_like(SymbolId::class_like(b"B")).is_some(), "class B is linked");
        assert!(table.get_function_like(SymbolId::function_like(b"f")).is_some(), "function f is linked");
        assert!(table.get_constant(SymbolId::constant(b"C")).is_some(), "constant C is linked");
    });
}

#[test]
fn records_descendants_through_extends() {
    let sources = [(Origin::Project, "<?php class A {} class B extends A {} class C extends B {}")];

    with_link(&sources, |table| {
        let a = SymbolId::class_like(b"A");
        let b = SymbolId::class_like(b"B");
        let c = SymbolId::class_like(b"C");

        assert!(table.is_direct_descendant(a, b), "B directly extends A");
        assert!(!table.is_direct_descendant(a, c), "C does not directly extend A");
        assert!(table.is_descendant(a, b), "B transitively descends from A");
        assert!(table.is_descendant(a, c), "C transitively descends from A");
    });
}

#[test]
fn resolves_collisions_by_origin_priority() {
    let sources = [
        (Origin::Dependency, "<?php function shared(): int { return 1; }"),
        (Origin::Project, "<?php function shared(): string { return 'x'; }"),
    ];

    with_link(&sources, |table| {
        let function = table.get_function_like(SymbolId::function_like(b"shared")).expect("shared is linked");
        assert_eq!(function.origin(), Origin::Project, "the project declaration wins over the dependency");
    });
}

#[test]
fn flattens_inherited_members_queryable_through_symbols() {
    let sources = [(
        Origin::Project,
        "<?php
        interface Contract { public function handle(): void; }
        class Base implements Contract {
            public function handle(): void {}
            public function shared(): void {}
            const VERSION = 1;
            protected int $state = 0;
        }
        class Derived extends Base { public function extra(): void {} }
        ",
    )];

    with_link(&sources, |table| {
        let contract = SymbolId::class_like(b"Contract");
        let base = SymbolId::class_like(b"Base");
        let derived = SymbolId::class_like(b"Derived");

        assert!(table.class_has_method(derived, b"extra"), "Derived declares extra");
        assert!(table.class_has_method(derived, b"shared"), "Derived inherits shared from Base");
        assert!(table.class_has_method(derived, b"handle"), "Derived inherits handle");
        assert!(table.class_has_property(derived, b"$state"), "Derived inherits the protected property");
        assert!(
            table
                .class_constants(derived)
                .iter()
                .any(|member| member.name.id == SymbolId::class_like_constant(b"Derived", b"VERSION")),
            "Derived inherits the VERSION constant",
        );

        assert!(table.descends_from(derived, base), "Derived descends from Base");
        assert!(table.descends_from(derived, contract), "Derived descends transitively from Contract");
        assert!(table.descends_from(base, contract), "Base implements Contract");

        let handle = table.class_constants(base);
        let _ = handle;
        assert!(matches!(table.class_like_kind(contract), Some(kind) if format!("{kind:?}") == "Interface"));
    });
}

#[test]
fn flattens_trait_methods_into_users() {
    let sources = [(
        Origin::Project,
        "<?php
        trait Greets { public function greet(): void {} }
        class Service { use Greets; public function run(): void {} }
        ",
    )];

    with_link(&sources, |table| {
        let service = SymbolId::class_like(b"Service");
        assert!(table.class_has_method(service, b"greet"), "Service gains greet from the trait");
        assert!(table.class_has_method(service, b"run"), "Service declares run");
        assert!(table.uses_trait(service, SymbolId::class_like(b"Greets")), "Service records the trait use");
    });
}

#[test]
fn records_override_edges() {
    let sources = [(
        Origin::Project,
        "<?php
        class Base { public function handle(): void {} }
        class Child extends Base { public function handle(): void {} }
        ",
    )];

    with_link(&sources, |table| {
        let child = SymbolId::class_like(b"Child");
        let class = table.get_class_like(child).expect("Child is linked");
        let methods = class.methods();

        let handle = SymbolId::method(b"Child", b"handle");
        let offset = methods.members.iter().position(|member| member.name.id == handle).expect("handle is present");

        let edge = methods
            .overrides
            .iter()
            .find(|edge| edge.member as usize == offset)
            .expect("Child::handle overrides an ancestor");
        assert!(
            edge.overrides.contains(&SymbolId::class_like(b"Base")),
            "the override edge points at the defining class Base",
        );
    });
}

#[test]
fn bridges_native_member_types() {
    use mago_oracle::ty::well_known;

    let sources = [(Origin::Project, "<?php class Box { public int $value = 0; const LIMIT = 1; }")];

    with_link(&sources, |table| {
        let box_id = SymbolId::class_like(b"Box");
        assert_eq!(
            table.class_property_type(box_id, b"$value"),
            Some(well_known::TYPE_INT),
            "the native int hint bridges into the property type slot",
        );
        assert!(
            table.class_constant_type(box_id, b"LIMIT").is_some(),
            "the class constant's value `1` is inferred, like a global constant",
        );
    });
}

#[test]
fn lowers_template_parameters() {
    let sources = [(
        Origin::Project,
        "<?php
        /**
         * @template TKey
         * @template TValue
         */
        class Collection {}
        ",
    )];

    with_link(&sources, |table| {
        let collection = SymbolId::class_like(b"Collection");
        assert_eq!(table.template_parameter_arity(collection), 2, "two @template parameters");
        assert_eq!(table.template_parameter_index(collection, b"TKey"), Some(0));
        assert_eq!(table.template_parameter_index(collection, b"TValue"), Some(1));
        assert_eq!(table.template_parameter_at(collection, 0).map(|parameter| parameter.name), Some(&b"TKey"[..]));
        assert!(table.template_parameter_forwards_to(collection, b"TKey", collection, b"TKey"), "reflexive forwarding");
    });
}

#[test]
fn bridges_phpdoc_member_types() {
    use mago_oracle::ty::well_known;

    let sources = [(
        Origin::Project,
        "<?php
        class Bag {
            /** @var int */
            public $count;
        }
        ",
    )];

    with_link(&sources, |table| {
        let bag = SymbolId::class_like(b"Bag");
        assert_eq!(
            table.class_property_type(bag, b"$count"),
            Some(well_known::TYPE_INT),
            "the @var annotation bridges into the property type slot when there is no native hint",
        );
    });
}

#[test]
fn bridges_structured_phpdoc_types_faithfully() {
    use mago_oracle::ty::AtomKind;

    let sources = [(
        Origin::Project,
        "<?php
        class Shapes {
            /** @var list<int> */
            public $list;
            /** @var array<string, int> */
            public $map;
            /** @var class-string<Throwable> */
            public $cls;
            /** @var callable(int): string */
            public $fn;
            /** @var array{a: int, b?: string} */
            public $shape;
            /** @var key-of<array<string, int>> */
            public $keys;
            /** @var stringable-object */
            public $stringable;
            /** @var int[] */
            public $slice;
            /** @var * */
            public $wildcard;
        }
        ",
    )];

    with_link(&sources, |table| {
        let shapes = SymbolId::class_like(b"Shapes");
        let kind = |property: &[u8]| {
            table.class_property_type(shapes, property).and_then(|ty| ty.atoms.first().map(|atom| atom.kind()))
        };

        assert_eq!(kind(b"$list"), Some(AtomKind::List), "list<int> bridges to a list atom, not mixed");
        assert_eq!(kind(b"$map"), Some(AtomKind::Array), "array<string,int> bridges to an array atom");
        assert_eq!(
            kind(b"$cls"),
            Some(AtomKind::ClassLikeString),
            "class-string<Throwable> bridges to a class-like-string atom",
        );
        assert_eq!(kind(b"$fn"), Some(AtomKind::Callable), "callable(int): string bridges to a callable atom");
        assert_eq!(kind(b"$shape"), Some(AtomKind::Array), "an array shape bridges to an array atom");
        assert_eq!(kind(b"$keys"), Some(AtomKind::Derived), "key-of<...> bridges to a derived atom");
        assert_eq!(
            kind(b"$stringable"),
            Some(AtomKind::Object),
            "stringable-object bridges to the Stringable named object, not bare object",
        );
        assert_eq!(kind(b"$slice"), Some(AtomKind::Array), "int[] bridges to array<array-key, int>");
        assert_eq!(kind(b"$wildcard"), Some(AtomKind::Placeholder), "the * wildcard bridges to a placeholder");
    });
}

#[test]
fn maps_annotation_tags_to_flags() {
    use mago_oracle::symbol::class_like::ClassLikeSymbol;
    use mago_oracle::symbol::class_like::class::ClassFlag;

    let sources = [(
        Origin::Project,
        "<?php
        /**
         * @final
         * @deprecated
         */
        class Legacy {}
        ",
    )];

    with_link(&sources, |table| {
        let legacy = SymbolId::class_like(b"Legacy");
        assert!(table.is_final(legacy), "@final marks the class final");

        let ClassLikeSymbol::Class(class) = table.get_class_like(legacy).expect("Legacy is linked") else {
            panic!("Legacy is a class");
        };
        assert!(class.flags.contains(ClassFlag::Deprecated), "@deprecated sets the deprecated flag");
    });
}

#[test]
fn resolves_inherited_template_arguments() {
    use mago_oracle::ty::well_known;

    let sources = [(
        Origin::Project,
        "<?php
        /** @template T */
        class Box {}
        /**
         * @extends Box<int>
         */
        class IntBox extends Box {}
        ",
    )];

    with_link(&sources, |table| {
        let box_id = SymbolId::class_like(b"Box");
        let int_box = SymbolId::class_like(b"IntBox");
        assert_eq!(
            table.inherited_template_argument(int_box, box_id, 0),
            Some(well_known::TYPE_INT),
            "IntBox passes int to Box's first type parameter",
        );
    });
}

#[test]
fn forwards_template_parameters_transitively() {
    let sources = [(
        Origin::Project,
        "<?php
        /** @template T */
        class Root {}
        /**
         * @template TM
         * @extends Root<TM>
         */
        class Middle extends Root {}
        /**
         * @template TL
         * @extends Middle<TL>
         */
        class Leaf extends Middle {}
        ",
    )];

    with_link(&sources, |table| {
        let root = SymbolId::class_like(b"Root");
        let middle = SymbolId::class_like(b"Middle");
        let leaf = SymbolId::class_like(b"Leaf");

        assert!(table.template_parameter_forwards_to(middle, b"TM", root, b"T"), "Middle forwards TM into Root::T");
        assert!(table.template_parameter_forwards_to(leaf, b"TL", middle, b"TM"), "Leaf forwards TL into Middle::TM",);
        assert!(
            table.template_parameter_forwards_to(leaf, b"TL", root, b"T"),
            "forwarding is transitively closed: Leaf forwards TL all the way into Root::T",
        );
        assert!(
            !table.template_parameter_forwards_to(leaf, b"TL", root, b"Other"),
            "forwarding does not invent unrelated targets",
        );
    });
}

#[test]
fn records_sealed_inheritors() {
    let sources = [(
        Origin::Project,
        "<?php
        /**
         * @sealed Dog|Cat
         */
        interface Animal {}
        interface Dog extends Animal {}
        interface Cat extends Animal {}
        ",
    )];

    with_link(&sources, |table| {
        let animal = SymbolId::class_like(b"Animal");
        let dog = SymbolId::class_like(b"Dog");

        let inheritors = table.sealed_direct_inheritors(animal).expect("Animal is sealed");
        assert_eq!(inheritors.len(), 2, "Animal permits exactly two inheritors");
        assert!(inheritors.iter().any(|edge| edge.target.id == dog), "Dog is among Animal's permitted inheritors",);

        assert!(table.sealed_parent_of(dog).is_some(), "Dog records its sealed parent");
    });
}

#[test]
fn records_type_aliases() {
    let sources = [(
        Origin::Project,
        "<?php
        /**
         * @type Id = int
         */
        class Repository {}
        ",
    )];

    with_link(&sources, |table| {
        let repository = SymbolId::class_like(b"Repository");
        assert!(table.alias_body(repository, b"Id").is_some(), "the @type alias body is recorded");
    });
}

#[test]
fn synthesizes_virtual_members() {
    use mago_oracle::symbol::class_like::ClassLikeSymbol;

    let sources = [(
        Origin::Project,
        "<?php
        /**
         * @method int compute(string $input, int $scale = 2)
         * @property string $label
         */
        class Proxy {}
        ",
    )];

    with_link(&sources, |table| {
        let proxy = SymbolId::class_like(b"Proxy");
        assert!(table.class_has_method(proxy, b"compute"), "@method synthesizes a virtual method");
        assert!(table.class_has_property(proxy, b"$label"), "@property synthesizes a virtual property");

        let ClassLikeSymbol::Class(class) = table.get_class_like(proxy).expect("Proxy is linked") else {
            panic!("Proxy is a class");
        };
        let compute = class
            .methods
            .members
            .iter()
            .find(|method| method.name.id == SymbolId::method(b"Proxy", b"compute"))
            .expect("compute is synthesized");
        assert!(
            compute.params.iter().any(|parameter| parameter.default_ty.effective(true).is_some()),
            "the @method's `int $scale = 2` default value is inferred, not skipped",
        );
    });
}

#[test]
fn applies_trait_alias_adaptation() {
    let sources = [(
        Origin::Project,
        "<?php
        trait Greets { public function greet(): void {} }
        class Service {
            use Greets { greet as welcome; }
        }
        ",
    )];

    with_link(&sources, |table| {
        let service = SymbolId::class_like(b"Service");
        assert!(table.class_has_method(service, b"greet"), "the original trait method survives the alias");
        assert!(table.class_has_method(service, b"welcome"), "the alias adds a renamed copy of the trait method");
    });
}

#[test]
fn applies_trait_insteadof_adaptation() {
    use mago_oracle::symbol::class_like::ClassLikeSymbol;

    let sources = [(
        Origin::Project,
        "<?php
        trait Alpha { public function run(): void {} }
        trait Beta { public function run(): void {} }
        class Runner {
            use Alpha, Beta { Alpha::run insteadof Beta; }
        }
        ",
    )];

    with_link(&sources, |table| {
        let runner = SymbolId::class_like(b"Runner");
        let ClassLikeSymbol::Class(class) = table.get_class_like(runner).expect("Runner is linked") else {
            panic!("Runner is a class");
        };

        let run = class
            .methods
            .members
            .iter()
            .find(|member| member.name.id == SymbolId::method(b"Runner", b"run"))
            .expect("Runner has run");
        assert_eq!(
            run.defining_symbol,
            SymbolId::class_like(b"Alpha"),
            "insteadof keeps Alpha::run and discards Beta::run",
        );
    });
}

#[test]
fn flattens_transitive_interfaces() {
    use mago_oracle::symbol::class_like::ClassLikeSymbol;

    let sources = [(
        Origin::Project,
        "<?php
        interface Top {}
        interface Middle extends Top {}
        class Concrete implements Middle {}
        ",
    )];

    with_link(&sources, |table| {
        let concrete = SymbolId::class_like(b"Concrete");
        let top = SymbolId::class_like(b"Top");
        let middle = SymbolId::class_like(b"Middle");

        let ClassLikeSymbol::Class(class) = table.get_class_like(concrete).expect("Concrete is linked") else {
            panic!("Concrete is a class");
        };

        assert!(class.implements.contains(middle), "Concrete directly implements Middle");
        assert!(class.implements.contains(top), "Concrete transitively implements Top through Middle's extends edge",);
    });
}

#[test]
fn lowers_signature_detail_channels() {
    use mago_oracle::symbol::class_like::ClassLikeSymbol;
    use mago_oracle::symbol::function_like::FunctionLikeSymbol;

    let sources = [(
        Origin::Project,
        "<?php
        /**
         * @template T
         * @param T $value
         * @param-out int $ref
         * @return T
         * @throws RuntimeException
         * @assert-if-true int $value
         */
        function identity(mixed $value, mixed &$ref): mixed { return $value; }

        enum Suit: string {
            case Hearts = 'H';
            case Spades = 'S';
        }

        /**
         * @template TItem
         */
        class Collection {
            /**
             * @template TMapped
             * @throws LogicException
             */
            public function map(callable $fn): void {}
        }
        ",
    )];

    with_link(&sources, |table| {
        let FunctionLikeSymbol::Function(function) =
            table.get_function_like(SymbolId::function_like(b"identity")).expect("identity is linked")
        else {
            panic!("identity is a function");
        };
        assert_eq!(function.generics.len(), 1, "@template T lowered onto the function");
        assert_eq!(function.generics.first().map(|parameter| parameter.name), Some(&b"T"[..]));
        assert_eq!(function.throws.len(), 1, "@throws RuntimeException lowered onto the function");
        assert_eq!(function.assertions.len(), 1, "@assert-if-true lowered onto the function");
        assert!(
            function.assertions.iter().all(|assertion| assertion.is_if_true()),
            "the assertion is conditional on true"
        );
        assert!(
            function.params.iter().any(|parameter| parameter.out_ty.effective(true).is_some()),
            "@param-out int lowered into a parameter's out type",
        );

        let ClassLikeSymbol::Enum(suit) = table.get_class_like(SymbolId::class_like(b"Suit")).expect("Suit is linked")
        else {
            panic!("Suit is an enum");
        };
        let hearts = suit
            .cases
            .members
            .iter()
            .find(|case| case.name.id == SymbolId::enum_case(b"Suit", b"Hearts"))
            .expect("Hearts case is present");
        assert!(hearts.value.is_some(), "the backed enum case carries its literal value");

        let ClassLikeSymbol::Class(collection) =
            table.get_class_like(SymbolId::class_like(b"Collection")).expect("Collection is linked")
        else {
            panic!("Collection is a class");
        };
        let map = collection
            .methods
            .members
            .iter()
            .find(|method| method.name.id == SymbolId::method(b"Collection", b"map"))
            .expect("map method is present");
        assert_eq!(map.generics.len(), 1, "method-level @template TMapped lowered onto the method");
        assert_eq!(map.throws.len(), 1, "method @throws lowered");
    });
}

#[test]
fn lowers_inferred_and_effect_channels() {
    use mago_oracle::symbol::class_like::ClassLikeSymbol;
    use mago_oracle::symbol::function_like::FunctionLikeSymbol;

    let sources = [(
        Origin::Project,
        "<?php
        const ANSWER = 41 + 1;

        /**
         * @pure-unless-callable-is-impure $fn
         */
        function apply(callable $fn, int $x = 5): mixed { return $fn($x); }

        class Box {
            public int $value { get => 1; }

            /**
             * @self-out static
             */
            public function tap(): static { return $this; }
        }
        ",
    )];

    with_link(&sources, |table| {
        let answer = table.get_constant(SymbolId::constant(b"ANSWER")).expect("ANSWER is linked");
        assert!(answer.ty.effective(true).is_some(), "the global const value 41 + 1 is inferred");

        let FunctionLikeSymbol::Function(apply) =
            table.get_function_like(SymbolId::function_like(b"apply")).expect("apply is linked")
        else {
            panic!("apply is a function");
        };
        assert_eq!(apply.pure_unless_impure_params, &[0], "the $fn parameter (index 0) is pure-unless-callable-impure");
        assert!(
            apply.params.iter().any(|parameter| parameter.default_ty.effective(true).is_some()),
            "the int default value is inferred into the parameter's default type",
        );

        let ClassLikeSymbol::Class(box_class) =
            table.get_class_like(SymbolId::class_like(b"Box")).expect("Box is linked")
        else {
            panic!("Box is a class");
        };
        let value = box_class
            .properties
            .members
            .iter()
            .find(|property| property.name.id == SymbolId::property(b"Box", b"$value"))
            .expect("value property is present");
        assert!(!value.hooks.is_empty(), "the value property has a hook");
        assert!(
            value.hooks.iter().all(|hook| hook.ty.effective(true).is_some()),
            "the property hook carries the property type as its return type",
        );

        let tap = box_class
            .methods
            .members
            .iter()
            .find(|method| method.name.id == SymbolId::method(b"Box", b"tap"))
            .expect("tap method is present");
        assert!(tap.self_out.is_some(), "@self-out is recorded on the method");
    });
}

#[test]
fn tracks_namespaces() {
    let sources = [(Origin::Project, "<?php namespace App\\Model; class User {}")];

    with_link(&sources, |table| {
        assert!(table.contains_namespace(b"App"), "App namespace is recorded");
        assert!(table.contains_namespace(b"App\\Model"), "App\\Model namespace is recorded");
        assert!(
            table.get_class_like(SymbolId::class_like(b"App\\Model\\User")).is_some(),
            "the namespaced class links"
        );
    });
}
