<?php

declare(strict_types=1);

/**
 * @param array{0: int, 9223372036854775807?: int} $a
 */
function test(array $a): void
{
    /** @mago-expect analysis:possibly-array-append-overflow */
    $a[] = 1;
}
