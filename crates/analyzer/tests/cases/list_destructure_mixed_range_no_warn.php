<?php

declare(strict_types=1);

/** @return array<int<-5, 5>, string> */
function t(): array
{
    return [0 => 'a'];
}

/**
 * @mago-expect analysis:possibly-undefined-int-array-index(2)
 */
[$a, $b] = t();
echo $a;
echo $b;
