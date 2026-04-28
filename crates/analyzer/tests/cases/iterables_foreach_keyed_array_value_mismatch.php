<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

/**
 * @param array<string, int> $items
 */
function iterate_keyed(array $items): void
{
    foreach ($items as $value) {
        /** @mago-expect analysis:invalid-argument */
        take_string($value);
    }
}
