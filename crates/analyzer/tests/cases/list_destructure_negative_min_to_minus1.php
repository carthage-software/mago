<?php

declare(strict_types=1);

/** @return array<int<min, -1>, string> */
function t(): array
{
    return [-1 => 'a'];
}

/**
 * @mago-expect analysis:list-destructure-negative-key
 * @mago-expect analysis:possibly-undefined-int-array-index(2)
 * @mago-expect analysis:mismatched-array-index(2)
 */
[$a, $b] = t();
echo $a;
echo $b;
