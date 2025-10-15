use std::borrow::Cow;
use std::sync::LazyLock;

use ahash::HashSet;
use bumpalo::Bump;
use indoc::indoc;

use mago_atom::AtomSet;
use mago_codex::populator::populate_codebase;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_guard::ArchitecturalGuard;
use mago_guard::path::NamespacePath;
use mago_guard::path::Path;
use mago_guard::path::SymbolSelector;
use mago_guard::report::FortressReport;
use mago_guard::report::breach::BreachVector;
use mago_guard::settings::PerimeterRule;
use mago_guard::settings::PerimeterSettings;
use mago_guard::settings::PermittedDependency;
use mago_guard::settings::PermittedDependencyKind;
use mago_guard::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_syntax::parser::parse_file;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);

fn test_guard(name: &'static str, code: &'static str, settings: Settings) -> FortressReport {
    let Prelude { mut database, mut metadata, mut symbol_references } = PRELUDE.clone();

    let file = File::ephemeral(Cow::Borrowed(name), Cow::Borrowed(code));
    let file_id = database.add(file);
    let source_file = database.get_ref(&file_id).expect("File just added should exist");

    let arena = Bump::new();
    let (program, parse_issues) = parse_file(&arena, source_file);
    if parse_issues.is_some() {
        panic!("Test '{}' failed during parsing:\n{:#?}", name, parse_issues);
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    metadata.extend(scan_program(&arena, source_file, program, &resolved_names));

    populate_codebase(&mut metadata, &mut symbol_references, AtomSet::default(), HashSet::default());

    let guard = ArchitecturalGuard::new(settings);
    guard.check(&metadata, program, &resolved_names)
}

#[test]
pub fn test_extends_violation() {
    let code = indoc! {r#"
        <?php
        namespace App\Core {}
        namespace App\Module {
            class MyClass extends \App\Core\BaseClass {}
        }
    "#};
    let settings = Settings::default(); // Deny by default
    let result = test_guard("extends_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::Extends);
}

#[test]
pub fn test_implements_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core {}

        namespace App\Module {
            class MyClass implements \App\Core\MyInterface {}
        }
    "#};

    let settings = Settings::default();
    let result = test_guard("implements_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::Implements);
}

// Test for UsageKind::ReturnType
#[test]
pub fn test_return_type_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core {}
        namespace App\Module {
            function my_function(): \App\Core\MyType {}
        }
    "#};

    let settings = Settings::default();
    let result = test_guard("return_type_violation", code, settings);

    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::ReturnType);
}

#[test]
pub fn test_instantiation_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core {}

        namespace App\Module {
            new \App\Core\MyClass();
        }
    "#};

    let settings = Settings::default();
    let result = test_guard("instantiation_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::Instantiation);
}

#[test]
pub fn test_static_method_call_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core { class Helper { public static function do() {} } }

        namespace App\Module {
            \App\Core\Helper::do();
        }
    "#};

    let settings = Settings::default();
    let result = test_guard("static_method_call_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::StaticMethodCall);
}

#[test]
pub fn test_interface_dependency_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core { interface ServiceInterface {} }

        namespace App\Module {
            class MyService implements \App\Core\ServiceInterface {}
        }
    "#};
    let settings = Settings::default();
    let result = test_guard("interface_dependency_violation", code, settings);

    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::ClassLike);
}

#[test]
pub fn test_trait_dependency_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core { trait MyTrait {} }

        namespace App\Module {
            class MyClass { use \App\Core\MyTrait; }
        }
    "#};

    let settings = Settings::default();
    let result = test_guard("trait_dependency_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::ClassLike);
}

#[test]
pub fn test_enum_dependency_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core {
            enum MyEnum {}
        }

        namespace App\Module {
            function test(\App\Core\MyEnum $e) {}
        }
    "#};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![],
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("enum_dependency_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);

    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::ClassLike);
}

#[test]
pub fn test_const_dependency_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core {
            const MY_CONST = 1;
        }

        namespace App\Module {
            $a = \App\Core\MY_CONST;
        }
    "#};

    let settings = Settings::default();
    let result = test_guard("const_dependency_violation", code, settings);

    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::Constant);
}

#[test]
pub fn test_native_type_is_allowed() {
    let code = indoc! {r#"
        <?php

        namespace App\Module;

        use DateTime;
        use Exception;

        function test(DateTime $d): Exception {
            throw new Exception();
        }
    "#};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Native)],
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("native_type_is_allowed", code, settings);
    assert!(result.is_empty(), "Expected no violations for native types, found: {:#?}", result.boundary_breaches);
}

#[test]
pub fn test_union_type_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core {
            class A {}
        }

        namespace App\Domain {
            class B {}
        }

        namespace App\Module {
            use App\Core\A;
            use App\Domain\B;

            function test(A|B $ab) {
            }
        }
    "#};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                    NamespacePath::Specific("App\\Domain\\".to_string()),
                )))],
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("union_type_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 2, "Expected 2 violations for Core\\A");
    assert_eq!(result.boundary_breaches[0].dependency_fqn, "App\\Core\\A"); // `use`
    assert_eq!(result.boundary_breaches[1].dependency_fqn, "App\\Core\\A"); // parameter type
}

