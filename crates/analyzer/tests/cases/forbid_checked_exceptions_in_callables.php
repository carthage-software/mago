<?php

final class DeferredFailure extends Exception
{
}

final class DeferredCatchHandlerFailure extends Exception
{
}

/**
 * @throws DeferredFailure
 */
function fail_deferred(): void
{
    throw new DeferredFailure();
}

/**
 * @throws LogicException
 */
function fail_unchecked(): void
{
    throw new LogicException();
}

final class TrustedDispatcher
{
    /**
     * @param Closure(): void $callback
     * @param-later-invoked-callable $callback
     */
    public static function consume(Closure $callback): void
    {
    }
}

/**
 * @param Closure(): void $callback
 * @param-later-invoked-callable $callback
 */
function defer_callback(Closure $callback): void
{
}

/**
 * @param callable(): void $callback
 * @param-later-invoked-callable $callback
 */
function defer_named_callable(callable $callback): void
{
}

/** @throws DeferredFailure */
function named_deferred_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function named_string_deferred_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function stored_only_named_callback(): void
{
    fail_deferred();
}

/** @throws DeferredFailure */
function stored_only_string_callback(): void
{
    fail_deferred();
}

/** @throws DeferredFailure */
function stored_and_directly_called_callback(): void
{
    fail_deferred();
}

/** @throws DeferredFailure */
function stored_and_caught_named_callback(): void
{
    fail_deferred();
}

