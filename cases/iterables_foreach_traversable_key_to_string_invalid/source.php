<?php

declare(strict_types=1);

function take_string(string $_s): void {}

/**
 * @param Traversable<int, string> $t
 */
function bad(Traversable $t): void
{
    foreach ($t as $k => $_v) {
        take_string($k);
    }
}
