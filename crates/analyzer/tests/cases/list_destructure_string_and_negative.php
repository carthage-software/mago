<?php

declare(strict_types=1);

/** @return array{-1: int, 0: int, foo: int} */
function t(): array
{
    return [-1 => 1, 0 => 2, 'foo' => 3];
}

/**
 * @mago-expect analysis:undefined-int-array-index
 */
[$a, $b] = t();
echo $a;
echo $b;
