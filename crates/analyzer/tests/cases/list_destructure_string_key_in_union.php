<?php

declare(strict_types=1);

/** @return array<int|string, int> */
function t(): array
{
    return ['a' => 0, 'b' => 1];
}

/**
 * @mago-expect analysis:list-destructure-string-key
 * @mago-expect analysis:possibly-undefined-int-array-index(2)
 */
[$a, $b] = t();
echo $a;
echo $b;
