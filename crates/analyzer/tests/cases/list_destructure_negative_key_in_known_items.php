<?php

declare(strict_types=1);

/** @return array{-1: string, 0: string, 1: string} */
function t(): array
{
    return [-1 => 'a', 0 => 'b', 1 => 'c'];
}

/** @mago-expect analysis:list-destructure-negative-key */
[$a, $b] = t();
echo $a;
echo $b;
