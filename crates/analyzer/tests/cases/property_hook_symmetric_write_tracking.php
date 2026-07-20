<?php

declare(strict_types=1);

namespace PropertyHookSymmetricWriteTracking;

/**
 * A `set` hook whose parameter type matches the property (read) type. Writes and reads do not
 * diverge, so clamping the memoized write to the read type must be a no-op: the written value is
 * tracked precisely and is not widened back to the declared type.
 */
class Counter
{
    public ?int $count = null {
        get {
            return $this->count;
        }
        set(?int $value) {
            $this->count = $value;
        }
    }
}

function takesInt(int $_value): void
{
}

$counter = new Counter();

// Writing a non-null `int` narrows the read: the `null` branch is excluded and the value passes
// where a plain `int` is required. Widening back to `?int` would wrongly reject this as nullable.
$counter->count = 5;
takesInt($counter->count);