/** @throws DeferredFailure */
function named_callback_with_uninvoked_alias(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function reassigned_named_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function conditionally_reassigned_named_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function guaranteed_reassignment_callback(): void
{
    fail_deferred();
}

/** @throws DeferredFailure */
function try_catch_reassignment_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_on_conditional_break(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

function conditional_break_does_not_clear_stored_callback(): void
{
    static $callback;
    $callback ??= callback_retained_on_conditional_break(...);

    while (true) {
        if (random_int(0, 1)) {
            break;
        }

        $callback = static function (): void {};
        break;
    }
}

/** @throws DeferredFailure */
function callback_cleared_after_unreachable_if(): void
{
    fail_deferred();
}

/** @throws DeferredFailure */
function callback_retained_after_conditional_return(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_throwing_condition(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_mixed_throw_condition(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_throwing_elseif(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_null_coalesce_condition(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_array_access_condition(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_clone_condition(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_static_null_coalesce_condition(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function callback_retained_after_immediate_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function maybe_throw_condition(): bool
{
    if (random_int(0, 1)) {
        throw new DeferredFailure();
    }

    return true;
}

/** @throws RuntimeException */
function maybe_throw_runtime_condition(): bool
{
    if (random_int(0, 1)) {
        throw new RuntimeException();
    }

    return true;
}

function unannotated_condition_helper(): bool
{
    return false;
}

function unreachable_if_does_not_retain_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    if (false) {
        return;
    }

    $callback = static function (): void {};
}

function unreachable_nested_if_does_not_retain_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    while (true) {
        if (random_int(0, 1)) {
            if (false) {
                break;
            }
        }

        $callback = static function (): void {};
        break;
    }
}

function unreachable_if_in_try_prefix_does_not_retain_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    try {
        if (false) {
            return;
        }

        $value = 1;
        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function unreachable_nested_if_in_try_does_not_retain_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    while (true) {
        if (random_int(0, 1)) {
            try {
                if (false) {
                    return;
                }
            } catch (Throwable) {
            }
        }

        $callback = static function (): void {};
        break;
    }
}

function unreachable_nested_if_in_finally_does_not_retain_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    while (true) {
        if (random_int(0, 1)) {
            try {
                if (false) {
                    return;
                }
            } finally {
            }
        }

        $callback = static function (): void {};
        break;
    }
}

function caught_throw_does_not_retain_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    if (random_int(0, 1)) {
        try {
            throw new RuntimeException();
        } catch (RuntimeException) {
        }
    }

    $callback = static function (): void {};
}

function conditional_return_before_final_throw_retains_callback(): void
{
    static $callback;
    $callback ??= callback_retained_after_conditional_return(...);

    try {
        if (random_int(0, 1)) {
            return;
        }

        $value = 1;
        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function conditional_caught_throw_before_final_throw_clears_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    try {
        if (maybe_throw_runtime_condition() === true) {
        }

        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function dead_untracked_try_condition_clears_callback(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    try {
        if (false && unannotated_condition_helper()) {
        }

        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function deferred_closure_body_does_not_block_guaranteed_catch(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    try {
        $deferred = static function (): void {
            unannotated_condition_helper();
        };
        $deferredArrow = static fn (): bool => unannotated_condition_helper();

        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function deferred_named_bodies_do_not_block_guaranteed_catch(): void
{
    static $callback;
    $callback ??= callback_cleared_after_unreachable_if(...);

    try {
        function deferred_named_function_for_try(): void
        {
            unannotated_condition_helper();
        }

        class DeferredTryMethodContainer
        {
            public function run(): void
            {
                unannotated_condition_helper();
            }
        }

        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function null_coalesce_dead_throw_does_not_clear_callback(): void
{
    static $callback;
    $callback ??= callback_retained_after_null_coalesce_condition(...);

    try {
        if (true ?? throw new RuntimeException()) {
            return;
        }
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function static_local_null_coalesce_throw_retains_callback(): void
{
    static $callback;
    static $flag = true;
    $callback ??= callback_retained_after_static_null_coalesce_condition(...);

    try {
        if ($flag ?? throw new RuntimeException()) {
            $callback = static function (): void {};
            return;
        }

        $callback = static function (): void {};
    } catch (RuntimeException) {
        return;
    }
}

function throwing_array_access_condition_retains_callback(ArrayAccess $value): void
{
    static $callback;
    $callback ??= callback_retained_after_array_access_condition(...);

    try {
        if ($value[0]) {
        }

        $callback = static function (): void {};
    } catch (Throwable) {
        return;
    }
}

function throwing_clone_condition_retains_callback(object $value): void
{
    static $callback;
    $callback ??= callback_retained_after_clone_condition(...);

    try {
        if (clone $value) {
        }

        $callback = static function (): void {};
    } catch (Throwable) {
        return;
    }
}

function immediate_callback_throw_keeps_stored_callback(callable $dynamicCallable): void
{
    static $callback;
    $callback ??= callback_retained_after_immediate_callback(...);

    try {
        function immediate_named_function_for_try(): void
        {
            unannotated_condition_helper();
        }

        $storedClosure = static function (mixed $value) use ($dynamicCallable): void {
            $dynamicCallable($value);
        };
        $storedArrow = static fn (mixed $value): mixed => $dynamicCallable($value);
        array_map($storedClosure, [1]);
        array_map($storedArrow, [1]);

        array_map(static function (mixed $value) use ($dynamicCallable): void {
            $dynamicCallable($value);
        }, [1]);
        array_map(static fn (mixed $value): mixed => $dynamicCallable($value), [1]);

        $methodCallbackSource = new class {
            public function run(): void
            {
                unannotated_condition_helper();
            }
        };
        array_map(immediate_named_function_for_try(...), [1]);
        array_map($methodCallbackSource->run(...), [1]);

        throw new RuntimeException();
    } catch (Throwable) {
        $callback = static function (): void {};
    }
}

function dead_immediate_callback_callsite_keeps_catch_reachable(callable $dynamicCallable): void
{
    static $callback;
    $callback ??= callback_retained_after_immediate_callback(...);
    $storedClosure = static function (mixed $value) use ($dynamicCallable): mixed {
        return $dynamicCallable($value);
    };
    $storedArrow = static fn (mixed $value): mixed => $dynamicCallable($value);

    try {
        if (false) {
            array_map($storedClosure, [1]);
        }

        false && array_map($storedArrow, [1]);
        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

function unreachable_nested_catch_does_not_keep_callback(callable $dynamicCallable): void
{
    static $callback;
    $callback ??= callback_retained_after_immediate_callback(...);
    $storedClosure = static fn (mixed $value): mixed => $dynamicCallable($value);

    try {
        try {
        } catch (Throwable) {
            array_map($storedClosure, [1]);
        }

        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

/** @throws DeferredFailure */
function throwing_condition_does_not_clear_callback(): void
{
    static $callback;
    $callback ??= callback_retained_after_throwing_condition(...);

    if (maybe_throw_condition()) {
    }

    $callback = static function (): void {};
}

/** @throws DeferredFailure */
function mixed_throwing_condition_does_not_clear_callback(): void
{
    static $callback;
    $callback ??= callback_retained_after_mixed_throw_condition(...);

    try {
        if (maybe_throw_condition()) {
        }

        throw new RuntimeException();
    } catch (RuntimeException) {
        $callback = static function (): void {};
    }
}

/** @throws DeferredFailure */
function throwing_elseif_does_not_clear_callback(): void
{
    static $callback;
    $callback ??= callback_retained_after_throwing_elseif(...);

    if (false) {
    } elseif (maybe_throw_condition()) {
    }

    $callback = static function (): void {};
}

/**
 * @throws DeferredFailure
 * @return Closure(): void
 */
function maybe_throwing_callback_factory(): Closure
{
    fail_deferred();

    return static function (): void {};
}

/** @throws DeferredFailure */
function array_stored_named_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function static_stored_named_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function unresolved_dispatch_named_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function immediate_only_named_callback(): void
{
    fail_deferred();
}

/** @throws DeferredFailure */
function array_union_existing_named_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

/** @throws DeferredFailure */
function array_union_added_named_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

final class DeferredNamedCallbackSources
{
    /** @throws DeferredFailure */
    public static function static_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public static function static_array_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public static function stored_only_method_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public static function immediate_alias_method_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public static function unresolved_target_method_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public static function stored_only_first_class_method_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public static function stored_only_method_string_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public static function array_union_added_method_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public function instance_callback(): void
    {
        fail_deferred();
    }

    /** @throws DeferredFailure */
    public function instance_array_callback(): void
    {
        fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
    }

    /** @throws DeferredFailure */
    public function stored_only_instance_array_callback(): void
    {
        fail_deferred();
    }
}

/**
 * @param Closure(): void $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_immediately(Closure $callback): void
{
    $callback();
}

function invoke_unresolved_dispatcher(callable $dispatcher): void
{
    $dispatcher(/** @throws DeferredFailure */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        });
    $dispatcher(unresolved_dispatch_named_callback(...));

    if (function_exists('unknown_callback_dispatcher')) {
        unknown_callback_dispatcher(/** @throws DeferredFailure */
            function (): void {
                fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
            });
        unknown_callback_dispatcher(unresolved_dispatch_named_callback(...));
        unknown_callback_dispatcher(DeferredNamedCallbackSources::unresolved_target_method_callback(...));
    }
}

/**
 * @param callable(): void $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_named_immediately(callable $callback): void
{
    $callback();
}

/**
 * @param Closure(): void $callback
 * @param-immediately-invoked-callable $callback
 * @throws DeferredCatchHandlerFailure
 * @mago-expect analysis:invalid-catch-type
 * @mago-expect analysis:no-valid-catch-type-found
 */
function invoke_with_unresolved_catch_metadata_fallback(Closure $callback): void
{
    try {
        $callback();
    } catch (DeferredFailure) {
        throw new DeferredCatchHandlerFailure();
    }

    try {
        throw new RuntimeException();
    } catch (self) {
    }
}

/**
 * @param Closure(): void $callback
 * @param-immediately-invoked-callable $callback
 * @mago-expect analysis:invalid-catch-type
 * @mago-expect analysis:no-valid-catch-type-found
 */
function invoke_callback_inside_unresolved_catch(Closure $callback): void
{
    try {
        $callback();
    } catch (self) {
    }
}

/**
 * @param Closure(): void $callback
 * @param-immediately-invoked-callable $callback
 * @throws DeferredCatchHandlerFailure
 */
function swallow_callback_when_try_has_unrelated_throw(Closure $callback): void
{
    try {
        $callback();
        throw new DeferredCatchHandlerFailure();
    } catch (DeferredFailure) {
    }
}

/**
 * @param Closure(): void $callback
 */
function unknown_callback(Closure $callback): void
{
}

/**
 * @param Closure(): void $safe
 * @param Closure(): void $checked
 * @param-later-invoked-callable $safe
 * @param-later-invoked-callable $checked
 */
function trusted_callbacks(Closure $safe, Closure $checked): void
{
}

function exercise_deferred_callbacks(?Closure $coalesce_assigned_callback): void
{
    $callbacks = [];

    $coalesce_assigned_callback ??= /** @throws DeferredFailure */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        };

    $stored_named_callback = stored_only_named_callback(...);
    $stored_and_directly_called = stored_and_directly_called_callback(...);
    $directly_called = stored_and_directly_called_callback(...);
    try {
        $directly_called();
    } catch (DeferredFailure) {
    }
    $stored_and_caught = stored_and_caught_named_callback(...);
    try {
        $stored_and_caught();
    } catch (DeferredFailure) {
    }
    $unused_named_alias = named_callback_with_uninvoked_alias(...);
    $immediate_named_alias = named_callback_with_uninvoked_alias(...);
    try {
        invoke_named_immediately($immediate_named_alias);
    } catch (DeferredFailure) {
    }

    $callbacks['unused'] = array_stored_named_callback(...);
    $callbacks['immediate'] = array_stored_named_callback(...);
    try {
        invoke_named_immediately($callbacks['immediate']);
    } catch (DeferredFailure) {
    }

    $unused_method_alias = DeferredNamedCallbackSources::immediate_alias_method_callback(...);
    $immediate_method_alias = DeferredNamedCallbackSources::immediate_alias_method_callback(...);
    try {
        invoke_named_immediately($immediate_method_alias);
    } catch (DeferredFailure) {
    }

    $reassigned_named_callback = reassigned_named_callback(...);
    try {
        $reassigned_named_callback();
    } catch (DeferredFailure) {
    }
    $reassigned_named_callback = reassigned_named_callback(...);

    $conditionally_reassigned_named_callback = conditionally_reassigned_named_callback(...);
    if (random_int(0, 1) === 1) {
        $conditionally_reassigned_named_callback = static function (): void {};
    }

    $guaranteed_if_callback = guaranteed_reassignment_callback(...);
    if (true) {
        $guaranteed_if_callback = static function (): void {};
    }

    $guaranteed_ternary_callback = guaranteed_reassignment_callback(...);
    true ? ($guaranteed_ternary_callback = static function (): void {}) : null;

    $guaranteed_match_callback = guaranteed_reassignment_callback(...);
    $match_result = match (true) {
        true => $guaranteed_match_callback = static function (): void {},
        default => null,
    };

    $guaranteed_while_callback = guaranteed_reassignment_callback(...);
    while (true) {
        $guaranteed_while_callback = static function (): void {};
        break;
    }

    $guaranteed_for_callback = guaranteed_reassignment_callback(...);
    for (;;) {
        $guaranteed_for_callback = static function (): void {};
        break;
    }

    $guaranteed_catch_callback = guaranteed_reassignment_callback(...);
    try {
        throw new RuntimeException();
    } catch (RuntimeException) {
        $guaranteed_catch_callback = static function (): void {};
    }

    $guaranteed_prefixed_catch_callback = guaranteed_reassignment_callback(...);
    try {
        $value = 1;
        throw new RuntimeException();
    } catch (RuntimeException) {
        $guaranteed_prefixed_catch_callback = static function (): void {};
    }

    $try_catch_callback = try_catch_reassignment_callback(...);
    try {
        fail_deferred();
        $try_catch_callback = static function (): void {};
    } catch (DeferredFailure) {
    }

    $rhs_throw_callback = try_catch_reassignment_callback(...);
    try {
        $rhs_throw_callback = maybe_throwing_callback_factory();
    } catch (DeferredFailure) {
    }

    conditional_break_does_not_clear_stored_callback();
    unreachable_if_does_not_retain_callback();
    unreachable_nested_if_does_not_retain_callback();
    unreachable_if_in_try_prefix_does_not_retain_callback();
    unreachable_nested_if_in_try_does_not_retain_callback();
    unreachable_nested_if_in_finally_does_not_retain_callback();
    caught_throw_does_not_retain_callback();
    conditional_caught_throw_before_final_throw_clears_callback();
    dead_untracked_try_condition_clears_callback();
    deferred_closure_body_does_not_block_guaranteed_catch();
    deferred_named_bodies_do_not_block_guaranteed_catch();
    null_coalesce_dead_throw_does_not_clear_callback();
    static_local_null_coalesce_throw_retains_callback();
    throwing_array_access_condition_retains_callback(new ArrayObject());
    throwing_clone_condition_retains_callback(new stdClass());
    immediate_callback_throw_keeps_stored_callback(static fn (mixed $value): mixed => $value);
    dead_immediate_callback_callsite_keeps_catch_reachable(static fn (mixed $value): mixed => $value);
    unreachable_nested_catch_does_not_keep_callback(static fn (mixed $value): mixed => $value);
    conditional_return_before_final_throw_retains_callback();
    throwing_condition_does_not_clear_callback();
    mixed_throwing_condition_does_not_clear_callback();
    throwing_elseif_does_not_clear_callback();

    $immediate_closure_alias = /** @throws DeferredFailure */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        };
    $unused_closure_alias = $immediate_closure_alias;
    try {
        invoke_immediately($immediate_closure_alias);
    } catch (DeferredFailure) {
    }

    $stored_named_method_callback = [DeferredNamedCallbackSources::class, 'stored_only_method_callback'];
    $stored_string_callback = 'stored_only_string_callback';
    $stored_method_string_callback = 'DeferredNamedCallbackSources::stored_only_method_string_callback';
    $stored_first_class_method_callback = DeferredNamedCallbackSources::stored_only_first_class_method_callback(...);
    $immediate_only = immediate_only_named_callback(...);
    try {
        invoke_named_immediately($immediate_only);
    } catch (DeferredFailure) {
    }

    $parenthesized_immediate = immediate_only_named_callback(...);
    try {
        ($parenthesized_immediate)();
    } catch (DeferredFailure) {
    }

    defer_named_callable(named_deferred_callback(...));
    defer_named_callable(DeferredNamedCallbackSources::static_callback(...));
    $callback_source = new DeferredNamedCallbackSources();
    $stored_instance_array_method_callback = [$callback_source, 'stored_only_instance_array_callback'];
    defer_named_callable($callback_source->instance_callback(...));
    defer_named_callable('named_string_deferred_callback');
    defer_named_callable([DeferredNamedCallbackSources::class, 'static_array_callback']);
    defer_named_callable([$callback_source, 'instance_array_callback']);

    $stored = /**
             * @throws DeferredFailure
             */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        };
    defer_callback($stored);

    $stored_arrow = /** @throws DeferredFailure */
        fn () => fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
    defer_callback($stored_arrow);

    defer_callback(/**
                   * @throws DeferredFailure
                   */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        });

    defer_callback(
        /** @throws DeferredFailure */
        fn () => fail_deferred(), // @mago-expect analysis:unhandled-thrown-type
    );

    unknown_callback(/**
                     * @throws DeferredFailure
                     */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        });

    unknown_callback(function (): void {
        fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
    });

    $caught = function (): void {
        try {
            fail_deferred();
        } catch (DeferredFailure) {
        }
    };
    defer_callback($caught);

    $unchecked = function (): void {
        fail_unchecked();
    };
    defer_callback($unchecked);

    trusted_callbacks(
        /** @throws DeferredFailure */
        function (): void {
            fail_deferred();
        },
        /** @throws DeferredFailure */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        },
    );

    trusted_callbacks(
        safe: (/** @throws DeferredFailure */ function (): void {
            fail_deferred();
        }),
        checked: $caught,
    );

    TrustedDispatcher::consume(callback: (/** @throws DeferredFailure */ fn () => fail_deferred()));

    $stored_allowed = /** @throws DeferredFailure */
        function (): void {
            fail_deferred();
        };
    TrustedDispatcher::consume(callback: $stored_allowed);

    TrustedDispatcher::consume(
        callback: /**
                   * @throws DeferredFailure
                   */
            function (): void {
                fail_deferred();
            },
    );

    (/** @throws DeferredFailure */ fn () => fail_deferred());
}

/** @throws DeferredFailure */
function invoke_stored_callable_immediately(): void
{
    $callback = /** @throws DeferredFailure */
        function (): void {
            fail_deferred();
        };
    $callback();
}

/**
 * @throws DeferredFailure
 * @return Closure(): void
 */
function invoke_and_return_stored_callable(): Closure
{
    $callback = /** @throws DeferredFailure */
        function (): void {
            fail_deferred();
        };
    $callback();

    return $callback;
}

/** @throws DeferredFailure */
function parenthesized_immediate_callbacks(): void
{
    invoke_immediately((/** @throws DeferredFailure */ function (): void {
        fail_deferred();
    }));

    invoke_immediately((/** @throws DeferredFailure */ fn () => fail_deferred()));

    $stored = /** @throws DeferredFailure */
        function (): void {
            fail_deferred();
        };
    invoke_immediately($stored);

    $stored_arrow = /** @throws DeferredFailure */ fn () => fail_deferred();
    invoke_immediately($stored_arrow);
}

function directly_invoked_stored_callable(): void
{
    $stored = /** @throws DeferredFailure */ fn () => fail_deferred();

    try {
        $stored();
    } catch (DeferredFailure) {
    }

    $parenthesized = /** @throws DeferredFailure */ fn () => fail_deferred();
    try {
        ($parenthesized)();
    } catch (DeferredFailure) {
    }
}

function array_union_stored_callables(): void
{
    $callbacks = ['existing' => array_union_existing_named_callback(...)];
    $callbacks += [
        'named' => array_union_added_named_callback(...),
        'method' => DeferredNamedCallbackSources::array_union_added_method_callback(...),
        'closure' => /** @throws DeferredFailure */ static function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        },
    ];
}

function array_stored_closure_aliases(): void
{
    $callbacks = [];

    $callbacks['unused'] = /** @throws DeferredFailure */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        };
    $callbacks['immediate'] = $callbacks['unused'];

    try {
        invoke_immediately($callbacks['immediate']);
    } catch (DeferredFailure) {
    }
}

function invoke_static_stored_named_callback(): void
{
    static $callback;
    $callback = static_stored_named_callback(...);

    try {
        invoke_named_immediately($callback);
    } catch (DeferredFailure) {
    }
}

function immediate_stored_closure_propagates_to_undocumented_caller(): void
{
    $stored = /** @throws DeferredFailure */
        function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        };

    invoke_immediately($stored);
}

function immediate_stored_arrow_propagates_to_undocumented_caller(): void
{
    $stored = /** @throws DeferredFailure */ fn () => fail_deferred(); // @mago-expect analysis:unhandled-thrown-type

    invoke_immediately($stored);
}

/**
 * @throws DeferredCatchHandlerFailure
 */
function unresolved_catch_metadata_fallback_preserves_unrelated_rethrow(): void
{
    invoke_with_unresolved_catch_metadata_fallback(
        /** @throws DeferredFailure */
        static function (): void {
            fail_deferred();
        },
    );
}

/**
 */
function unresolved_catch_does_not_swallow_callback_exception(): void
{
    invoke_callback_inside_unresolved_catch(
        /** @throws DeferredFailure */
        static function (): void {
            fail_deferred();
        },
    );
}

/** @throws DeferredCatchHandlerFailure */
function unrelated_try_throw_does_not_escape_empty_callback_catch(): void
{
    swallow_callback_when_try_has_unrelated_throw(
        /** @throws DeferredFailure */
        static function (): void {
            fail_deferred();
        },
    );
}

exercise_deferred_callbacks(null);
array_union_stored_callables();
directly_invoked_stored_callable();
array_stored_closure_aliases();
invoke_static_stored_named_callback();
immediate_stored_closure_propagates_to_undocumented_caller();
immediate_stored_arrow_propagates_to_undocumented_caller();
unresolved_catch_metadata_fallback_preserves_unrelated_rethrow();
unresolved_catch_does_not_swallow_callback_exception();
unrelated_try_throw_does_not_escape_empty_callback_catch();

try {
    parenthesized_immediate_callbacks();
} catch (DeferredFailure) {
}
