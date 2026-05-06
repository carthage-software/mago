<?php

declare(strict_types=1);

/**
 * @param int<1, 500> $x
 * @return int<1, 500>
 */
function iterate(int $x): int
{
    for (; $x <= 300; $x++) {
        if (rand(min: 0, max: 10) === 0) {
            break;
        }
    }

    return $x;
}
