<?php

declare(strict_types=1);

/**
 * @param Generator<int, string> $g
 *
 * @mago-expect analysis:invalid-argument
 */
function bad(Generator $g): int
{
    return count($g);
}
