<?php

declare(strict_types=1);

/**
 * @param Traversable<int, string> $t
 *
 * @mago-expect analysis:possibly-invalid-argument
 */
function bad(Traversable $t): int
{
    return count($t);
}
