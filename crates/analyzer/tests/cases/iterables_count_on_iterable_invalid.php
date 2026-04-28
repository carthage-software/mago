<?php

declare(strict_types=1);

/**
 * @param iterable<int> $it
 *
 * @mago-expect analysis:possibly-invalid-argument
 */
function bad_count(iterable $it): int
{
    return count($it);
}
