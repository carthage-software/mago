<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

/**
 * @param iterable<int, string> $it
 */
function bad(iterable $it): void
{
    foreach ($it as $k => $_v) {
        /** @mago-expect analysis:invalid-argument */
        take_string($k);
    }
}
