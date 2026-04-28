<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

/**
 * @param array<int, int> $items
 */
function bad(array $items): void
{
    foreach ($items as $k => $_v) {
        /** @mago-expect analysis:invalid-argument */
        take_string($k);
    }
}
