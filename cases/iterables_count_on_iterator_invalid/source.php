<?php

declare(strict_types=1);

/**
 * @param Iterator<int, string> $it
 *
 */
function bad_count(Iterator $it): int
{
    return count($it);
}
