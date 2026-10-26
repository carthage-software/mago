<?php

function dead_named_callable_body_does_not_keep_catch_reachable(): void
{
    try {
        function issue_2505_recursive_named_callable(callable $dynamicCallable): void
        {
            array_map(issue_2505_recursive_named_callable(...), [$dynamicCallable]);
            $dynamicCallable();
        }
    } catch (LogicException) {
    }
}
