<?php

declare(strict_types=1);

/** @return array{-1: string, 0: string, 1: string} */
function t(): array
{
    return [-1 => 'a', 0 => 'b', 1 => 'c'];
}

[$a, $b] = t();
echo $a;
echo $b;
