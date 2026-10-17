<?php

/**
 * @param-immediately-invoked-callable $callback
 * @throws LogicException
 */
function issue_2505_invoke_nested_callback(callable $callback): void
{
    $callback();
}

function nested_immediate_callbacks_keep_runtime_catch_reachable(callable $dynamicCallable): void
{
    try {
        issue_2505_invoke_nested_callback(static function () use ($dynamicCallable): void {
            issue_2505_invoke_nested_callback(static function () use ($dynamicCallable): void {
                $dynamicCallable();
            });
        });

        throw new LogicException();
    } catch (LogicException) {
    } catch (RuntimeException) {
    }
}
