<?php

declare(strict_types=1);

/** @return array<int, string> */
function t(): array
{
    return [0 => 'a', 1 => 'b'];
}

/**
 * @mago-expect analysis:possibly-undefined-int-array-index(2)
 */
[$a, $b] = t();
echo $a;
echo $b;
