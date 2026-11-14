<?php

declare(strict_types=1);

/**
 * @psalm-assert object{a: int, ...} $obj
 *
 * @throws InvalidArgumentException
 */
function assertA(object $obj): void
{
    if (!property_exists($obj, 'a') || !is_int($obj->a)) {
        throw new InvalidArgumentException('a');
    }
}

/**
 * @psalm-assert object{b: string, ...} $obj
 *
 * @throws InvalidArgumentException
 */
function assertB(object $obj): void
{
    if (!property_exists($obj, 'b') || !is_string($obj->b)) {
        throw new InvalidArgumentException('b');
    }
}

/**
 * @throws InvalidArgumentException
 */
function both(object $o): string
{
    assertA($o);
    assertB($o);

    return $o->b . $o->a;
}

/** @psalm-assert-if-true stdClass $o */
function is_std(object $o): bool
{
    return $o instanceof stdClass;
}

/** @param object{a: int, ...} $o */
function std_shape(object $o): int
{
    if (is_std($o)) {
        return $o->a;
    }

    return 0;
}
