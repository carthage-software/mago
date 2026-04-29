<?php

declare(strict_types=1);

/** @return array{0: string, 2: string} */
function t(): array
{
    return [0 => 'a', 2 => 'b'];
}

/** @mago-expect analysis:undefined-int-array-index */
[$a, $b] = t();
echo $a;
echo $b;
