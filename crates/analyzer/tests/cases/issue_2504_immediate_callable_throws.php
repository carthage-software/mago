<?php

declare(strict_types=1);

final class ImmediateCallbackFailure extends RuntimeException
{
}

final class SwallowedCallbackFailure extends RuntimeException
{
}

/**
 * @throws ImmediateCallbackFailure
 */
function named_immediate_array_callback(array $matches): string
{
    return (string) might_fail();
}

final class ImmediateArrayCallbackProvider
{
    /**
     * @throws ImmediateCallbackFailure
     */
    public static function callback(array $matches): string
    {
        return (string) might_fail();
    }
}

/**
 * @throws ImmediateCallbackFailure
 */
function might_fail(): int
{
    throw new ImmediateCallbackFailure();
}

/**
 * @throws SwallowedCallbackFailure
 */
function might_fail_with_swallowed_type(): int
{
    throw new SwallowedCallbackFailure();
}

/**
 * @throws RuntimeException
 */
function might_fail_with_runtime_exception(): int
{
    throw new RuntimeException();
}

/**
 * @throws ImmediateCallbackFailure
 */
function might_fail_for_value(int $value): int
{
    return might_fail();
}

/**
 * @throws ImmediateCallbackFailure
 */
function array_map_caller(): array
{
    return array_map(static fn (int $value): int => might_fail(), [1]);
}

/**
 * @throws ImmediateCallbackFailure
 */
function parenthesized_array_map_caller(): array
{
    return array_map((static fn (int $value): int => might_fail()), [1]);
}

/**
 * @throws ImmediateCallbackFailure
 */
function array_filter_caller(): array
{
    return array_filter(array: [1], callback: static fn (int $value): bool => might_fail() > 0);
}

/**
 * @throws ImmediateCallbackFailure
 */
