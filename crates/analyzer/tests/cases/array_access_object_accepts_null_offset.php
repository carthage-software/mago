<?php

declare(strict_types=1);

/**
 * @implements ArrayAccess<string|null, int>
 */
final class NullableKeyBag implements ArrayAccess
{
    #[Override]
    public function offsetExists(mixed $offset): bool
    {
        return $offset === null;
    }

    #[Override]
    public function offsetGet(mixed $offset): int
    {
        return $offset === null ? 1 : 2;
    }

    #[Override]
    public function offsetSet(mixed $offset, mixed $value): void
    {
    }

    #[Override]
    public function offsetUnset(mixed $offset): void
    {
    }
}

function read_null_offset(NullableKeyBag $bag, null|string $key): int
{
    $bag[null] = 3;
    unset($bag[null]);

    return $bag[null] + $bag[$key] + (isset($bag[null]) ? 1 : 0);
}

/**
 * @param array<string, int> $values
 */
function read_null_array_offset(array $values): int
{
    /** @mago-expect analysis:null-array-index */
    return $values[null];
}
