use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::AtomMap;
use mago_atom::atom;

use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_object;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::standin_type_replacer::get_most_specific_type_from_bounds;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::MethodTargetContext;
use crate::invocation::analyzer::analyze_invocation;
use crate::invocation::post_process::post_invocation_process;
use crate::resolver::class_name::ResolvedClassname;
use crate::resolver::class_name::resolve_classnames_from_expression;
use crate::utils::template::get_generic_parameter_for_offset;
use crate::visibility::check_method_visibility;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Instantiation<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let classnames = resolve_classnames_from_expression(context, block_context, artifacts, self.class, false)?;
        if classnames.is_empty() {
            return Ok(());
        }

        if classnames.len() > 1 {
            let possible_class_names_str = classnames
                .iter()
                .map(|classname| classname.fqcn.map(|id| id.as_str()).unwrap_or("<unknown>"))
                .collect::<Vec<_>>()
                .join(", ");

            let class_expression_type_str =
                artifacts.get_expression_type(&self.class).map_or("<unknown>", |u| u.get_id().as_str());

            context.collector.report_with_code(
                IssueCode::AmbiguousInstantiationTarget,
                Issue::warning("Ambiguous instantiation: the expression used with `new` can resolve to multiple different classes.".to_string())
                .with_annotation(
                    Annotation::primary(self.class.span())
                        .with_message(format!(
                            "This expression (type `{class_expression_type_str}`) can instantiate one of: [{possible_class_names_str}]"
                        )),
                )
                .with_note(
                    "Instantiating from an expression with a union of different class types is a risky practice."
                )
                .with_note(
                    "The resolved classes may have different constructor signatures, distinct type parameters, or incompatible behaviors, leading to potential runtime errors or unexpected outcomes."
                )
                .with_help(
                    "To ensure type safety and predictability, refine the type of the expression used with `new` to a single specific `class-string<T>` or use conditional logic to instantiate explicitly based on the desired class.",
                ),
            );
        }

        let mut resulting_type = None;
        for classname in classnames {
            let instantiation_span = self.span();
            let class_expression_span = self.class.span();

            let argument_list = if let Some(arg_list) = &self.argument_list { Some(arg_list) } else { None };

            let type_candidate = analyze_class_instantiation(
                context,
                block_context,
                artifacts,
                classname,
                instantiation_span,
                class_expression_span,
                argument_list,
            )?;

            resulting_type = Some(add_optional_union_type(type_candidate, resulting_type.as_ref(), context.codebase));
        }

        if let Some(resulting_type) = resulting_type {
            artifacts.set_expression_type(self, resulting_type);
        } else {
            artifacts.set_expression_type(self, get_object()); // Fallback to object if no valid instantiation was found
        }

        Ok(())
    }
}

