<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

/**
 * @param iterable<int, string> $it
 */
function bad(iterable $it): void
{
    foreach ($it as $v) {
        /** @mago-expect analysis:invalid-argument */
        take_int($v);
    }
}
