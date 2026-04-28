<?php

declare(strict_types=1);

/**
 * @param list<string> $expected
 */
function callables_takes_list_no_named(array $expected): void
{
    foreach ($expected as $s) {
        echo $s;
    }
}

/**
 * @no-named-arguments
 */
function callables_variadic_no_named(string ...$args): void
{
    callables_takes_list_no_named($args);
}

callables_variadic_no_named('a', 'b');
