<?php

declare(strict_types=1);

final class Stack
{
    /** @var list<string> */
    public array $items = [];
}

/**
 * @param list<string> $items
 */
function report(array $items, Stack $stack): void
{
    // @mago-expect analysis:impossible-condition
    if (! $stack instanceof Stack) {
        // @mago-expect analysis:invalid-property-access,no-value
        echo count(array_merge($items, $stack->items));

        return;
    }

    echo count(array_merge($items, $stack->items));
}