#[test]
pub fn test_intersection_type_violation() {
    let code = indoc! {r#"
        <?php

        namespace App\Core {
            interface A {}
        }

        namespace App\Domain {
            interface B {}
        }

        namespace App\Module {
            use App\Domain\B;

            function test(\App\Core\A&B $ab) {}
        }
    "#};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                    NamespacePath::Specific("App\\Domain\\".to_string()),
                )))],
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("intersection_type_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1, "Expected 1 violation for Core\\A");
    assert_eq!(result.boundary_breaches[0].dependency_fqn, "App\\Core\\A");
}

#[test]
pub fn test_multiple_allowed_types_rule() {
    let code = indoc! {r#"
        <?php

        namespace App\Vendor {
            class MyClass {}
            interface MyInterface {}
            trait MyTrait {}
        }

        namespace App\Module {
            use App\Vendor\MyClass;
            use App\Vendor\MyInterface;

            class Test implements MyInterface {
                public function create(): MyClass {
                    \App\Vendor\some_function(...);

                    return new MyClass();
                }
            }
        }
    "#};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::DependencyOfKind {
                    path: Path::Selector(SymbolSelector::Pattern("App\\Vendor\\**".to_string())),
                    kinds: vec![PermittedDependencyKind::ClassLike],
                }],
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("multiple_allowed_types_rule", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1, "Expected 1 violation for some_function");
    assert_eq!(result.boundary_breaches[0].dependency_fqn, "App\\Vendor\\some_function");
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::Function);
}

#[test]
pub fn test_global_namespace_dependency_violation() {
    let code = indoc! {r#"
        <?php

        namespace { class GlobalClass {} }

        namespace App\Module {
            function test(\GlobalClass $g) {}
        }
    "#};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![],
            }],
            ..Default::default()
        },

        ..Default::default()
    };
    let result = test_guard("global_namespace_dependency_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_fqn, "GlobalClass");
}

#[test]
pub fn test_ddd() {
    let code = indoc! {r#"
        <?php

        namespace Symfony\Component\HttpFoundation {
            class Request {}
            class Response {}
        }

        namespace CarthageSoftware\Domain\Shared\Repository {
            interface RepositoryInterface {
                public function getOne(int $id): ?object;
            }
        }

        namespace CarthageSoftware\Domain\Blogging\Entity {
            class Post {}
        }

        namespace CarthageSoftware\Domain\Blogging\Repository {
            use CarthageSoftware\Domain\Blogging\Entity\Post;
            use CarthageSoftware\Domain\Shared\Repository\RepositoryInterface;

            interface PostRepositoryInterface extends RepositoryInterface {
                public function getOne(int $id): ?Post;
            }
        }

        namespace CarthageSoftware\Application\Blogging\Command {
            class CreatePostCommand {}
        }

        namespace CarthageSoftware\Application\Shared\Command {
            interface CommandBusInterface {
                public function dispatch(object $command): void;
            }
        }

        namespace CarthageSoftware\UI\Blogging\Web\Controller {
            use CarthageSoftware\Application\Blogging\Command\CreatePostCommand;
            use CarthageSoftware\Application\Shared\Command\CommandBusInterface;
            use CarthageSoftware\Domain\Blogging\Repository\PostRepositoryInterface;
            use CarthageSoftware\Domain\Blogging\Entity\Post;
            use Symfony\Component\HttpFoundation\Request;
            use Symfony\Component\HttpFoundation\Response;

            class PostController {
                public function __construct(private CommandBusInterface $commandBus) {}

                public function create(Request $request): Response {
                    $command = new CreatePostCommand();
                    $this->commandBus->dispatch($command);

                    return new Response();
                }
            }

            class ShowController {
                public function __construct(private PostRepositoryInterface $postRepository) {}

                public function show(int $id): ?Post {
                    return $this->postRepository->getOne($id);
                }
            }
        }
    "#};

    let settings = Settings {
        perimeter: PerimeterSettings {
            layering: vec![
                NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                NamespacePath::Specific("CarthageSoftware\\Application\\".to_string()),
                NamespacePath::Specific("CarthageSoftware\\UI\\".to_string()),
                NamespacePath::Specific("CarthageSoftware\\Infrastructure\\".to_string()),
            ],
            rules: vec![
                PerimeterRule {
                    namespace: NamespacePath::Specific("CarthageSoftware\\UI\\".to_string()),
                    permit: vec![
                        PermittedDependency::Dependency(Path::Native),
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                        ))),
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("CarthageSoftware\\Application\\".to_string()),
                        ))),
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("Symfony\\Component\\HttpFoundation\\".to_string()),
                        ))),
                    ],
                },
                PerimeterRule {
                    namespace: NamespacePath::Specific("CarthageSoftware\\Application\\".to_string()),
                    permit: vec![
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                        ))),
                        PermittedDependency::Dependency(Path::Native),
                    ],
                },
                PerimeterRule {
                    namespace: NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                    permit: vec![
                        PermittedDependency::Dependency(Path::Self_),
                        PermittedDependency::Dependency(Path::Native),
                    ],
                },
                PerimeterRule {
                    namespace: NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                    permit: vec![PermittedDependency::Dependency(Path::Native)],
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("test_ddd", code, settings);

    assert_eq!(result.boundary_breaches.len(), 0, "Expected no violations, found: {:#?}", result.boundary_breaches);
}
