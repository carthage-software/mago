<?php

declare(strict_types=1);

/** @return array<string, int> */
function t(): array
{
    return ['a' => 1, 'b' => 2];
}

/**
 * @mago-expect analysis:list-destructure-string-key
 * @mago-expect analysis:possibly-undefined-int-array-index(2)
 * @mago-expect analysis:mismatched-array-index(2)
 */
[$a, $b] = t();
echo $a;
echo $b;
