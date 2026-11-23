<?php

declare(strict_types=1);

final class A {}

/**
 * @template T of A|null
 *
 * @param class-string<T> $class
 */
function make(string $class): A
{
    return new $class();
}

/**
 * @template T of string|bool
 *
 * @param class-string<T> $s
 */
function no_class(string $s): string
{
    return $s;
}
