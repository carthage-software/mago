<?php

declare(strict_types=1);

/** @return array{0: string, 1: string} */
function t(): array
{
    return ['a', 'b'];
}

[$a, $b] = t();
echo $a;
echo $b;
