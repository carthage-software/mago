<?php

declare(strict_types=1);

/**
 * @pure
 */
function callables_pure_compute(int $n): int
{
    return $n * 2;
}

/** @mago-expect analysis:unused-statement */
callables_pure_compute(5);
