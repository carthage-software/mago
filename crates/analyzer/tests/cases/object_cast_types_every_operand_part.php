<?php

declare(strict_types=1);

final class Foo {}

function from_mixed(mixed $value): bool
{
    $object = (object) $value;
    if ($object instanceof Foo) {
        return true;
    }

    return false;
}

function from_object_or_array(Foo|array $value): bool
{
    $object = (object) $value;
    if ($object instanceof Foo) {
        return true;
    }

    return false;
}

function from_nullable_object(?Foo $value): bool
{
    $object = (object) $value;
    if ($object instanceof Foo) {
        return true;
    }

    return false;
}

function from_scalar(int|string|null $value): stdClass
{
    return (object) $value;
}

/**
 * @param array{a: int}|null $value
 */
function from_nullable_shape(?array $value): stdClass
{
    $object = (object) $value;
    if ($object instanceof stdClass) { // @mago-expect analysis:redundant-condition
        return $object;
    }

    return new stdClass();
}

/**
 * @param array{a: int} $value
 */
function from_shape(array $value): int
{
    $object = (object) $value;

    return $object->a;
}
