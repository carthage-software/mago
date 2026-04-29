<?php

declare(strict_types=1);

/** @return array{-3: string, -2: string} */
function t(): array
{
    return [-3 => 'a', -2 => 'b'];
}

/**
 * @mago-expect analysis:list-destructure-negative-key
 * @mago-expect analysis:undefined-int-array-index(2)
 */
[$a, $b] = t();
echo $a;
echo $b;
