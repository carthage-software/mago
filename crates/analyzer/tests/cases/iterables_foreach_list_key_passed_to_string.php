<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

/**
 * @param list<string> $items
 */
function iterate_list(array $items): void
{
    foreach ($items as $key => $_value) {
        /** @mago-expect analysis:invalid-argument */
        take_string($key);
    }
}
