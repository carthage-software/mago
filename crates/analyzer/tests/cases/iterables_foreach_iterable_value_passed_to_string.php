<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

/**
 * @param iterable<int, int> $it
 */
function bad(iterable $it): void
{
    foreach ($it as $v) {
        /** @mago-expect analysis:invalid-argument */
        take_string($v);
    }
}
