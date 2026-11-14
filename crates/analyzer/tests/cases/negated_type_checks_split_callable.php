<?php

declare(strict_types=1);

/**
 * @return array<array-key, mixed>|object
 */
function not_a_string(callable $c): array|object
{
    if (is_string($c)) {
        return [];
    }

    return $c;
}

function only_object(callable $c): object
{
    if (is_string($c) || is_array($c)) {
        return new stdClass();
    }

    return $c;
}

function pretty(callable $callback): string
{
    if (is_string($callback)) {
        return $callback;
    }

    if (is_array($callback)) {
        return is_string($callback[0]) ? $callback[0] : $callback[1];
    }

    if ($callback instanceof Closure) {
        return 'closure';
    }

    return 'invokable';
}

/** @param callable(int): string $fn */
function call_it(callable $fn): string
{
    if (is_string($fn)) {
        return 'string';
    }

    return $fn(42);
}
