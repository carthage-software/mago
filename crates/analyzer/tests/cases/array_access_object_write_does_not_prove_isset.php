<?php

declare(strict_types=1);

/**
 * @implements ArrayAccess<string, int>
 */
final class Frozen implements ArrayAccess
{
    public function offsetSet(mixed $offset, mixed $value): void {}

    public function offsetGet(mixed $offset): mixed
    {
        exit(1);
    }

    public function offsetExists(mixed $offset): bool
    {
        return false;
    }

    public function offsetUnset(mixed $offset): void {}
}

function probe(Frozen $map, string $key): bool
{
    $map[$key] = 1;

    if (isset($map[$key])) {
        return true;
    }

    return false;
}
