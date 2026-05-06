<?php

/**
 * @psalm-assert-if-true bool $v
 */
function is_true_or_false(mixed $v): bool
{
    return is_true_or_false($v);
}

/**
 * @param array{'x': mixed} $data
 */
function foo(array $data): int
{
    if (!is_true_or_false($data['x'])) {
        return 2;
    }

    return bar($data);
}

/**
 * @param array{'x': bool} $data
 */
function bar(array $data): int
{
    return $data['x'] ? 3 : 4;
}
