<?php

declare(strict_types=1);

/** @return array{0: string, 1: string}|null */
function m(string $s): ?array
{
    return '' === $s ? null : [$s, $s];
}

function take(string $s): void
{
}

function subject(string $method): void
{
    $matches = m($method);
    if (null !== $matches) {
        [, $t] = $matches;
        take($t);
    } elseif (null !== ($matches = m($method))) {
        [, $u] = $matches;
        take($u);
    }
}

function freshVariable(string $method): void
{
    if ('x' === $method) {
        return;
    } elseif (null !== ($matches = m($method))) {
        [, $value] = $matches;
        take($value);
    }
}

function plainIf(string $method): void
{
    $matches = null;
    if (null !== ($matches = m($method))) {
        [, $value] = $matches;
        take($value);
    }
}
