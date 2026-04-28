<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function double_in_place(array $xs): void
{
    array_walk($xs, static function (int &$x): void {
        $x *= 2;
    });
}
