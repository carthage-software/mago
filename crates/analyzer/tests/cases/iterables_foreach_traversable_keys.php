<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

function take_string(string $_s): void
{
}

/**
 * @param Traversable<int, string> $t
 */
function iterate_trav(Traversable $t): void
{
    foreach ($t as $k => $v) {
        take_int($k);
        take_string($v);
    }
}
