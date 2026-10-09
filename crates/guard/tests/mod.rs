#![allow(clippy::unwrap_used, clippy::expect_used, clippy::missing_panics_doc)]

use std::borrow::Cow;
use std::sync::LazyLock;

use foldhash::HashSet;
use indoc::indoc;
use mago_allocator::LocalArena;

use mago_codex::populator::populate_codebase;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_guard::ArchitecturalGuard;
use mago_guard::path::NamespacePath;
use mago_guard::path::Path;
use mago_guard::path::SymbolSelector;
use mago_guard::report::FortressReport;
use mago_guard::report::breach::BreachReason;
use mago_guard::report::breach::BreachVector;
use mago_guard::report::flaw::FlawKind;
use mago_guard::settings::DependencyRestriction;
use mago_guard::settings::PerimeterRule;
use mago_guard::settings::PerimeterSettings;
use mago_guard::settings::PermittedDependency;
use mago_guard::settings::PermittedDependencyKind;
use mago_guard::settings::Settings;
use mago_guard::settings::StructuralRule;
use mago_guard::settings::StructuralSettings;
use mago_guard::settings::StructuralSymbolKind;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_reporting::Issue;
use mago_syntax::parser::parse_file;
use mago_word::WordSet;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);

/// Creates settings with a deny-all rule for the App\Module\ namespace.
/// This is needed because the guard now skips when there's no perimeter config.
fn deny_all_settings() -> Settings {
    Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![], // Deny everything
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn test_guard(name: &'static str, code: &'static str, settings: Settings) -> FortressReport {
    let Prelude { mut database, mut metadata, mut symbol_references } = PRELUDE.clone();

    let file = File::ephemeral(Cow::Borrowed(name.as_bytes()), Cow::Borrowed(code.as_bytes()));
    let file_id = database.add(file);
    let source_file = database.get_ref(&file_id).expect("File just added should exist");

    let arena = LocalArena::new();
    let program = parse_file(&arena, source_file);
    if program.has_errors() {
        panic!("Failed to parse code for guard test, errors: {:?}", program.errors);
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    metadata.extend(scan_program(&arena, source_file, program, &resolved_names, mago_php_version::PHPVersion::LATEST));

    populate_codebase(&mut metadata, &mut symbol_references, WordSet::default(), HashSet::default());

    let guard = ArchitecturalGuard::new(settings);
    guard.check(&metadata, program, &resolved_names)
}

#[test]
pub fn test_extends_violation() {
    let code = indoc! {r"
        <?php
        namespace App\Core {}
        namespace App\Module {
            class MyClass extends \App\Core\BaseClass {}
        }
    "};
    let settings = deny_all_settings();
    let result = test_guard("extends_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::Extends);
}

#[test]
pub fn test_implements_violation() {
    let code = indoc! {r"
        <?php

        namespace App\Core {}

        namespace App\Module {
            class MyClass implements \App\Core\MyInterface {}
        }
    "};

    let settings = deny_all_settings();
    let result = test_guard("implements_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::Implements);
}

// Test for UsageKind::ReturnType
#[test]
pub fn test_return_type_violation() {
    let code = indoc! {r"
        <?php

        namespace App\Core {}
        namespace App\Module {
            function my_function(): \App\Core\MyType {}
        }
    "};

    let settings = deny_all_settings();
    let result = test_guard("return_type_violation", code, settings);

    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::ReturnType);
}

#[test]
pub fn test_instantiation_violation() {
    let code = indoc! {r"
        <?php

        namespace App\Core {}

        namespace App\Module {
            new \App\Core\MyClass();
        }
    "};

    let settings = deny_all_settings();
    let result = test_guard("instantiation_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::Instantiation);
}

#[test]
pub fn test_static_method_call_violation() {
    let code = indoc! {r"
        <?php

        namespace App\Core { class Helper { public static function do() {} } }

        namespace App\Module {
            \App\Core\Helper::do();
        }
    "};

    let settings = deny_all_settings();
    let result = test_guard("static_method_call_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::StaticMethodCall);
}

#[test]
pub fn test_interface_dependency_violation() {
    let code = indoc! {r"
        <?php

        namespace App\Core { interface ServiceInterface {} }

        namespace App\Module {
            class MyService implements \App\Core\ServiceInterface {}
        }
    "};
    let settings = deny_all_settings();
    let result = test_guard("interface_dependency_violation", code, settings);

    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::ClassLike);
}

#[test]
pub fn test_trait_dependency_violation() {
    let code = indoc! {r"
        <?php

        namespace App\Core { trait MyTrait {} }

        namespace App\Module {
            class MyClass { use \App\Core\MyTrait; }
        }
    "};

    let settings = deny_all_settings();
    let result = test_guard("trait_dependency_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::ClassLike);
}

#[test]
pub fn test_enum_dependency_violation() {
    let code = indoc! {r"
        <?php

        namespace App\Core {
            enum MyEnum {}
        }

        namespace App\Module {
            function test(\App\Core\MyEnum $e) {}
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![],
                reason: None,
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
    let code = indoc! {r"
        <?php

        namespace App\Core {
            const MY_CONST = 1;
        }

        namespace App\Module {
            $a = \App\Core\MY_CONST;
        }
    "};

    let settings = deny_all_settings();
    let result = test_guard("const_dependency_violation", code, settings);

    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::Constant);
}

#[test]
pub fn test_native_type_is_allowed() {
    let code = indoc! {r"
        <?php

        namespace App\Module;

        use DateTime;
        use Exception;

        function test(DateTime $d): Exception {
            throw new Exception();
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Native)],
                reason: None,
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
    let code = indoc! {r"
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
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                    NamespacePath::Specific("App\\Domain\\".to_string()),
                )))],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("union_type_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 2, "Expected 2 violations for Core\\A");
    assert_eq!(result.boundary_breaches[0].dependency_fqn, b"App\\Core\\A".as_slice()); // `use`
    assert_eq!(result.boundary_breaches[1].dependency_fqn, b"App\\Core\\A".as_slice()); // parameter type
}

#[test]
pub fn test_intersection_type_violation() {
    let code = indoc! {r"
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
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                    NamespacePath::Specific("App\\Domain\\".to_string()),
                )))],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("intersection_type_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1, "Expected 1 violation for Core\\A");
    assert_eq!(result.boundary_breaches[0].dependency_fqn, b"App\\Core\\A".as_slice());
}

#[test]
pub fn test_multiple_allowed_types_rule() {
    let code = indoc! {r"
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
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::DependencyOfKind {
                    path: Path::Selector(SymbolSelector::Pattern("App\\Vendor\\**".to_string())),
                    kinds: vec![PermittedDependencyKind::ClassLike],
                }],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("multiple_allowed_types_rule", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1, "Expected 1 violation for some_function");
    assert_eq!(result.boundary_breaches[0].dependency_fqn, b"App\\Vendor\\some_function".as_slice());
    assert_eq!(result.boundary_breaches[0].dependency_kind, PermittedDependencyKind::Function);
}

#[test]
pub fn test_global_namespace_dependency_violation() {
    let code = indoc! {r"
        <?php

        namespace { class GlobalClass {} }

        namespace App\Module {
            function test(\GlobalClass $g) {}
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![],
                reason: None,
            }],
            ..Default::default()
        },

        ..Default::default()
    };
    let result = test_guard("global_namespace_dependency_violation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].dependency_fqn, b"GlobalClass".as_slice());
}

#[test]
pub fn test_ddd() {
    let code = indoc! {r"
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
    "};

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
                    reason: None,
                },
                PerimeterRule {
                    namespace: NamespacePath::Specific("CarthageSoftware\\Application\\".to_string()),
                    permit: vec![
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                        ))),
                        PermittedDependency::Dependency(Path::Native),
                    ],
                    reason: None,
                },
                PerimeterRule {
                    namespace: NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                    permit: vec![
                        PermittedDependency::Dependency(Path::Self_),
                        PermittedDependency::Dependency(Path::Native),
                    ],
                    reason: None,
                },
                PerimeterRule {
                    namespace: NamespacePath::Specific("CarthageSoftware\\Domain\\".to_string()),
                    permit: vec![PermittedDependency::Dependency(Path::Native)],
                    reason: None,
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("test_ddd", code, settings);

    assert_eq!(result.boundary_breaches.len(), 0, "Expected no violations, found: {:#?}", result.boundary_breaches);
}

#[test]
pub fn test_narrow_rule_not_widened_by_broad_catchall() {
    let code = indoc! {r"
        <?php
        namespace App\Infrastructure { class Service {} }
        namespace App\Domain {
            use App\Infrastructure\Service;

            class Model {
                public function __construct() {
                    new Service();
                }
            }
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![
                PerimeterRule {
                    namespace: NamespacePath::Specific("App\\Domain\\".to_string()),
                    permit: vec![
                        PermittedDependency::Dependency(Path::Native),
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("App\\Domain\\".to_string()),
                        ))),
                    ],
                    reason: None,
                },
                PerimeterRule {
                    namespace: NamespacePath::Specific("App\\".to_string()),
                    permit: vec![
                        PermittedDependency::Dependency(Path::Native),
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("App\\Domain\\".to_string()),
                        ))),
                        PermittedDependency::Dependency(Path::Selector(SymbolSelector::Namespace(
                            NamespacePath::Specific("App\\Infrastructure\\".to_string()),
                        ))),
                    ],
                    reason: None,
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("narrow_rule_not_widened", code, settings);

    assert!(
        !result.boundary_breaches.is_empty(),
        "Expected violations: Domain should not be allowed to use Infrastructure"
    );
}

#[test]
pub fn test_self_permit_is_scoped_to_rule_namespace() {
    let code = indoc! {r"
        <?php

        namespace App\Infrastructure\Doctrine\Orm {
            class Entity {}
        }

        namespace App\Domain\Model {
            class BaseModel {}
            class LocalUser extends BaseModel {}

            use App\Infrastructure\Doctrine\Orm\Entity;

            class User extends Entity {}
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Domain\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Self_)],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("self_permit_is_scoped_to_rule_namespace", code, settings);

    assert_eq!(result.boundary_breaches.len(), 2);
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::Use);
    assert_eq!(result.boundary_breaches[1].vector, BreachVector::Extends);
}

#[test]
pub fn test_dependency_restriction_allows_only_configured_source_namespaces() {
    let code = indoc! {r"
        <?php

        namespace App\Http\Controllers {
            class Controller {}
            class AllowedController extends Controller {}
        }

        namespace App\Http\Controllers\Internal {
            class InternalController extends \App\Http\Controllers\Controller {}
        }

        namespace Vendor\Package {
            class BaseClass {}
        }

        namespace App\Services {
            class ForbiddenController extends \App\Http\Controllers\Controller {}
            class UnrestrictedClass extends \Vendor\Package\BaseClass {}
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            restrictions: vec![DependencyRestriction {
                dependency: SymbolSelector::Symbol("App\\Http\\Controllers\\Controller".to_string()),
                allow_from: vec!["App\\Http\\Controllers\\".to_string()],
                deny_from: vec!["App\\Http\\Controllers\\Internal\\".to_string()],
                kinds: vec![],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("dependency_restriction_allow_from", code, settings);

    assert_eq!(result.boundary_breaches.len(), 2);
    assert!(
        result.boundary_breaches.iter().all(|breach| breach.dependency_fqn == b"App\\Http\\Controllers\\Controller")
    );
    assert!(result.boundary_breaches.iter().all(|breach| breach.vector == BreachVector::Extends));
    assert!(
        result
            .boundary_breaches
            .iter()
            .all(|breach| matches!(&breach.reason, BreachReason::ForbiddenByRestriction { .. }))
    );
}

#[cfg(feature = "serde")]
#[test]
pub fn test_issue_2372_brace_patterns_in_perimeter_configuration() {
    let settings = toml::from_str(include_str!("cases/issue_2372/settings.toml")).unwrap();
    let result = test_guard("issue_2372", include_str!("cases/issue_2372/repro.php"), settings);
    let breaches: Vec<_> = result
        .boundary_breaches
        .iter()
        .map(|breach| (breach.source_namespace.as_slice(), breach.dependency_fqn.as_slice()))
        .collect();

    assert_eq!(
        breaches,
        [
            (b"App\\LayerConsumer".as_slice(), b"App\\Baz\\Thing".as_slice()),
            (b"App\\PermitConsumer".as_slice(), b"App\\Baz\\Thing".as_slice()),
            (b"App\\TypedPermitConsumer".as_slice(), b"App\\Baz\\Thing".as_slice()),
            (b"App\\RestrictedConsumer".as_slice(), b"App\\Foo\\Thing".as_slice()),
            (b"App\\RestrictedConsumer".as_slice(), b"App\\Bar\\Thing".as_slice()),
        ]
    );
}

#[test]
pub fn test_dependency_restriction_takes_precedence_over_permits() {
    let code = indoc! {r"
        <?php

        namespace Illuminate\Foundation\Bus {
            trait Dispatchable {}
            function dispatch(): void {}
        }

        namespace Vendor\Package {
            trait AllowedTrait {}
            function allowed(): void {}
        }

        namespace App\Jobs {
            class ForbiddenJob {
                use \Illuminate\Foundation\Bus\Dispatchable;
            }

            class AllowedJob {
                use \Vendor\Package\AllowedTrait;
            }

            function run(): void {
                \Illuminate\Foundation\Bus\dispatch();
                \Vendor\Package\allowed();
            }
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::All)],
                reason: None,
            }],
            restrictions: vec![
                DependencyRestriction {
                    dependency: SymbolSelector::Symbol("Illuminate\\Foundation\\Bus\\Dispatchable".to_string()),
                    allow_from: vec![],
                    deny_from: vec!["App\\".to_string()],
                    kinds: vec![PermittedDependencyKind::ClassLike],
                    reason: None,
                },
                DependencyRestriction {
                    dependency: SymbolSelector::Symbol("Illuminate\\Foundation\\Bus\\dispatch".to_string()),
                    allow_from: vec![],
                    deny_from: vec!["App\\".to_string()],
                    kinds: vec![PermittedDependencyKind::Function],
                    reason: None,
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("dependency_restriction_deny_from", code, settings);

    assert_eq!(result.boundary_breaches.len(), 2);
    assert!(result.boundary_breaches.iter().any(|breach| {
        breach.dependency_fqn == b"Illuminate\\Foundation\\Bus\\Dispatchable" && breach.vector == BreachVector::TraitUse
    }));
    assert!(result.boundary_breaches.iter().any(|breach| {
        breach.dependency_fqn == b"Illuminate\\Foundation\\Bus\\dispatch" && breach.vector == BreachVector::FunctionCall
    }));
    assert!(
        result
            .boundary_breaches
            .iter()
            .all(|breach| matches!(&breach.reason, BreachReason::ForbiddenByRestriction { .. }))
    );
}

#[test]
pub fn test_only_public_methods_restricts_declared_class_api() {
    let code = indoc! {r"
        <?php

        namespace App\Http\Controllers {
            class InvokableController {
                public function __construct() {}
                public function __invoke() {}
                public function helper() {}
                function implicitPublic() {}
                protected function validate() {}
                private function normalize() {}
            }
        }

        namespace App\Services {
            class UnrestrictedService {
                public function helper() {}
            }
        }
    "};

    let settings = Settings {
        structural: StructuralSettings {
            rules: vec![StructuralRule {
                on: "App\\Http\\Controllers\\**".to_string(),
                target: Some(StructuralSymbolKind::Class),
                only_public_methods: Some(vec!["__construct".to_string(), "__INVOKE".to_string()]),
                ..Default::default()
            }],
        },
        ..Default::default()
    };

    let result = test_guard("only_public_methods", code, settings);

    assert_eq!(result.structural_flaws.len(), 2);
    assert!(
        result.structural_flaws.iter().any(|flaw| {
            matches!(&flaw.kind, FlawKind::PublicMethodNotAllowed { method, .. } if method == b"helper")
        })
    );
    assert!(result.structural_flaws.iter().any(|flaw| {
        matches!(&flaw.kind, FlawKind::PublicMethodNotAllowed { method, .. } if method == b"implicitPublic")
    }));
    assert!(result.structural_flaws.iter().all(|flaw| flaw.kind.error_code() == "only-public-methods"));
}

#[test]
pub fn test_is_final_annotation_counted() {
    let code = indoc! {r"
        <?php
            namespace App\Entity {
                /** @final */
                class Test {

                }
            }
    "};

    let settings = Settings {
        structural: StructuralSettings {
            rules: vec![StructuralRule {
                on: "**\\Entity\\*".into(),
                target: StructuralSymbolKind::Class.into(),
                must_be_final: Some(true),
                ..Default::default()
            }],
        },
        ..Default::default()
    };

    let result = test_guard("count_final_annotation_as_real_final_class", code, settings);
    assert!(
        result.structural_flaws.is_empty(),
        "Expected non violations: Should allow declare final class with annotation"
    );
}

#[test]
pub fn test_clone_exit_die_are_not_namespaced_function_dependencies() {
    let code = indoc! {r"
        <?php

        namespace App\Module {
            function with_value(object $value): object {
                return clone($value, ['value' => 'x']);
            }

            exit('done');
            die('done');
            custom_function();
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Native)],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = test_guard("clone_exit_die_are_not_namespaced_function_dependencies", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1, "found: {:#?}", result.boundary_breaches);
    assert_eq!(result.boundary_breaches[0].dependency_fqn, b"App\\Module\\custom_function".as_slice());
    assert_eq!(result.boundary_breaches[0].vector, BreachVector::FunctionCall);
}

#[test]
pub fn test_restriction_reason_is_attached_to_issue() {
    let code = indoc! {r"
        <?php

        namespace App\Http\Controllers {
            class Controller {}
        }

        namespace App\Services {
            class ForbiddenController extends \App\Http\Controllers\Controller {}
        }
    "};

    let reason = "Controllers must stay in the HTTP layer.".to_string();
    let settings = Settings {
        perimeter: PerimeterSettings {
            restrictions: vec![DependencyRestriction {
                dependency: SymbolSelector::Symbol("App\\Http\\Controllers\\Controller".to_string()),
                allow_from: vec!["App\\Http\\Controllers\\".to_string()],
                deny_from: vec![],
                kinds: vec![],
                reason: Some(reason.clone()),
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let mut result = test_guard("restriction_reason_is_attached_to_issue", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    let breach = result.boundary_breaches.remove(0);
    assert!(matches!(&breach.reason, BreachReason::ForbiddenByRestriction { .. }));
    assert_eq!(breach.explanation.as_deref(), Some(reason.as_str()));

    let issue = Issue::from(breach);
    assert!(issue.notes.iter().any(|note| note == &reason));
    assert!(issue.notes.iter().any(|note| note == "Dependency forbidden by an architectural restriction"));
}

#[test]
pub fn test_restriction_without_reason_has_no_explanation() {
    let code = indoc! {r"
        <?php

        namespace App\Http\Controllers {
            class Controller {}
        }

        namespace App\Services {
            class ForbiddenController extends \App\Http\Controllers\Controller {}
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            restrictions: vec![DependencyRestriction {
                dependency: SymbolSelector::Symbol("App\\Http\\Controllers\\Controller".to_string()),
                allow_from: vec!["App\\Http\\Controllers\\".to_string()],
                deny_from: vec![],
                kinds: vec![],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("restriction_without_reason_has_no_explanation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].explanation, None);
}

#[test]
pub fn test_rule_reason_is_attached_to_issue() {
    let code = indoc! {r"
        <?php

        namespace App\Core { class Helper {} }

        namespace App\Module {
            new \App\Core\Helper();
        }
    "};

    let reason = "Module code may only use PHP built-ins.".to_string();
    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Native)],
                reason: Some(reason.clone()),
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let mut result = test_guard("rule_reason_is_attached_to_issue", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    let breach = result.boundary_breaches.remove(0);
    assert!(matches!(&breach.reason, BreachReason::ForbiddenByRule { .. }));
    assert_eq!(breach.explanation.as_deref(), Some(reason.as_str()));

    let issue = Issue::from(breach);
    assert!(issue.notes.iter().any(|note| note == &reason));
    assert!(issue.notes.iter().any(|note| note == "Dependency forbidden by architectural rules"));
}

#[test]
pub fn test_rule_without_reason_has_no_explanation() {
    let code = indoc! {r"
        <?php

        namespace App\Core { class Helper {} }

        namespace App\Module {
            new \App\Core\Helper();
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Native)],
                reason: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("rule_without_reason_has_no_explanation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert_eq!(result.boundary_breaches[0].explanation, None);
}

#[test]
pub fn test_layering_reason_is_attached_to_issue() {
    let code = indoc! {r"
        <?php

        namespace App\Outer { class Helper {} }

        namespace App\Core {
            new \App\Outer\Helper();
        }
    "};

    let reason = "Inner layers must not depend on outer layers.".to_string();
    let settings = Settings {
        perimeter: PerimeterSettings {
            layering: vec![
                NamespacePath::Specific("App\\Core\\".to_string()),
                NamespacePath::Specific("App\\Outer\\".to_string()),
            ],
            layering_reason: Some(reason.clone()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut result = test_guard("layering_reason_is_attached_to_issue", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    let breach = result.boundary_breaches.remove(0);
    assert!(matches!(&breach.reason, BreachReason::Layering { .. }));
    assert_eq!(breach.explanation.as_deref(), Some(reason.as_str()));

    let issue = Issue::from(breach);
    assert!(issue.notes.iter().any(|note| note == &reason));
    assert!(issue.notes.iter().any(|note| note == "Layering Rule Conflict"));
}

#[test]
pub fn test_layering_without_reason_has_no_explanation() {
    let code = indoc! {r"
        <?php

        namespace App\Outer { class Helper {} }

        namespace App\Core {
            new \App\Outer\Helper();
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            layering: vec![
                NamespacePath::Specific("App\\Core\\".to_string()),
                NamespacePath::Specific("App\\Outer\\".to_string()),
            ],
            layering_reason: None,
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("layering_without_reason_has_no_explanation", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    assert!(matches!(&result.boundary_breaches[0].reason, BreachReason::Layering { .. }));
    assert_eq!(result.boundary_breaches[0].explanation, None);
}

#[test]
pub fn test_allowed_dependency_ignores_rule_reason() {
    let code = indoc! {r"
        <?php

        namespace App\Module;

        use DateTime;

        function test(DateTime $d): void {}
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![PermittedDependency::Dependency(Path::Native)],
                reason: Some("unused when permitted".to_string()),
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = test_guard("allowed_dependency_ignores_rule_reason", code, settings);
    assert!(result.is_empty(), "found: {:#?}", result.boundary_breaches);
}

#[test]
pub fn test_whitespace_reason_adds_no_note() {
    let code = indoc! {r"
        <?php

        namespace App\Core { class Helper {} }

        namespace App\Module {
            new \App\Core\Helper();
        }
    "};

    let settings = Settings {
        perimeter: PerimeterSettings {
            rules: vec![PerimeterRule {
                namespace: NamespacePath::Specific("App\\Module\\".to_string()),
                permit: vec![],
                reason: Some("   \t  ".to_string()),
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let mut result = test_guard("whitespace_reason_adds_no_note", code, settings);
    assert_eq!(result.boundary_breaches.len(), 1);
    let issue = Issue::from(result.boundary_breaches.remove(0));
    assert!(!issue.notes.iter().any(|note| note.trim().is_empty()));
    assert!(issue.notes.iter().any(|note| note == "Dependency forbidden by architectural rules"));
}

#[cfg(feature = "serde")]
#[test]
pub fn test_perimeter_reason_is_optional_in_toml() {
    let without_reason = r#"
[perimeter]
layering = ["App\\Core\\", "App\\Outer\\"]

[[perimeter.rules]]
namespace = "App\\Module\\"
permit = ["@native"]

[[perimeter.restrictions]]
dependency = "App\\Http\\Controllers\\Controller"
deny-from = ["App\\"]
"#;

    let with_reason = r#"
[perimeter]
layering = ["App\\Core\\", "App\\Outer\\"]
layering-reason = "Inner layers must not depend on outer layers."

[[perimeter.rules]]
namespace = "App\\Module\\"
permit = ["@native"]
reason = "Module code may only use PHP built-ins."

[[perimeter.restrictions]]
dependency = "App\\Http\\Controllers\\Controller"
deny-from = ["App\\"]
reason = "Controllers must stay in the HTTP layer."
"#;

    let settings_without: Settings = toml::from_str(without_reason).unwrap();
    assert_eq!(settings_without.perimeter.layering_reason, None);
    assert_eq!(settings_without.perimeter.rules[0].reason, None);
    assert_eq!(settings_without.perimeter.restrictions[0].reason, None);

    let settings_with: Settings = toml::from_str(with_reason).unwrap();
    assert_eq!(
        settings_with.perimeter.layering_reason.as_deref(),
        Some("Inner layers must not depend on outer layers.")
    );
    assert_eq!(settings_with.perimeter.rules[0].reason.as_deref(), Some("Module code may only use PHP built-ins."));
    assert_eq!(
        settings_with.perimeter.restrictions[0].reason.as_deref(),
        Some("Controllers must stay in the HTTP layer.")
    );
}
