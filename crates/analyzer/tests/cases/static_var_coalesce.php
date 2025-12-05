<?php

declare(strict_types=1);

class Counter
{
    /**
     * Static variable with null coalesce assignment.
     *
     * This should NOT be flagged as redundant-null-coalesce.
     * Static variables persist across calls, so even though
     * $count is initialized to null, on subsequent calls it
     * may hold a non-null value.
     */
    public function increment(): int
    {
        static $count = null;

        // This ??= is NOT redundant - $count persists across calls
        $count ??= 0;

        return ++$count;
    }
}

/**
 * Standalone function with static variable coalesce.
 */
function getCounter(): int
{
    static $counter = null;

    // This ??= is NOT redundant - $counter persists across calls
    $counter ??= 0;

    return ++$counter;
}

function test(): void
{
    $obj = new Counter();
    echo $obj->increment() . "\n"; // 1
    echo $obj->increment() . "\n"; // 2

    echo getCounter() . "\n"; // 1
    echo getCounter() . "\n"; // 2
}