fn analyze_class_instantiation<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    classname: ResolvedClassname,
    instantiation_span: Span,
    class_expression_span: Span,
    argument_list: Option<&ArgumentList<'arena>>,
) -> Result<TUnion, AnalysisError> {
    if classname.is_invalid() {
        argument_list.analyze(context, block_context, artifacts)?;

        return Ok(get_never());
    }

    let Some(fq_classname) = classname.fqcn else {
        context.collector.report_with_code(
            IssueCode::UnknownClassInstantiation,
            Issue::error("Cannot determine the concrete class for instantiation.")
                .with_annotation(Annotation::primary(class_expression_span).with_message("This expression resolves to an unknown or non-specific class type"))
                .with_note("This can happen if instantiating from a variable with a general type like `object`, `class-string` (without a specific class), or `mixed`.")
                .with_note("Without a known class, constructor arguments and type parameters cannot be validated accurately.")
                .with_help("Use a more specific type hint for the variable (e.g., `class-string<MyClass>`, `MyClass`), or ensure it always holds a known instantiable class name."),
        );

        argument_list.analyze(context, block_context, artifacts)?;

        return Ok(get_object());
    };

    let Some(metadata) = context.codebase.get_class_like(&fq_classname) else {
        context.collector.report_with_code(
            IssueCode::NonExistentClass,
            Issue::error(format!("Class `{fq_classname}` not found."))
            .with_annotation(
                Annotation::primary(class_expression_span)
                    .with_message(format!("`{fq_classname}` is not defined or cannot be autoloaded")),
            )
            .with_help(
                "Ensure the name is correct, including its namespace, and that it's properly defined and autoloadable.",
            ),
        );

        argument_list.analyze(context, block_context, artifacts)?;

        return Ok(get_never());
    };

    let classname_str = &metadata.original_name;
    if metadata.kind.is_interface() {
        context.collector.report_with_code(
             IssueCode::InterfaceInstantiation,
             Issue::error(format!("Interface `{classname_str}` cannot be instantiated with `new`."))
                 .with_annotation(
                     Annotation::primary(class_expression_span)
                         .with_message("Attempting to instantiate an interface"),
                 )
                 .with_note("Interfaces are contracts and cannot be directly instantiated. You need to instantiate a class that implements the interface.")
                 .with_help(format!("Instantiate a concrete class that implements `{classname_str}` instead.")),
         );

        argument_list.analyze(context, block_context, artifacts)?;

        return Ok(get_never());
    } else if metadata.kind.is_trait() {
        context.collector.report_with_code(
            IssueCode::TraitInstantiation,
            Issue::error(format!("Trait `{classname_str}` cannot be instantiated with `new`."))
                .with_annotation(
                    Annotation::primary(class_expression_span).with_message("Attempting to instantiate a trait"),
                )
                .with_note("Traits are designed for code reuse and cannot be instantiated directly.")
                .with_help(format!(
                    "Use the trait `{classname_str}` within a class definition using the `use` keyword."
                )),
        );

        argument_list.analyze(context, block_context, artifacts)?;

        return Ok(get_never());
    } else if metadata.kind.is_enum() {
        context.collector.report_with_code(
            IssueCode::EnumInstantiation,
            Issue::error(format!("Enum `{classname_str}` cannot be instantiated with `new`."))
                .with_annotation(
                    Annotation::primary(class_expression_span)
                        .with_message("Attempting to instantiate an enum with `new`"),
                )
                .with_note("Enum instances are created by accessing their cases directly (e.g., `MyEnum::CaseName`).")
                .with_help(format!(
                    "Use `{classname_str}::CASE_NAME` to get an enum case instance, or `{classname_str}::cases()` to get all cases."
                )),
        );

        argument_list.analyze(context, block_context, artifacts)?;

        return Ok(get_never());
    }

    let mut is_impossible = false;
    if metadata.flags.is_abstract() && !classname.can_extend_static() {
        context.collector.report_with_code(
            IssueCode::AbstractInstantiation,
            Issue::error(format!("Cannot instantiate abstract class `{classname_str}`."))
                .with_annotation(
                    Annotation::primary(class_expression_span)
                        .with_message("Attempting to instantiate an abstract class"),
                )
                .with_help(if classname.is_static() {
                    "Use `new static()` in a non-final child class, or instantiate a concrete subclass."
                } else {
                    "Instantiate a concrete subclass of this abstract class."
                }),
        );

        is_impossible = true;
    }

    if metadata.flags.is_deprecated()
        && block_context.scope.get_class_like_name().is_none_or(|self_id| *self_id != metadata.original_name)
    {
        context.collector.report_with_code(
            IssueCode::DeprecatedClass,
            Issue::warning(format!("Class `{classname_str}` is deprecated and should no longer be used."))
                .with_annotation(
                    Annotation::primary(class_expression_span).with_message("Instantiation of deprecated class"),
                )
                .with_help(
                    "Consult the documentation for this class to find its replacement or an alternative approach.",
                ),
        );
    }

    let mut type_parameters = None;

    let constructor_id = MethodIdentifier::new(atom(&metadata.original_name), atom("__construct"));
    let constructor_declraing_id = context.codebase.get_declaring_method_identifier(&constructor_id);

    artifacts.symbol_references.add_reference_for_method_call(&block_context.scope, &constructor_id);

    let mut has_inconsistent_constructor =
        !metadata.flags.is_final() && metadata.name_span.is_some() && !metadata.flags.has_consistent_constructor();
    let mut constructor_span = None;

    let mut template_result = TemplateResult::new(
        IndexMap::with_hasher(RandomState::default()),
        IndexMap::with_hasher(RandomState::default()),
    );

    let is_spl_object_storage = classname_str.eq_ignore_ascii_case("splobjectstorage");
    if let Some(constructor) = context.codebase.get_method_by_id(&constructor_declraing_id) {
        has_inconsistent_constructor =
            has_inconsistent_constructor && !constructor.method_metadata.as_ref().is_some_and(|meta| meta.is_final);
        constructor_span = Some(constructor.name_span.unwrap_or(constructor.span));

        artifacts.symbol_references.add_reference_for_method_call(&block_context.scope, &constructor_declraing_id);

        let constructor_call = Invocation {
            target: InvocationTarget::FunctionLike {
                identifier: FunctionLikeIdentifier::Method(
                    *constructor_declraing_id.get_class_name(),
                    *constructor_declraing_id.get_method_name(),
                ),
                metadata: constructor,
                inferred_return_type: None,
                method_context: Some(MethodTargetContext {
                    declaring_method_id: Some(constructor_declraing_id),
                    class_like_metadata: metadata,
                    class_type: StaticClassType::None,
                }),
                span: instantiation_span,
            },
            arguments_source: match argument_list.as_ref() {
                Some(arg_list) => InvocationArgumentsSource::ArgumentList(arg_list),
                None => InvocationArgumentsSource::None(instantiation_span),
            },
            span: instantiation_span,
        };

        let mut argument_types = AtomMap::default();
        analyze_invocation(
            context,
            block_context,
            artifacts,
            &constructor_call,
            Some((metadata.name, None)),
            &mut template_result,
            &mut argument_types,
        )?;

        post_invocation_process(
            context,
            block_context,
            artifacts,
            &constructor_call,
            None,
            &template_result,
            &argument_types,
            true,
        )?;

        if !check_method_visibility(
            context,
            block_context,
            constructor_declraing_id.get_class_name(),
            constructor_declraing_id.get_method_name(),
            instantiation_span,
            None,
        ) {
            is_impossible = true;
        }

        let mut resolved_template_types = vec![];
        for (template_name, base_type) in metadata.template_types.iter() {
            let template_type = if let Some(lower_bounds) =
                template_result.get_lower_bounds_for_class_like(template_name, &metadata.name)
            {
                get_most_specific_type_from_bounds(lower_bounds, context.codebase)
            } else if !metadata.template_extended_parameters.is_empty() && !template_result.lower_bounds.is_empty() {
                let found_generic_parameters = template_result
                    .lower_bounds
                    .iter()
                    .map(|(template_name, lower_bounds_map)| {
                        (
                            *template_name,
                            lower_bounds_map
                                .iter()
                                .map(|(generic_parent, lower_bounds)| {
                                    (
                                        *generic_parent,
                                        get_most_specific_type_from_bounds(lower_bounds, context.codebase),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<AtomMap<_>>();

                get_generic_parameter_for_offset(
                    &metadata.name,
                    template_name,
                    &metadata.template_extended_parameters,
                    &found_generic_parameters,
                )
            } else if is_spl_object_storage {
                get_never()
            } else {
                base_type.first().map(|(_, constraint)| constraint).cloned().unwrap_or_else(get_never)
            };

            resolved_template_types.push(template_type);
        }

        if !resolved_template_types.is_empty() {
            type_parameters = Some(resolved_template_types);
        }
    } else if let Some(argument_list) = &argument_list
        && !argument_list.arguments.is_empty()
    {
        context.collector.report_with_code(
            IssueCode::TooManyArguments,
            Issue::error(format!(
                "Class `{classname_str}` has no `__construct` method, but arguments were provided to `new`."
            ))
            .with_annotation(Annotation::primary(argument_list.span()).with_message("Arguments provided here"))
            .with_annotation(
                Annotation::secondary(class_expression_span)
                    .with_message(format!("For class `{classname_str}` which has no constructor")),
            )
            .with_help("Remove the arguments, or define a `__construct` method in the class if arguments are needed for initialization."),
        );

        argument_list.analyze(context, block_context, artifacts)?;
    } else if !metadata.template_types.is_empty() {
        type_parameters = Some(
            metadata
                .template_types
                .iter()
                .map(|(_, map)| {
                    if is_spl_object_storage {
                        get_never()
                    } else {
                        map.iter().next().map(|(_, i)| i).cloned().unwrap_or_else(get_never)
                    }
                })
                .collect(),
        );
    }

    if has_inconsistent_constructor
        && (classname.is_static() || classname.is_from_class_string() || classname.is_object_instance())
    {
        let mut issue = if classname.is_static() {
            Issue::warning(format!(
                "Unsafe `new static()`: constructor of `{classname_str}` is not final and its signature might change in child classes, potentially leading to runtime errors.",
            ))
            .with_annotation(Annotation::primary(class_expression_span).with_message("`new static()` used here"))
        } else if classname.is_from_class_string() {
            Issue::warning(format!(
                "Unsafe `new $class_name`: constructor of `{classname_str}` is not final and its signature might change in child classes, potentially leading to runtime errors.",
            ))
            .with_annotation(Annotation::primary(class_expression_span).with_message("`new $class_name()` used here"))
        } else {
            Issue::warning(format!(
                "Unsafe `new $object`: constructor of `{classname_str}` is not final and its signature might change in child classes, potentially leading to runtime errors.",
            ))
            .with_annotation(Annotation::primary(class_expression_span).with_message("`new $object` used here"))
        };

        if let Some(constructor_span) = constructor_span {
            issue = issue.with_annotation(
                Annotation::secondary(constructor_span)
                    .with_message("Constructor defined here could be overridden with an incompatible signature"),
            );
        }

        context.collector.report_with_code(
            IssueCode::UnsafeInstantiation,
            issue
                .with_help("Ensure constructor signature consistency across inheritance (e.g., using `@consistent-constructor` if applicable) or mark the class/constructor as final.")
        );
    }

    if classname.is_from_class_string() || classname.is_from_any_object() {
        let descendants = context.codebase.get_all_descendants(&metadata.name);

        for descendant_class in descendants {
            artifacts.symbol_references.add_reference_to_overridden_class_member(
                &block_context.scope,
                (descendant_class, *constructor_id.get_method_name()),
            );
        }
    }

    if is_impossible {
        return Ok(get_never());
    }

    let result_type = wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject {
        name: metadata.original_name,
        type_parameters,
        is_this: classname.is_static() || (classname.is_self() && metadata.flags.is_final()),
        intersection_types: None,
        remapped_parameters: false,
    })));

    Ok(result_type)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = templated_class_instantiation,
        code = indoc! {r#"
            <?php

            /**
             * @phpstan-template K as string|int
             * @phpstan-template V
             */
            class Collection
            {
                /**
                 * @var array<K, V>
                 */
                public $items = [];

                /**
                 * @param array<K, V> $items
                 */
                public function __construct(array $items = [])
                {
                    foreach ($items as $key => $value) {
                        $this->items[$key] = $value;
                    }
                }
            }

            /**
             * @param Collection<string, string> $collection
             *
             * @return Collection<string, string>
             */
            function i_take_string_collection(Collection $collection): Collection
            {
                return $collection;
            }

            $collection = new Collection(['name' => 'John Doe']);
            i_take_string_collection($collection); // ok

            $collection = new Collection(['age' => 30]);
            i_take_string_collection($collection); // error
        "#},
        issues = [
            IssueCode::InvalidArgument, // expected Collection<string, string>, got Collection<string, int>
        ],
    }

    test_analysis! {
        name = ambiguous_instantiation_target,
        code = indoc! {r#"
            <?php

            class A {}
            class B {}
            class C {}

            /**
             * @param A|class-string<B>|class-string<C> $instance
             */
            function foo(A|string $instance): A|B|C {
                $instance = new $instance;

                return $instance;
            }
        "#},
        issues = [
            IssueCode::AmbiguousInstantiationTarget, // `new $instance` could be A, B, C, or <unknown>
            IssueCode::UnsafeInstantiation, // `A` is not final
            IssueCode::UnsafeInstantiation, // `B` is not final
            IssueCode::UnsafeInstantiation, // `C` is not final
        ],
    }

    test_analysis! {
        name = instantiation_of_interface,
        code = indoc! {r#"
            <?php

            interface MyInterface {}

            $a = new MyInterface();
        "#},
        issues = [
            IssueCode::InterfaceInstantiation,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_of_trait,
        code = indoc! {r#"
            <?php

            trait MyTrait {}

            $a = new MyTrait();
        "#},
        issues = [
            IssueCode::TraitInstantiation,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_of_enum,
        code = indoc! {r#"
            <?php

            enum MyEnum {}

            $a = new MyEnum();
        "#},
        issues = [
            IssueCode::EnumInstantiation,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_of_abstract_class,
        code = indoc! {r#"
            <?php

            abstract class MyAbstractClass {}

            $a = new MyAbstractClass();
        "#},
        issues = [
            IssueCode::AbstractInstantiation,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_self_outside_class,
        code = indoc! {r#"
            <?php

            $a = new self();
        "#},
        issues = [
            IssueCode::SelfOutsideClassScope,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_static_outside_class,
        code = indoc! {r#"
            <?php

            $a = new static();
        "#},
        issues = [
            IssueCode::StaticOutsideClassScope,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_parent_outside_class,
        code = indoc! {r#"
            <?php

            $a = new parent();
        "#},
        issues = [
            IssueCode::ParentOutsideClassScope,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_of_undefined_class,
        code = indoc! {r#"
            <?php

            $a = new NonExistentClass();
        "#},
        issues = [
            IssueCode::NonExistentClass,
            IssueCode::ImpossibleAssignment, // $a becomes never
        ]
    }

    test_analysis! {
        name = instantiation_from_invalid_expression_type,
        code = indoc! {r#"
            <?php

            $className = 123; // Not a class string

            $a = new $className();
        "#},
        issues = [
            IssueCode::InvalidClassStringExpression,
            IssueCode::ImpossibleAssignment, // `$a` becomes never
        ]
    }

    test_analysis! {
        name = instantiation_from_general_string_variable,
        code = indoc! {r#"
            <?php

            /** @param string $className */
            function create_instance(string $className) {
                return new $className();
            }
        "#},
        issues = [
            IssueCode::UnknownClassInstantiation, // `new $className()` could be any object
        ]
    }

    test_analysis! {
        name = instantiation_from_mixed_variable,
        code = indoc! {r#"
            <?php
            /** @param mixed $className */
            function create_instance_mixed($className) {
                return new $className();
            }
        "#},
        issues = [
            IssueCode::UnknownClassInstantiation, // `new $className()` could be any object
        ]
    }

    test_analysis! {
        name = instantiation_too_many_args_no_constructor,
        code = indoc! {r#"
            <?php
            class NoConstructor {}
            $a = new NoConstructor(1, 2, 3);
        "#},
        issues = [IssueCode::TooManyArguments]
    }

    test_analysis! {
        name = instantiation_too_many_args_with_constructor,
        code = indoc! {r#"
            <?php
            class WithConstructor {
                public function __construct(int $a, int $b) {}
            }
            $a = new WithConstructor(1, 2, 3);
        "#},
        issues = [IssueCode::TooManyArguments]
    }

    test_analysis! {
        name = instantiation_with_child_constructor,
        code = indoc! {r#"
            <?php

            class Base {
                public function __construct(int $a) {}
            }

            class Child extends Base {
                public function __construct(string $b) {}
            }

            $a = new Child(1);
        "#},
        issues = [
            IssueCode::InvalidArgument,
        ]
    }

    test_analysis! {
        name = instantiation_with_parent_constructor,
        code = indoc! {r#"
            <?php

            class Base {
                public function __construct(int $a) {}
            }

            final class Child extends Base {
            }

            $a = new Child(1);
        "#}
    }

    test_analysis! {
        name = resolve_nested_type_parameters,
        code = indoc! {r#"
            <?php

            /**
             * @template-covariant T
             */
            final readonly class Box
            {
                /**
                 * @param T $value
                 */
                public function __construct(
                    private mixed $value,
                ) {}

                /**
                 * @return T
                 */
                public function get(): mixed {
                    return $this->value;
                }
            }

            /**
             * @return Box<Box<Box<42>>>
             */
            function get_box_of_box_of_box(): Box {
                return new Box(new Box(new Box(42)));
            }
        "#},
    }

    test_analysis! {
        name = handles_recursive_type,
        code = indoc! {r#"
            <?php

            /** @template T */
            final readonly class Example {
                /** @return Example<Example<T>> */
                public function return_something(): Example {
                    /** @var Example<Example<T>> */
                    return new Example();
                }
            }
        "#},
        issues = [
            // TODO(azjezz): we want to trigger an issue here in the future about
            // the type parameter not being used at all in the class `Example`
        ]
    }

    test_analysis! {
        name = self_is_static_in_final_class,
        code = indoc! {r#"
            <?php

            /**
             * @template Tk of array-key
             * @template Tv
             */
            final class Map {
                /**
                 * @var array<Tk, Tv> $elements
                 */
                private array $elements;

                /**
                 * @param array<Tk, Tv> $elements
                 */
                public function __construct(array $elements = []) {
                    $this->elements = $elements;
                }

                /**
                 * @return static
                 */
                public static function getStatic(): static {
                    return new self(); // `self` is same as `static` since the class is final
                }
            }
        "#},
    }
}
