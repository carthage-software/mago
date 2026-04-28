<?php

declare(strict_types=1);

/**
 * @param null|iterable<int> $it
 */
function iterate_nullable(null|iterable $it): void
{
    if ($it === null) {
        return;
    }

    foreach ($it as $_) {
    }
}
