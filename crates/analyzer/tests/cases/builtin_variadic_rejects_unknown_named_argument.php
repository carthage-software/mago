<?php

declare(strict_types=1);

function user_variadic_collects(int ...$values): int
{
    return count($values);
}

function builtin_variadic_named(): void
{
    // @mago-expect analysis:invalid-named-argument
    $_ = max([1, 2], foo: 3);
    // @mago-expect analysis:invalid-named-argument
    echo sprintf('%s', 'a', foo: 'b');
    echo user_variadic_collects(1, foo: 2);
}
