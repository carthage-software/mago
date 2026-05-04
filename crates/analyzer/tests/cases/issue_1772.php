<?php

declare(strict_types=1);

/**
 * @param callable(bool &$ret): void $callable
 */
function take_ref_callback(callable $callable): bool
{
    $result = false;
    $callable($result);
    return $result;
}

/**
 * @param callable(string &...$rest): int $callable
 */
function take_variadic_ref_callback(callable $callable): int
{
    $items = ['a', 'b'];
    return $callable(...$items);
}

/**
 * @param callable(Countable&Traversable $value): int $callable
 */
function take_intersection_callback(callable $callable, Countable&Traversable $value): int
{
    return $callable($value);
}