function preg_replace_callback_array_caller(): array|string|null
{
    return preg_replace_callback_array(
        ['~foo~' => static fn (array $matches): string => (string) might_fail()],
        'foo',
    );
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_preg_replace_callback_array_caller(): array|string|null
{
    return preg_replace_callback_array(
        ['~foo~' => static fn (array $matches): string => (string) might_fail()],
        'foo',
    );
}

/**
 * @throws ImmediateCallbackFailure
 */
function preg_replace_callback_array_named_function_caller(): array|string|null
{
    return preg_replace_callback_array(['~foo~' => 'named_immediate_array_callback'], 'foo');
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_preg_replace_callback_array_named_function_caller(): array|string|null
{
    return preg_replace_callback_array(['~foo~' => 'named_immediate_array_callback'], 'foo');
}

/**
 * @throws ImmediateCallbackFailure
 */
function preg_replace_callback_array_method_array_caller(): array|string|null
{
    return preg_replace_callback_array(
        ['~foo~' => [ImmediateArrayCallbackProvider::class, 'callback']],
        'foo',
    );
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_preg_replace_callback_array_method_array_caller(): array|string|null
{
    return preg_replace_callback_array(
        ['~foo~' => [ImmediateArrayCallbackProvider::class, 'callback']],
        'foo',
    );
}

/**
 * @throws ImmediateCallbackFailure
 */
function preg_replace_callback_array_callable_variable_caller(): array|string|null
{
    $callback = 'named_immediate_array_callback';

    return preg_replace_callback_array(['~foo~' => $callback], 'foo');
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_preg_replace_callback_array_callable_variable_caller(): array|string|null
{
    $callback = 'named_immediate_array_callback';

    return preg_replace_callback_array(['~foo~' => $callback], 'foo');
}

/**
 * @throws ImmediateCallbackFailure
 */
function array_udiff_caller(): array
{
    return array_udiff([1], [2], static fn (mixed $left, mixed $right): int => might_fail());
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_array_udiff_caller(): array
{
    return array_udiff([1], [2], static fn (mixed $left, mixed $right): int => might_fail());
}

function array_udiff_with_stored_closure_value(): array
{
    return array_udiff(
        [1],
        [
            /**
             * @throws ImmediateCallbackFailure
             */
            static function (): int {
                return might_fail();
            },
        ],
        static fn (mixed $left, mixed $right): int => 0,
    );
}

function array_udiff_callable_array_operand_is_not_callback(): array
{
    return array_udiff(
        [1],
        [ImmediateCallbackMethods::class, 'mightFail'],
        static fn (mixed $left, mixed $right): int => 0,
    );
}

/**
 * @throws ImmediateCallbackFailure
 * @throws SwallowedCallbackFailure
 */
function array_udiff_uassoc_caller(): array
{
    return array_udiff_uassoc(
        [1],
        [2],
        static fn (mixed $left, mixed $right): int => might_fail(),
        static fn (mixed $left, mixed $right): int => might_fail_with_swallowed_type(),
    );
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_array_map_caller(): array
{
    return array_map(static fn (int $value): int => might_fail(), [1]);
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_now(callable $callback): int
{
    return $callback();
}

/**
 * @throws ImmediateCallbackFailure
 */
function userland_caller(): int
{
    return invoke_now(static fn (): int => might_fail());
}

function call_user_func_wrapper_caller_before_wrapper(): void
{
    invoke_and_catch_via_call_user_func(static fn (): int => might_fail());
}

function array_map_wrapper_caller_before_wrapper(): void
{
    invoke_and_catch_via_array_map(static fn (): int => might_fail());
}

function array_map_adapter_wrapper_caller_before_wrapper(): void
{
    invoke_and_catch_via_array_map_adapter(static fn (): int => might_fail());
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_userland_caller(): int
{
    return invoke_now(static fn (): int => might_fail());
}

function forward_caught_wrapper_caller(): void
{
    invoke_and_catch_later(static fn (): int => might_fail());
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function forward_uncaught_wrapper_caller(): int
{
    return invoke_now_later(static fn (): int => might_fail());
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function forward_rethrowing_wrapper_caller(): int
{
    return invoke_and_rethrow_later(static fn (): int => might_fail());
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function forward_mixed_rethrow_wrapper_caller(): int
{
    return invoke_rethrow_or_catch_later(static fn (): int => might_fail());
}

function forward_mixed_swallowed_wrapper_caller(): int
{
    return invoke_rethrow_or_catch_later(static fn (): int => might_fail_with_swallowed_type());
}

function forward_shadowed_catch_wrapper_caller(): int
{
    return invoke_shadowed_catch_later(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch(callable $callback): void
{
    try {
        $callback();
    } catch (ImmediateCallbackFailure) {
    }
}

function caught_by_userland_wrapper(): void
{
    invoke_and_catch(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch_via_array_map(callable $callback): void
{
    try {
        array_map($callback, [1]);
    } catch (ImmediateCallbackFailure) {
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch_via_array_map_adapter(callable $callback): void
{
    try {
        array_map(static fn (int $value): int => $callback(), [1]);
    } catch (ImmediateCallbackFailure) {
    }
}

function caught_by_array_map_userland_wrapper(): void
{
    invoke_and_catch_via_array_map(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch_inside_nested_try(callable $callback): void
{
    try {
        $callback();
    } catch (ImmediateCallbackFailure) {
        try {
            throw new ImmediateCallbackFailure();
        } catch (RuntimeException) {
        }
    }
}

function caught_by_nested_userland_wrapper(): void
{
    invoke_and_catch_inside_nested_try(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch_dynamic_nested_try(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        try {
            throw $exception;
        } catch (RuntimeException) {
        }
    }
}

function caught_by_dynamic_nested_wrapper(): void
{
    invoke_and_catch_dynamic_nested_try(static fn (): int => might_fail_with_runtime_exception());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws RuntimeException
 * @mago-expect analysis:dead-catch
 */
function invoke_and_rethrow_after_unmatched_nested_catch(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException) {
        try {
            throw new RuntimeException();
        } catch (LogicException) {
        }

        throw new SwallowedCallbackFailure();
    }
}

/**
 * @throws RuntimeException
 */
function caught_by_unmatched_nested_wrapper(): void
{
    invoke_and_rethrow_after_unmatched_nested_catch(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 * @mago-expect analysis:dead-catch
 * @mago-expect analysis:avoid-catching-error
 */
function invoke_with_unreachable_nested_catch(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        try {
            throw new Error();
        } catch (Throwable) {
        } catch (Error) {
            throw $exception;
        }
    }
}

/**
 * @throws Error
 */
function unreachable_nested_catch_caller(): void
{
    invoke_with_unreachable_nested_catch(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unevaluated-code(2)
 */
function invoke_and_rethrow_from_goto_label(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        goto rethrow_callback_exception;
        return;
        rethrow_callback_exception:
        throw $exception;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_goto_label_rethrow_caller(): void
{
    invoke_and_rethrow_from_goto_label(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unevaluated-code(6)
 * @mago-expect analysis:useless-control-flow
 */
function invoke_and_rethrow_through_chained_goto_labels(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        goto first_rethrow_label;
        return;
        second_rethrow_label:
        goto final_rethrow_label;
        return;
        first_rethrow_label:
        goto second_rethrow_label;
        final_rethrow_label:
        throw $exception;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_chained_goto_label_rethrow_caller(): void
{
    invoke_and_rethrow_through_chained_goto_labels(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:dead-catch
 * @mago-expect analysis:avoid-catching-error
 * @mago-expect analysis:unhandled-thrown-type
 */
function replace_callback_exception_before_unreachable_nested_catch_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        try {
            throw new Error();
        } catch (Throwable) {
            throw new SwallowedCallbackFailure();
        } catch (Error) {
            goto unreachable_nested_catch_label;
        }

        throw $exception;
    }

    unreachable_nested_catch_label:
    return;
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_before_unreachable_nested_catch_goto_caller(): void
{
    replace_callback_exception_before_unreachable_nested_catch_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function invoke_with_break_before_unreachable_rethrow(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        switch (mt_rand(0, 1)) {
            case 0:
                if (mt_rand(0, 1) === 1) {
                    break;
                    throw $exception;
                }

                throw new SwallowedCallbackFailure();
            default:
                throw new SwallowedCallbackFailure();
        }
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function break_before_unreachable_rethrow_caller(): void
{
    invoke_with_break_before_unreachable_rethrow(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unevaluated-code
 */
function invoke_with_nested_switch_transfers(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        while (mt_rand(0, 1) === 1) {
            switch (mt_rand(0, 2)) {
                case 0:
                    break 2;
                case 1:
                    continue 2;
                default:
                    return;
            }

            throw $exception;
        }
    }
}

function nested_switch_transfers_caller(): void
{
    invoke_with_nested_switch_transfers(static fn (): int => might_fail());
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function nested_switch_break_three_exits_outer_loop(): void
{
    try {
        throw new RuntimeException();
    } catch (RuntimeException $exception) {
        while (mt_rand(0, 1) === 1) {
            while (mt_rand(0, 1) === 1) {
                switch (mt_rand(0, 1)) {
                    case 0:
                        break 3;
                    default:
                        break 3;
                }

                throw $exception;
            }
        }
    }
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function nested_switch_break_two_exits_outer_switch(): void
{
    switch (mt_rand(0, 1)) {
        case 0:
            switch (mt_rand(0, 1)) {
                case 0:
                    break 2;
                default:
                    break 2;
            }

            throw new RuntimeException();
        default:
            return;
    }
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function nested_switch_continue_two_exits_outer_switch(): void
{
    switch (mt_rand(0, 1)) {
        case 0:
            switch (mt_rand(0, 1)) {
                case 0:
                    continue 2;
                default:
                    continue 2;
            }

            throw new RuntimeException();
        default:
            return;
    }
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function nested_switch_transfers_preserve_outer_switch_fallthrough(): int
{
    switch (mt_rand(0, 1)) {
        case 0:
            switch (mt_rand(0, 1)) {
                case 0:
                    break 2;
                default:
                    return 0;
            }

            throw new RuntimeException();
        default:
            return 0;
    }

    return 1;
}

function nested_switch_break_three_merges_exiting_case_variables(): int
{
    $value = 1;

    do {
        do {
            switch (mt_rand(0, 1)) {
                case 0:
                    $value = 'string';
                    break 3;
                default:
                    return 0;
            }
        } while (false);
    } while (false);

    return strlen($value);
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws RuntimeException
 * @mago-expect analysis:dead-catch
 * @mago-expect analysis:unevaluated-code
 */
function invoke_with_try_nested_switch_transfer_and_unreachable_catch(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        while (mt_rand(0, 1) === 1) {
            switch (mt_rand(0, 1)) {
                case 0:
                    try {
                        break 2;
                    } catch (Throwable) {
                    }
                default:
                    return;
            }

            throw $exception;
        }
    }
}

function try_nested_switch_transfer_with_unreachable_catch_caller(): void
{
    try {
        invoke_with_try_nested_switch_transfer_and_unreachable_catch(static fn (): int => might_fail());
    } catch (RuntimeException) {
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unevaluated-code(2)
 */
function invoke_with_goto_before_nested_switch_transfer(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        while (mt_rand(0, 1) === 1) {
            switch (mt_rand(0, 1)) {
                case 0:
                    goto after_switch;
                    break 2;
                default:
                    return;
            }

            after_switch:
            throw $exception;
        }
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unevaluated-code
 */
function invoke_with_try_nested_switch_transfers(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        while (mt_rand(0, 1) === 1) {
            switch (mt_rand(0, 2)) {
                case 0:
                    try {
                        break 2;
                    } finally {
                    }
                case 1:
                    try {
                        continue 2;
                    } finally {
                    }
                default:
                    return;
            }

            throw $exception;
        }
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_and_rethrow_runtime_exception(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_broad_catch_rethrow_caller(): void
{
    invoke_and_rethrow_runtime_exception(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws RuntimeException
 */
function invoke_and_rethrow_callback_from_finally(callable $callback): void
{
    $caught = null;

    try {
        $callback();
    } catch (RuntimeException $exception) {
        $caught = $exception;
    } finally {
        if ($caught !== null) {
            throw $caught;
        }
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_finally_rethrow_caller(): void
{
    invoke_and_rethrow_callback_from_finally(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function replace_callback_exception_in_finally(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        throw new SwallowedCallbackFailure();
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_from_finally_caller(): void
{
    replace_callback_exception_in_finally(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function replace_callback_exception_before_unreachable_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        throw new SwallowedCallbackFailure();
        goto after_finally_throw;
        after_finally_throw:
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_before_unreachable_goto_caller(): void
{
    replace_callback_exception_before_unreachable_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function replace_callback_exception_before_unreachable_switch_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        switch (mt_rand(0, 1)) {
            case 0:
                throw new SwallowedCallbackFailure();
                goto after_finally_switch;
            default:
                throw new SwallowedCallbackFailure();
        }

        after_finally_switch:
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_before_unreachable_switch_goto_caller(): void
{
    replace_callback_exception_before_unreachable_switch_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function replace_callback_exception_before_unreachable_if_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        if (mt_rand(0, 1) === 1):
            throw new SwallowedCallbackFailure();
            goto after_finally_if;
        else:
            throw new SwallowedCallbackFailure();
        endif;

        after_finally_if:
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_before_unreachable_if_goto_caller(): void
{
    replace_callback_exception_before_unreachable_if_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function replace_callback_exception_in_exhaustive_finally_branches(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        if (mt_rand(0, 1) === 1) {
            throw new SwallowedCallbackFailure();
        } else {
            throw new SwallowedCallbackFailure();
        }
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_in_exhaustive_finally_branches_caller(): void
{
    replace_callback_exception_in_exhaustive_finally_branches(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unevaluated-code
 */
function return_from_finally_after_callback_rethrow(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        return;
        throw $exception;
    }
}

function callback_exception_suppressed_by_finally_return(): void
{
    return_from_finally_after_callback_rethrow(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function replace_callback_exception_in_nested_try_finally(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        try {
            throw new SwallowedCallbackFailure();
        } finally {
        }
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_in_nested_try_finally_caller(): void
{
    replace_callback_exception_in_nested_try_finally(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function replace_callback_exception_in_nested_try_catch(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        try {
            throw new SwallowedCallbackFailure();
        } catch (RuntimeException) {
            throw new SwallowedCallbackFailure();
        }
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_in_nested_try_catch_caller(): void
{
    replace_callback_exception_in_nested_try_catch(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code(2)
 */
function replace_callback_exception_in_exhaustive_finally_switch(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        switch (mt_rand(0, 1)) {
            case 0:
                while (mt_rand(0, 1) === 1) {
                    break 1;
                }

                throw new SwallowedCallbackFailure();
                throw $exception;
            default:
                throw new SwallowedCallbackFailure();
                throw $exception;
        }
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_in_exhaustive_finally_switch_caller(): void
{
    replace_callback_exception_in_exhaustive_finally_switch(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws RuntimeException
 * @throws ImmediateCallbackFailure
 * @throws SwallowedCallbackFailure
 */
function maybe_replace_callback_exception_in_finally_switch(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        switch (mt_rand(0, 1)) {
            case 0:
                if (mt_rand(0, 1) === 1) {
                    break;
                }

                throw new SwallowedCallbackFailure();
            default:
                throw new SwallowedCallbackFailure();
        }
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function maybe_replaced_callback_exception_in_finally_switch_caller(): void
{
    maybe_replace_callback_exception_in_finally_switch(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws ImmediateCallbackFailure
 * @throws RuntimeException
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function preserve_callback_exception_across_nested_finally_break(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        switch (mt_rand(0, 1)) {
            case 0:
                switch (mt_rand(0, 1)) {
                    case 0:
                        break 2;
                    default:
                        break 2;
                }

                throw new SwallowedCallbackFailure();
            default:
                throw new SwallowedCallbackFailure();
        }
    }
}

/**
 * @throws ImmediateCallbackFailure
 * @throws RuntimeException
 * @throws SwallowedCallbackFailure
 */
function preserved_callback_exception_across_nested_finally_break_caller(): void
{
    preserve_callback_exception_across_nested_finally_break(static fn (): int => might_fail());
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function finally_local_break_preserves_return(): int
{
    try {
        return 0;
    } finally {
        switch (mt_rand(0, 1)) {
            case 0:
                break;
            default:
                break;
            }
    }

    return might_fail();
}

/**
 * @throws ImmediateCallbackFailure
 */
function callback_after_return_or_break_switch_case(): int
{
    switch (mt_rand(0, 1)) {
        case 0:
            return 0;
        default:
            break;
    }

    return might_fail();
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws RuntimeException
 * @throws ImmediateCallbackFailure
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function skip_finally_throw_with_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        goto after_throw;
        throw new SwallowedCallbackFailure();
        after_throw:
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function skip_finally_throw_with_goto_caller(): void
{
    skip_finally_throw_with_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function replace_callback_exception_through_finally_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        goto after_unreachable_throw;
        throw new SwallowedCallbackFailure();
        after_unreachable_throw:
        throw new SwallowedCallbackFailure();
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_through_finally_goto_caller(): void
{
    replace_callback_exception_through_finally_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code(2)
 */
function replace_callback_exception_through_finally_goto_chain(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        goto after_throw;
        before_throw:
        throw new SwallowedCallbackFailure();
        after_throw:
        goto before_throw;
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_through_finally_goto_chain_caller(): void
{
    replace_callback_exception_through_finally_goto_chain(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unevaluated-code
 */
function replace_callback_exception_through_finally_unreachable_switch_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        switch (1) {
            case 0:
                goto after_switch;
            default:
                throw new SwallowedCallbackFailure();
        }

        after_switch:
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_through_finally_unreachable_switch_goto_caller(): void
{
    replace_callback_exception_through_finally_unreachable_switch_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function replace_callback_exception_in_exhaustive_finally_match(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        match (mt_rand(0, 1)) {
            0 => throw new SwallowedCallbackFailure(),
            default => throw new SwallowedCallbackFailure(),
        };
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_callback_exception_in_exhaustive_finally_match_caller(): void
{
    replace_callback_exception_in_exhaustive_finally_match(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function invoke_and_replace_caught_exception(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        $exception = new SwallowedCallbackFailure();
        throw $exception;
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_caught_exception_caller(): void
{
    invoke_and_replace_caught_exception(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_and_conditionally_replace_caught_exception(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        if (mt_rand(0, 1) === 1) {
            $exception = new SwallowedCallbackFailure();
        }

        throw $exception;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_conditional_catch_rethrow_caller(): void
{
    invoke_and_conditionally_replace_caught_exception(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_and_conditionally_replace_caught_exception_in_loop(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        while (mt_rand(0, 1) === 1) {
            $exception = new SwallowedCallbackFailure();
            break;
        }

        throw $exception;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_loop_catch_rethrow_caller(): void
{
    invoke_and_conditionally_replace_caught_exception_in_loop(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_and_conditionally_replace_caught_exception_in_switch(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        switch (mt_rand(0, 1)) {
            case 1:
                $exception = new SwallowedCallbackFailure();
                break;
        }

        throw $exception;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_switch_catch_rethrow_caller(): void
{
    invoke_and_conditionally_replace_caught_exception_in_switch(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function invoke_and_replace_caught_exception_in_nested_try(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        try {
            $exception = new SwallowedCallbackFailure();
        } finally {
        }

        throw $exception;
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function replaced_caught_exception_in_nested_try_caller(): void
{
    invoke_and_replace_caught_exception_in_nested_try(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch_later(callable $callback): void
{
    try {
        $callback();
    } catch (ImmediateCallbackFailure) {
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch_via_call_user_func(callable $callback): void
{
    try {
        call_user_func($callback);
    } catch (ImmediateCallbackFailure) {
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_now_later(callable $callback): int
{
    return $callback();
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_and_rethrow_later(callable $callback): int
{
    try {
        return $callback();
    } catch (ImmediateCallbackFailure $exception) {
        throw $exception;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function forward_alias_rethrowing_wrapper_caller(): int
{
    return invoke_alias_rethrow_later(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_alias_rethrow_later(callable $callback): int
{
    try {
        return $callback();
    } catch (ImmediateCallbackFailure $exception) {
        $alias = $exception;
        throw $alias;
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function forward_same_type_throw_wrapper_caller(): int
{
    return invoke_same_type_throw_later(static fn (): int => might_fail());
}

/**
 * @throws SwallowedCallbackFailure
 */
function forward_unrelated_throw_wrapper_caller(): int
{
    return invoke_unrelated_throw_later(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_same_type_throw_later(callable $callback): int
{
    try {
        return $callback();
    } catch (ImmediateCallbackFailure) {
        throw new ImmediateCallbackFailure();
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function invoke_unrelated_throw_later(callable $callback): int
{
    try {
        return $callback();
    } catch (ImmediateCallbackFailure) {
        throw new SwallowedCallbackFailure();
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_rethrow_or_catch_later(callable $callback): int
{
    try {
        return $callback();
    } catch (ImmediateCallbackFailure $exception) {
        throw $exception;
    } catch (SwallowedCallbackFailure) {
        return 0;
    }
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function invoke_shadowed_catch_later(callable $callback): int
{
    try {
        return $callback();
    } catch (ImmediateCallbackFailure $exception) {
        try {
            throw new SwallowedCallbackFailure();
        } catch (SwallowedCallbackFailure $exception) {
            throw $exception;
        }
    }
}

/**
 * @throws ImmediateCallbackFailure
 */
function array_map_function_string_caller(): array
{
    return array_map('might_fail_for_value', [1]);
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_array_map_function_string_caller(): array
{
    return array_map('might_fail_for_value', [1]);
}

/**
 * @throws ImmediateCallbackFailure
 */
function array_map_callable_variable_caller(): array
{
    $callback = /** @throws ImmediateCallbackFailure */ static fn (int $value): int => might_fail();

    return array_map($callback, [1]);
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_array_map_callable_variable_caller(): array
{
    $callback = /** @throws ImmediateCallbackFailure */ static fn (int $value): int => might_fail();

    return array_map($callback, [1]);
}

final class ImmediateCallbackMethods
{
    /**
     * @throws ImmediateCallbackFailure
     */
    public static function mightFail(int $value): int
    {
        return might_fail();
    }
}

/**
 * @throws ImmediateCallbackFailure
 */
function array_map_method_callable_caller(): array
{
    return array_map([ImmediateCallbackMethods::class, 'mightFail'], [1]);
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_array_map_method_callable_caller(): array
{
    return array_map([ImmediateCallbackMethods::class, 'mightFail'], [1]);
}

/**
 * @throws ImmediateCallbackFailure
 */
function array_walk_recursive_caller(array $values): void
{
    array_walk_recursive($values, static function (mixed &$value, int|string $key): void {
        might_fail();
    });
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function unhandled_array_walk_recursive_caller(array $values): void
{
    array_walk_recursive($values, static function (mixed &$value, int|string $key): void {
        might_fail();
    });
}

function callback_catches_failure(): array
{
    return array_map(
        static function (int $value): int {
            try {
                might_fail();
            } catch (ImmediateCallbackFailure) {
                return $value;
            }

            return $value;
        },
        [1],
    );
}

function uppercase_array_udiff_callable_array_operand_is_not_callback(): array
{
    return ARRAY_UDIFF(
        [1],
        [ImmediateCallbackMethods::class, 'mightFail'],
        static fn (mixed $left, mixed $right): int => 0,
    );
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:always-matching-switch-case,unreachable-switch-default
 */
function caught_callback_exception_does_not_escape_switch_break(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        switch (1) {
            case 1:
                break;
            default:
                throw $exception;
        }
    }
}

function caught_callback_exception_switch_break_caller(): void
{
    caught_callback_exception_does_not_escape_switch_break(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:always-matching-switch-case,unreachable-switch-case
 */
function caught_callback_exception_does_not_escape_try_switch_break(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        switch (1) {
            case 1:
                try {
                    break;
                } finally {
                }
            case 2:
                throw $exception;
        }
    }
}

function caught_callback_exception_try_switch_break_caller(): void
{
    caught_callback_exception_does_not_escape_try_switch_break(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:always-matching-switch-case,unreachable-switch-case
 */
function caught_callback_exception_does_not_escape_switch_goto(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        switch (1) {
            case 1:
                goto swallowed_callback_exception;
            case 2:
                throw $exception;
        }

        swallowed_callback_exception:
        return;
    }
}

function caught_callback_exception_switch_goto_caller(): void
{
    caught_callback_exception_does_not_escape_switch_goto(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function caught_callback_exception_does_not_escape_switch_fallthrough(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        switch (1) {
            case 1:
            default:
                return;
        }

        throw $exception; // @mago-expect analysis:unevaluated-code
    }
}

function caught_callback_exception_switch_fallthrough_caller(): void
{
    caught_callback_exception_does_not_escape_switch_fallthrough(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unhandled-thrown-type,unevaluated-code
 */
function conditionally_preserve_callback_exception_in_finally(callable $callback, bool $skip_throw): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        if ($skip_throw) {
            goto after_finally_throw;
        }

        throw new SwallowedCallbackFailure();
        after_finally_throw:
    }
}

/**
 * @throws SwallowedCallbackFailure
 * @mago-expect analysis:unhandled-thrown-type
 */
function conditionally_preserve_callback_exception_in_finally_caller(): void
{
    conditionally_preserve_callback_exception_in_finally(static fn (): int => might_fail(), true);
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function finally_switch_goto_overrides_callback_throw(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        switch (1) {
            case 1:
                goto replace_callback_throw;
        }

        replace_callback_throw:
        throw new SwallowedCallbackFailure();
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function finally_switch_goto_overrides_callback_throw_caller(): void
{
    finally_switch_goto_overrides_callback_throw(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function finally_return_after_nonthrowing_try_overrides_callback_throw(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        try {
            $callback = null;
            return;
        } catch (Throwable) { // @mago-expect analysis:dead-catch
        }
    }
}

function finally_return_after_nonthrowing_try_overrides_callback_throw_caller(): void
{
    finally_return_after_nonthrowing_try_overrides_callback_throw(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function finally_without_catch_return_overrides_callback_throw(callable $callback): void
{
    try {
        $callback();
    } finally {
        return;
    }
}

function finally_without_catch_return_overrides_callback_throw_caller(): void
{
    finally_without_catch_return_overrides_callback_throw(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function finally_throw_overrides_unmatched_callback_exception(callable $callback): void
{
    try {
        $callback();
    } catch (InvalidArgumentException $exception) {
        throw $exception;
    } finally {
        throw new SwallowedCallbackFailure();
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function finally_throw_overrides_unmatched_callback_exception_caller(): void
{
    finally_throw_overrides_unmatched_callback_exception(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @throws SwallowedCallbackFailure
 */
function finally_nested_switch_break_two_overrides_callback_throw(
    callable $callback,
    int $outer,
    int $inner,
): void {
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        switch ($outer) {
            case 0:
                switch ($inner) {
                    case 0:
                        break 2;
                    default:
                        break 2;
                }
            default:
                break;
        }

        throw new SwallowedCallbackFailure();
    }
}

/**
 * @throws SwallowedCallbackFailure
 */
function finally_nested_switch_break_two_overrides_callback_throw_caller(): void
{
    finally_nested_switch_break_two_overrides_callback_throw(static fn (): int => might_fail(), 0, 0);
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:always-matching-switch-case,unreachable-switch-default
 */
function finally_break_two_exits_outer_switch_overrides_callback_throw(callable $callback): void
{
    switch (1) {
        case 1:
            try {
                $callback();
            } catch (RuntimeException $exception) {
                throw $exception;
            } finally {
                switch (1) {
                    case 1:
                        break 2;
                    default:
                        break 2;
                }
            }
    }

}

function finally_break_two_exits_outer_switch_overrides_callback_throw_caller(): void
{
    finally_break_two_exits_outer_switch_overrides_callback_throw(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function inner_finally_catch_after_return_is_unreachable(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        throw $exception;
    } finally {
        try {
            return;
        } catch (Throwable) { // @mago-expect analysis:dead-catch
        }
    }
}

function inner_finally_catch_after_return_caller(): void
{
    inner_finally_catch_after_return_is_unreachable(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:unhandled-thrown-type
 */
function caught_callback_exception_is_not_rethrown_by_unreached_case_expression(callable $callback): void
{
    try {
        $callback();
    } catch (RuntimeException $exception) {
        switch (1) {
            case 1:
            case throw $exception:
                return;
        }
    }
}

function caught_callback_exception_unreached_case_expression_caller(): void
{
    caught_callback_exception_is_not_rethrown_by_unreached_case_expression(static fn (): int => might_fail());
}
