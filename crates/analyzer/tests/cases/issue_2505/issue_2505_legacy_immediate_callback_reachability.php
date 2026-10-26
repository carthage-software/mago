<?php

/**
 * @param Closure(): void $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_immediately_for_catch_reachability(Closure $callback): void
{
    $callback();
}

function catch_unknown_exception_from_immediate_callback(callable $dynamicCallable): void
{
    try {
        invoke_immediately_for_catch_reachability(static function () use ($dynamicCallable): void {
            $dynamicCallable();
        });
    } catch (Throwable) {
    }
}

catch_unknown_exception_from_immediate_callback(static fn (): null => null);
