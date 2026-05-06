<?php

declare(strict_types=1);

/**
 * @param list<string> $expected
 */
function callables_takes_list_strict(array $expected): void
{
    foreach ($expected as $s) {
        echo $s;
    }
}

function callables_variadic_strict_default(string ...$args): void
{
    callables_takes_list_strict($args);
}

callables_variadic_strict_default('a', 'b');
