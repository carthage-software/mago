<?php

/** @throws RuntimeException */
function throw_callback_runtime_exception(): int
{
    throw new RuntimeException();
}

/** @throws LogicException */
function throw_callback_logic_exception(): void
{
    throw new LogicException();
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_callback_with_reachability_sensitive_catches(callable $callback): void
{
    try {
        if (false) {
            $callback();
        }
    } catch (LogicException) {
    }

    try {
        $callback();
    } catch (RuntimeException) {
    }
}

/**
 * @param callable(): void $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_callback_under_unknown_condition(callable $callback, bool $condition): void
{
    try {
        if ($condition) {
            $callback();
        }
    } catch (LogicException) {
    }
}

function dead_callback_invocation_does_not_leak_catch_summary(): void
{
    invoke_callback_with_reachability_sensitive_catches(static function (): int {
        return throw_callback_runtime_exception();
    });
}

function unknown_condition_callback_invocation_keeps_catch_summary(bool $condition): void
{
    invoke_callback_under_unknown_condition(static function (): void {
        throw_callback_logic_exception();
    }, $condition);
}
