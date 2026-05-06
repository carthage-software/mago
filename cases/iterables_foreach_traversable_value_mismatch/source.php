<?php

declare(strict_types=1);

function take_int(int $_n): void {}

/**
 * @param Traversable<int, string> $t
 */
function bad(Traversable $t): void
{
    foreach ($t as $v) {
        take_int($v);
    }
}
