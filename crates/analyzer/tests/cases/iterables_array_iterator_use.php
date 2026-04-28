<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

/**
 * @param ArrayIterator<int, int> $iter
 */
function consume(ArrayIterator $iter): void
{
    foreach ($iter as $v) {
        take_int($v);
    }
}
