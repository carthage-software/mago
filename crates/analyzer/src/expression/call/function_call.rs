use mago_atom::atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::template::TemplateResult;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::call::analyze_invocation_targets;
use crate::expression::call::get_function_like_target;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for FunctionCall<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut template_result = TemplateResult::default();
        let (invocation_targets, encountered_invalid_targets) =
            resolve_targets(context, block_context, artifacts, self.function, &mut template_result)?;

        analyze_invocation_targets(
            context,
            block_context,
            artifacts,
            template_result,
            invocation_targets,
            InvocationArgumentsSource::ArgumentList(&self.argument_list),
            self.span(),
            encountered_invalid_targets,
            false,
            false,
        )
    }
}

pub(super) fn resolve_targets<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    expression: &Expression<'arena>,
    template_result: &mut TemplateResult,
) -> Result<(Vec<InvocationTarget<'ctx>>, bool), AnalysisError> {
    if let Expression::Identifier(function_name) = expression {
        let name = atom(context.resolved_names.get(function_name));
        let unqualified_name = atom(function_name.value());

        let identifier = FunctionLikeIdentifier::Function(name);
        let alternative = if function_name.is_local() && unqualified_name != name {
            Some(FunctionLikeIdentifier::Function(unqualified_name))
        } else {
            None
        };

        let target = get_function_like_target(context, identifier, alternative, expression.span(), None)?;

        return Ok(if let Some(t) = target { (vec![t], false) } else { (vec![], false) });
    }

    let was_inside_call = block_context.inside_call;
    block_context.inside_call = true;
    expression.analyze(context, block_context, artifacts)?;
    block_context.inside_call = was_inside_call;

    let Some(expression_type) = artifacts.get_expression_type(expression) else {
        return Ok((vec![], false));
    };

    let mut encountered_invalid_targets = false;
    let mut targets = vec![];
    for atomic in expression_type.types.as_ref() {
        if let Some(callable) = cast_atomic_to_callable(atomic, context.codebase, Some(template_result)) {
            match callable.as_ref() {
                TCallable::Signature(callable_signature) => {
                    if let Some(id) = callable_signature.get_source()
                        && let Some(target) = get_function_like_target(
                            context,
                            id,
                            None,
                            expression.span(),
                            callable_signature.return_type.clone(),
                        )?
                    {
                        targets.push(target);
                        continue;
                    }

                    targets.push(InvocationTarget::Callable {
                        signature: callable_signature.clone(),
                        span: expression.span(),
                        source: callable_signature.get_source(),
                    });
                }
                TCallable::Alias(id) => {
                    if let Some(t) = get_function_like_target(context, *id, None, expression.span(), None)? {
                        targets.push(t);
                    } else {
                        encountered_invalid_targets = true;
                    }
                }
            };
        } else {
            let type_name = atomic.get_id();

            context.collector.report_with_code(
                IssueCode::InvalidCallable,
                Issue::error(format!(
                    "Expression of type `{type_name}` cannot be called as a function or method.",
                ))
                .with_annotation(
                    Annotation::primary(expression.span())
                        .with_message(format!("This expression (type `{type_name}` ) is not a valid callable"))
                )
                .with_note("To be callable, an expression must resolve to a function name (string), a Closure, an invocable object (object with `__invoke` method), or an array representing a static/instance method.")
                .with_help("Ensure the expression evaluates to a callable type. If it's a variable, check its assigned type. If it's a string, ensure it's a defined function name or valid callable array syntax.".to_string()),
            );

            encountered_invalid_targets = true;
        }
    }

    Ok((targets, encountered_invalid_targets))
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = call_simple_addition,
        code = indoc! {r#"
            <?php

            function add(int $a, int $b): int {
                return $a + $b;
            }

            $result = add(5, 10);
        "#}
    }

    test_analysis! {
        name = call_string_concat,
        code = indoc! {r#"
            <?php

            function concatenate(string $s1, string $s2): string {
                return $s1 . $s2;
            }

            $greeting = concatenate("Hello, ", "World!");
        "#}
    }

    test_analysis! {
        name = call_function_returning_void,
        code = indoc! {r#"
            <?php

            function print_message(string $message): void {
                echo $message;
            }

            print_message("Test message");
        "#}
    }

    test_analysis! {
        name = call_with_optional_parameter_provided,
        code = indoc! {r#"
            <?php

            function greet(string $name, string $greeting = "Hello"): string {
                return $greeting . ", " . $name . "!";
            }

            $message = greet("Alice", "Hi");
        "#}
    }

    test_analysis! {
        name = call_with_optional_parameter_omitted,
        code = indoc! {r#"
            <?php

            function greet(string $name, string $greeting = "Hello"): string {
                return $greeting . ", " . $name . "!";
            }

            $message = greet("Bob");
        "#}
    }

    test_analysis! {
        name = call_with_function_result_as_argument,
        code = indoc! {r#"
            <?php

            function get_number(): int {
                return 42;
            }

            function display_number(int $num): void {
                echo "Number is: " . $num;
            }

            display_number(get_number());
        "#}
    }

    test_analysis! {
        name = call_wrong_argument_type_int_for_string,
        code = indoc! {r#"
            <?php

            function needs_string(string $s): void {
                echo $s;
            }

            needs_string(123);
        "#},
        issues = [IssueCode::InvalidArgument]
    }

    test_analysis! {
        name = call_wrong_argument_type_string_for_int,
        code = indoc! {r#"
            <?php

            function needs_int(int $i): void {}

            needs_int("hello");
        "#},
        issues = [
            IssueCode::InvalidArgument,
        ]
    }

    test_analysis! {
        name = call_too_few_arguments,
        code = indoc! {r#"
            <?php

            function requires_two(int $a, int $b): void {}

            requires_two(1);
        "#},
        issues = [
            IssueCode::TooFewArguments,
        ]
    }

    test_analysis! {
        name = call_too_many_arguments,
        code = indoc! {r#"
            <?php

            function accepts_one(int $a): void {}

            accepts_one(1, 2);
        "#},
        issues = [
            IssueCode::TooManyArguments,
        ]
    }

    test_analysis! {
        name = call_null_for_non_nullable_string,
        code = indoc! {r#"
            <?php

            function needs_string(string $s): void {}

            needs_string(null);
        "#},
        issues = [
            IssueCode::NullArgument,
        ]
    }

    test_analysis! {
        name = call_nullable_for_non_nullable_string,
        code = indoc! {r#"
            <?php

            function needs_string(string $s): void {}
            function get_string_or_null(): string|null {
                return get_string_or_null();
            }

            needs_string(get_string_or_null());
        "#},
        issues = [
            IssueCode::PossiblyNullArgument,
        ]
    }

    test_analysis! {
        name = call_false_for_non_falsable_string,
        code = indoc! {r#"
            <?php

            function needs_string(string $s): void {}

            needs_string(false);
        "#},
        issues = [
            IssueCode::FalseArgument,
        ]
    }

    test_analysis! {
        name = call_falsable_for_non_falsable_string,
        code = indoc! {r#"
            <?php

            function needs_string(string $s): void {}
            function get_string_or_false(): string|false {
                return get_string_or_false();
            }

            needs_string(get_string_or_false());
        "#},
        issues = [
            IssueCode::PossiblyFalseArgument,
        ]
    }

    test_analysis! {
        name = call_unknown_named_argument,

        code = indoc!{r#"
            <?php

            function known_params(int $a, string $b): void {}

            known_params(a: 1, c: "test");
        "#},
        issues = [
            IssueCode::InvalidNamedArgument,
        ]
    }

    test_analysis! {
        name = call_callable_param_type_mismatch,
        code = indoc! {r#"
            <?php

            /**
             * @param (callable(string): void) $cb
             */
            function needs_callable_string_param(callable $cb): void {
                $cb("hello");
            }

            /**
             * @param (callable(int): void) $arg_cb
             */
            function needs_callable_int_param(callable $arg_cb): void {
                needs_callable_string_param($arg_cb); // invalid callable type
            }
        "#},
        issues = [
            IssueCode::InvalidArgument
        ]
    }

    test_analysis! {
        name = call_callable_return_type_mismatch,
        code = indoc! {r#"
            <?php

            /**
             * @param (callable(string): int) $cb
             */
            function needs_callable_return_int(callable $cb): void {
                $val = $cb("test");

                echo $val;
            }

            /**
             * @param (callable(string): string) $arg_cb
             */
            function main_callable_return_string(callable $arg_cb): void {
                needs_callable_return_int($arg_cb);
            }
        "#},
        issues = [IssueCode::InvalidArgument]
    }

    test_analysis! {
        name = call_callable_too_few_params_in_arg,
        code = indoc! {r#"
            <?php

            /**
             * @param (callable(string, int): void) $cb
             */
            function needs_callable_two_params(callable $cb): void {
                $cb("hello", 1);
            }

            /**
             * @param (callable(string): void) $arg_cb
             */
            function main_callable_one_param(callable $arg_cb): void {
                needs_callable_two_params($arg_cb);
            }
        "#}
    }

    test_analysis! {
        name = call_callable_too_many_params_in_arg,
        code = indoc! {r#"
            <?php

            /**
             * @param (callable(string): void) $cb
             */
            function needs_callable_one_param(callable $cb): void {
                $cb("hello");
            }

            /**
             * @param (callable(string, int): void) $arg_cb
             */
            function main_callable_two_params(callable $arg_cb): void {
                needs_callable_one_param($arg_cb);
            }
        "#},
        issues = [IssueCode::PossiblyInvalidArgument]
    }

    test_analysis! {
        name = call_array_element_type_mismatch,
        code = indoc! {r#"
            <?php

            /**
             * @param list<string> $_list_of_strings
             */
            function needs_list_of_strings(array $_list_of_strings): void {}

            /**
             * @param list<int> $arg_list_of_ints
             */
            function main_list_of_ints(array $arg_list_of_ints): void {
                needs_list_of_strings($arg_list_of_ints);
            }

            main_list_of_ints([1, 2, 3]);
        "#},
        issues = [IssueCode::InvalidArgument]
    }

    test_analysis! {
        name = call_keyed_array_value_mismatch,
        code = indoc! {r#"
            <?php

            /**
             * @param array<string, int> $_map_to_int
             */
            function needs_map_to_int(array $_map_to_int): void {}

            /**
             * @param array<string, string> $arg_map_to_string
             */
            function main_map_to_string(array $arg_map_to_string): void {
                needs_map_to_int($arg_map_to_string);
            }

            main_map_to_string(["a" => "apple", "b" => "banana"]);
        "#},
        issues = [IssueCode::InvalidArgument]
    }

    test_analysis! {
        name = call_non_empty_list_constraint_violation,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-list<int> $_nel
             */
            function needs_non_empty_list(array $_nel): void {}

            /**
             * @param list<int> $arg_list Can be empty
             */
            function main_list_can_be_empty(array $arg_list): void {
                needs_non_empty_list($arg_list);
            }

            main_list_can_be_empty([]); // Definitely empty
        "#},
        issues = [IssueCode::PossiblyInvalidArgument]
    }

    test_analysis! {
        name = call_non_empty_array_key_type_mismatch,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-array<string, int> $_nea
             */
            function needs_non_empty_array_string_keys(array $_nea): void {}

            /**
             * @param non-empty-array<int, int> $arg_array_int_keys
             */
            function main_array_int_keys(array $arg_array_int_keys): void {
                needs_non_empty_array_string_keys($arg_array_int_keys);
            }

            main_array_int_keys([0 => 1, 1 => 2]);
        "#},
        issues = [IssueCode::InvalidArgument]
    }

    test_analysis! {
        name = call_union_param_invalid_type,
        code = indoc! {r#"
            <?php

            /**
             * @param int|string $_value
             */
            function needs_int_or_string(mixed $_value): void {}

            /**
             * @param bool $arg_bool
             */
            function main_bool_arg(bool $arg_bool): void {
                needs_int_or_string($arg_bool);
            }

            main_bool_arg(true);
        "#},
        issues = [IssueCode::InvalidArgument]
    }

    test_analysis! {
        name = call_template_callable_param_mismatch,
        code = indoc!{r#"
            <?php

            /**
             * @template T
             * @param (callable(T): int) $cb
             * @param T $val
             */
            function process_with_template(callable $cb, mixed $val): int {
                return $cb($val);
            }

            /**
             * @param (callable(string): int) $string_cb
             */
            function main_template_callable_mismatch(callable $string_cb): void {
                process_with_template($string_cb, 123); // T is int from 123, but $string_cb expects string
            }

            function string_to_int(string $s): int {
                return (int) $s;
            }

            main_template_callable_mismatch(
                string_to_int(...)
            );
        "#},
        issues = [IssueCode::PossiblyInvalidArgument]
    }

    test_analysis! {
        name = map_twice,
        code = indoc!{r#"
            <?php

            /**
             * @template T1
             * @template T2
             * @template T3
             * @param (Closure(T2):T3) $c2
             * @param (Closure(T1):T2) $c1
             * @param list<T1> $a
             * @return list<T3>
             */
            function maptwice(Closure $c2, Closure $c1, array $a): array
            {
                $res = [];
                foreach ($a as $v) {
                    $res[] = $c2($c1($v));
                }
                return $res;
            }

            /**
             * @param list<array{'a': string, 'b': int}> $input
             * @return list<int>
             */
            function foo(array $input): array
            {
                return maptwice(
                    static function (int $b): int {
                        return $b + 1;
                    },
                    /**
                     * @param array{'b': int, ...} $in
                     */
                    static function (array $in): int {
                        return $in['b'];
                    },
                    $input,
                );
            }
        "#}
    }

    test_analysis! {
        name = call_with_generic_constraints,
        code = indoc!{r#"
            <?php

            /**
             * @template TKey as array-key
             * @template TValue
             * @param array<TKey, TValue> $_array
             * @param array ...$_arrays
             * @return array<TKey, TValue>
             */
            function array_intersect(array $_array, array ...$_arrays): array
            {
                exit();
            }

            /**
             * @template K of array-key
             * @template V
             * @param iterable<K, V> $iterable
             * @return array<K, V>
             */
            function keyed_array_from_iterable(iterable $iterable): array
            {
                $dict = [];
                foreach ($iterable as $key => $value) {
                    $dict[$key] = $value;
                }
                return $dict;
            }

            /**
             * @template T
             * @template R
             * @param iterable<T> $iterable
             * @param (callable(T): R) $callback
             * @return list<R>
             */
            function map_list(iterable $iterable, callable $callback): array
            {
                $result = [];
                foreach ($iterable as $value) {
                    $result[] = $callback($value);
                }

                return $result;
            }

            /**
             * @template Tk of array-key
             * @template Tv
             * @param iterable<Tk, Tv> $first
             * @param iterable<Tk, mixed> $second
             * @param list<iterable<Tk, mixed>> $rest
             * @return array<Tk, Tv>
             */
            function intersect(iterable $first, iterable $second, iterable ...$rest): array
            {
                return array_intersect(
                    keyed_array_from_iterable($first),
                    keyed_array_from_iterable($second),
                    ...map_list(
                        $rest,
                        /**
                         * @param iterable<Tk, Tv> $iterable
                         * @return array<Tk, Tv>
                         */
                        static fn(iterable $iterable): array => keyed_array_from_iterable($iterable),
                    ),
                );
            }
        "#},
    }

    test_analysis! {
        name = conditional_returns,
        code = indoc!{r#"
            <?php

            /**
             * @template Input as int
             *
             * @param Input $a
             *
             * @return (Input is 1 ? 2 : (
             *    Input is 2 ? 3 : (
             *      Input is 3 ? 4 : (
             *        Input is not 4 ? (
             *            Input is 5 ? 6 : int
             *        ) : 5
             *      )
             *    )
             * ))
             */
            function add_one(int $a): int {
                return $a + 1;
            }

            /**
             * @param 20 $_
             */
            function i_take_20(int $_): void {}

            $a = add_one(1); // 2
            $b = add_one(2); // 3
            $c = add_one(3); // 4
            $d = add_one(4); // 5
            $e = add_one(5); // 6

            $f = $a + $b + $c + $d + $e; // 5 + 6 + 7 + 8 + 9 = 20

            i_take_20($f); // no error, $f is 20
        "#},
    }

    test_analysis! {
        name = call_capture_groups,
        code = indoc! {r#"
            <?php

            /**
             * @template-covariant T
             */
            interface TypeInterface
            {
            }

            /**
             * @template Tk of array-key
             * @template Tv
             * @param iterable<Tk, Tv> $iterable
             * @return array<Tk, Tv>
             */
            function dict_unique(iterable $iterable): array
            {
                return dict_unique($iterable);
            }

            /**
             * @template Tk as array-key
             * @template Tv
             * @param iterable<Tk> $keys
             * @param (Closure(Tk): Tv) $value_func
             * @return array<Tk, Tv>
             */
            function dict_from_keys(iterable $keys, Closure $value_func): array
            {
                return dict_from_keys($keys, $value_func);
            }

            /**
             * @template Tk of array-key
             * @template Tv
             * @param array<Tk, TypeInterface<Tv>> $elements
             * @return TypeInterface<array<Tk, Tv>>
             */
            function shape_type(array $elements, bool $allow_unknown_fields = false): TypeInterface
            {
                return shape_type($elements, $allow_unknown_fields);
            }

            /**
             * @return TypeInterface<string>
             */
            function string_type(): TypeInterface
            {
                return string_type();
            }

            /**
             * @param list<array-key> $groups
             * @return TypeInterface<array<array-key, string>>
             */
            function capture_groups(array $groups): TypeInterface
            {
                return shape_type(dict_from_keys(
                    dict_unique([0, ...$groups]),
                    /**
                     * @return TypeInterface<string>
                     */
                    static fn($_): TypeInterface => string_type(),
                ));
            }

            /**
             * @param list<array-key> $groups
             * @return TypeInterface<array<array-key, string>>
             */
            function capture_groups_2(array $groups): TypeInterface
            {
                return shape_type(dict_from_keys(
                    dict_unique([0, ...$groups]),
                    /**
                     * @return TypeInterface<string>
                     */
                    static fn(): TypeInterface => string_type(),
                ));
            }

            /**
             * @param list<array-key> $groups
             * @return TypeInterface<array<array-key, string>>
             */
            function capture_groups_3(array $groups): TypeInterface
            {
                return shape_type(dict_from_keys(
                    dict_unique([0, ...$groups]),
                    string_type(...),
                ));
            }
        "#},
    }

    test_analysis! {
        name = member_reference_argument,
        code = indoc! {r#"
            <?php

            class ChangeKind {
                public const ADD = 'add';
                public const REMOVE = 'remove';
                public const UPDATE = 'update';
                public const RENAME = 'rename';
                public const MOVE = 'move';
            }

            /**
             * @param ChangeKind::ADD|ChangeKind::*ME|ChangeKind::U* $kind
             */
            function foo(string $kind): string {
                return $kind;
            }

            foo(ChangeKind::ADD);    // OK (literal matches)
            foo('add');              // OK (literal matches)
            foo(ChangeKind::UPDATE); // OK (starts with 'U')
            foo('update');           // OK (starts with 'U')
            foo(ChangeKind::RENAME); // OK (ends with 'ME')
            foo('rename');           // OK (ends with 'ME')
        "#},
    }

    test_analysis! {
        name = iterable_for_traversable_or_array,
        code = indoc! {r#"
            <?php

            /**
             * @template K
             * @template-covariant V
             */
            interface Traversable {}

            /**
             * @template K as array-key
             * @template V
             *
             * @param array<K, V>|Traversable<K, V> $_input
             *
             * @return array<K, V>
             */
            function as_array(array|Traversable $_input): array {
                return [];
            }

            /**
             * @template K as array-key
             * @template V
             *
             * @param iterable<K, V> $input
             *
             * @return array<K, V>
             */
            function iter_as_array(iterable $input): array {
                return as_array($input);
            }
        "#},
    }

    test_analysis! {
        name = invalid_member_reference_argument,
        code = indoc! {r#"
            <?php

            class ChangeKind {
                public const ADD = 'add';
                public const REMOVE = 'remove';
                public const UPDATE = 'update';
                public const RENAME = 'rename';
                public const MOVE = 'move';
            }

            /**
             * @param ChangeKind::ADD|ChangeKind::*ME|ChangeKind::U* $kind
             */
            function foo(string $kind): string {
                return $kind;
            }

            foo(ChangeKind::MOVE);   // Error: 'move' does not match any pattern
            foo('move');             // Error: 'move' does not match any pattern
            foo(ChangeKind::REMOVE); // Error: 'remove' does not match any pattern
            foo('remove');           // Error: 'remove' does not match any pattern
            foo('unknown');          // Error: 'unknown' does not match any pattern
        "#},
        issues = [
            IssueCode::InvalidArgument,
            IssueCode::InvalidArgument,
            IssueCode::InvalidArgument,
            IssueCode::InvalidArgument,
            IssueCode::InvalidArgument,
        ],
    }

    test_analysis! {
        name = enum_member_reference_argument,
        code = indoc! {r#"
            <?php

            enum ChangeKind {
                case ADD;
                case REMOVE;
                case UPDATE;
                case RENAME;
                case MOVE;
            }

            /**
             * @param ChangeKind::ADD|ChangeKind::*ME|ChangeKind::U* $kind
             */
            function foo(ChangeKind $kind): ChangeKind {
                return $kind;
            }

            foo(ChangeKind::ADD);    // OK (literal matches)
            foo(ChangeKind::UPDATE); // OK (starts with 'U')
            foo(ChangeKind::RENAME); // OK (ends with 'ME')
        "#},
    }

    test_analysis! {
        name = invalid_enum_member_reference_argument,
        code = indoc! {r#"
            <?php

            enum ChangeKind {
                case ADD;
                case REMOVE;
                case UPDATE;
                case RENAME;
                case MOVE;
            }

            /**
             * @param ChangeKind::ADD|ChangeKind::*ME|ChangeKind::U* $kind
             */
            function foo(ChangeKind $kind): ChangeKind {
                return $kind;
            }

            foo(ChangeKind::MOVE);   // Error: 'move' does not match any pattern
            foo(ChangeKind::REMOVE); // Error: 'remove' does not match any pattern
        "#},
        issues = [
            IssueCode::PossiblyInvalidArgument,
            IssueCode::PossiblyInvalidArgument,
        ],
    }

    test_analysis! {
        name = type_logic_sanity_test,
        code = indoc! {r#"
            <?php

            interface A {}
            interface B {}

            function i_take_a(A $a): A {
                i_take_a_or_b($a);
                i_take_b_or_a($a);

                return $a;
            }

            function i_take_b(B $b): B {
                i_take_b_or_a($b);
                i_take_a_or_b($b);

                return $b;
            }

            function i_take_a_or_b(A|B $aOrB): B|A {
                if ($aOrB instanceof A) {
                    i_take_a($aOrB);
                } else {
                    i_take_b($aOrB);
                }

                return $aOrB;
            }

            function i_take_b_or_a(B|A $bOrA): A|B {
                if ($bOrA instanceof B) {
                    i_take_b($bOrA);
                } else {
                    i_take_a($bOrA);
                }

                return $bOrA;
            }

            function i_take_a_and_b(A&B $aAndB): B&A {
                i_take_a($aAndB);
                i_take_b($aAndB);
                i_take_a_or_b($aAndB);
                i_take_b_or_a($aAndB);

                return $aAndB;
            }

            function i_take_b_and_a(B&A $bAndA): A&B {
                i_take_b($bAndA);
                i_take_a($bAndA);
                i_take_a_or_b($bAndA);
                i_take_b_or_a($bAndA);

                return $bAndA;
            }
        "#},
    }
}
