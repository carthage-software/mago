<?php

function local_closure_with_dynamic_throw_is_scanned(callable $dynamicCallable): void
{
    try {
        $callback = /** @throws RuntimeException */ static function () use ($dynamicCallable): void {
            $dynamicCallable();
        };

        $callback();
    } catch (LogicException) {
    } catch (RuntimeException) {
    }
}

function local_function_with_dynamic_throw_is_scanned(callable $dynamicCallable): void
{
    try {
        /** @throws RuntimeException */
        function issue_2505_local_named_callback(callable $dynamicCallable): void
        {
            $dynamicCallable();
        }

        $callback = issue_2505_local_named_callback(...);
        $callback($dynamicCallable);
    } catch (LogicException) {
    } catch (RuntimeException) {
    }
}

function local_method_with_dynamic_throw_is_scanned(callable $dynamicCallable): void
{
    try {
        class Issue2505LocalCallback
        {
            /** @throws RuntimeException */
            public static function invoke(callable $dynamicCallable): void
            {
                $dynamicCallable();
            }
        }

        $callback = Issue2505LocalCallback::invoke(...);
        $callback($dynamicCallable);
    } catch (LogicException) {
    } catch (RuntimeException) {
    }
}
