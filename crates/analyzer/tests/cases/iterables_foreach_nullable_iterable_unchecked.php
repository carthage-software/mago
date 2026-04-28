<?php

declare(strict_types=1);

/**
 * @param null|iterable<int> $it
 *
 * @mago-expect analysis:possibly-null-iterator
 */
function iterate_nullable(null|iterable $it): void
{
    foreach ($it as $_) {
    }
}
