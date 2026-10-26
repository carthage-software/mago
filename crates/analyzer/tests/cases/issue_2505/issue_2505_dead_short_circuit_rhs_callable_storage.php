<?php

final class Issue2505DeadShortCircuitFailure extends Exception
{
}

final class Issue2505DeadShortCircuitCallback
{
    /** @throws Issue2505DeadShortCircuitFailure */
    public static function stored(): void
    {
        throw new Issue2505DeadShortCircuitFailure();
    }
}

function false_and_rhs_does_not_store_callable(): void
{
    $callback = null;
    false && ($callback = Issue2505DeadShortCircuitCallback::stored(...));
}

function false_and_dead_unknown_callable_does_not_report(): void
{
    if (function_exists('unknown_callable_dispatcher')) {
        false && unknown_callable_dispatcher(static function (): void {
            throw new Issue2505DeadShortCircuitFailure();
        });
    }
}

function true_or_rhs_does_not_store_callable(): void
{
    $callback = null;
    true || ($callback = Issue2505DeadShortCircuitCallback::stored(...));
}

function non_null_coalesce_rhs_does_not_store_callable(): void
{
    $callback = null;
    1 ?? ($callback = Issue2505DeadShortCircuitCallback::stored(...));
}
