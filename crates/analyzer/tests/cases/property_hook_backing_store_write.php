<?php

declare(strict_types=1);

namespace PropertyHookBackingStoreWrite;

/**
 * Assigning to `$this->foo` inside `foo`'s own `set` hook writes the backing store directly — it
 * does not re-enter the hook. So the write is checked against the property (backing) type, not the
 * wider `set` parameter type: a raw `string|int|false` value is invalid for the `int|false` store.
 */
class Converter
{
    public int|false $foo = 0 {
        get {
            return $this->foo;
        }
        set(string|int|false $value) {
            $this->foo = $value; // @mago-expect analysis:invalid-property-assignment-value
        }
    }
}
