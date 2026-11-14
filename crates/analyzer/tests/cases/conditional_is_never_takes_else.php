<?php

declare(strict_types=1);

/**
 * @template T
 * @param T $t
 * @return (T is never ? 'e' : T)
 */
function is_never(mixed $t): mixed
{
    return $t;
}

/**
 * @template T
 * @param T $t
 * @return (T is not never ? T : 'e')
 */
function is_not_never(mixed $t): mixed
{
    return $t;
}

function a(int $i): int
{
    return is_never($i);
}

function b(int $i): int
{
    return is_not_never($i);
}
