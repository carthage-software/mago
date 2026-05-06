<?php

declare(strict_types=1);

/**
 * @param Traversable<int, string> $t
 *
 */
function bad(Traversable $t): int
{
    return count($t);
}
