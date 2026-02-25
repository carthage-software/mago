<?php

declare(strict_types=1);

/**
 * @template Obj of object
 * @param class-string<Obj> $classString
 * @return Obj
 * @throws Exception
 */
function parse_instance_of_ok(string $classString, mixed $unknown): object
{
    if (!is_object($unknown)) {
        throw new \Exception();
    }

    if (!$unknown instanceof $classString) {
        throw new \Exception();
    }

    return $unknown;
}

/**
 * @template Obj of object
 * @param class-string<Obj> $classString
 * @return Obj
 * @throws Exception
 */
function parse_instance_of_combined(string $classString, mixed $unknown): object
{
    if (!is_object($unknown) || !$unknown instanceof $classString) {
        throw new \Exception();
    }

    return $unknown;
}

/**
 * @template T of object
 * @param class-string<T> $class
 * @return T
 * @throws Exception
 */
function parse_with_negated_and(string $class, mixed $value): object
{
    if (!($value instanceof $class)) {
        throw new \Exception();
    }

    return $value;
}
