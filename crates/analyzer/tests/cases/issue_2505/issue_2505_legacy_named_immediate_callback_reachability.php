<?php

/**
 * @param-immediately-invoked-callable $callback
 */
function invoke_immediately_with_callable_argument(callable $callback, callable $argument): void
{
    $callback($argument);
}

function catch_unknown_exception_from_named_callbacks(callable $dynamicCallable): void
{
    try {
        function invoke_dynamic_callable_from_named_function(callable $callback): void
        {
            $callback();
        }

        invoke_immediately_with_callable_argument(
            invoke_dynamic_callable_from_named_function(...),
            $dynamicCallable,
        );
    } catch (Throwable) {
    }

    try {
        class NamedImmediateCallback
        {
            public static function invokeDynamicCallable(callable $callback): void
            {
                $callback();
            }
        }

        invoke_immediately_with_callable_argument(
            NamedImmediateCallback::invokeDynamicCallable(...),
            $dynamicCallable,
        );
    } catch (Throwable) {
    }
}

catch_unknown_exception_from_named_callbacks(static fn (): null => null);
