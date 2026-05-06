<?php

declare(strict_types=1);

/**
 * @param Generator<int, string> $g
 *
 */
function bad(Generator $g): int
{
    return count($g);
}
