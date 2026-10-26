<?php

declare(strict_types=1);

namespace Issue2504\NamespacedShadowing;

final class ImmediateCallbackFailure extends \RuntimeException
{
}

/**
 * @throws ImmediateCallbackFailure
 */
function might_fail(): int
{
    throw new ImmediateCallbackFailure();
}

function call_user_func(callable $callback): void
{
}

function call_user_func_array(callable $callback, array $arguments): void
{
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
 * @mago-expect analysis:unhandled-thrown-type
 */
function call_user_func_caller(): void
{
    invoke_and_catch_via_call_user_func(static fn (): int => might_fail());
}

/**
 * @param callable(): int $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_and_catch_via_call_user_func_array(callable $callback): void
{
    try {
        call_user_func_array($callback, []);
    } catch (ImmediateCallbackFailure) {
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function call_user_func_array_caller(): void
{
    invoke_and_catch_via_call_user_func_array(static fn (): int => might_fail());
}
