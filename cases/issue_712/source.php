<?php

/**
 * @template TKey of array-key
 * @template TValue
 * @implements ArrayAccess<TKey, TValue>
 */
class CustomArrayAccess implements ArrayAccess
{
    public function offsetExists(mixed $offset): bool
    {
        return true;
    }

    public function offsetGet(mixed $offset): mixed
    {
        return null;
    }

    public function offsetSet(mixed $offset, mixed $value): void {}

    public function offsetUnset(mixed $offset): void {}
}

function test(): void
{
    /** @var CustomArrayAccess<string, string> $arr */
    $arr = new CustomArrayAccess();

    $arr[1] = 'hello';

    $arr['foo'] = true;

    // OK: correct types
    $arr['bar'] = 'hello';
}
