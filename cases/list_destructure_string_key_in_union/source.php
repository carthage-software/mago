<?php

declare(strict_types=1);

/** @return array<int|string, int> */
function t(): array
{
    return ['a' => 0, 'b' => 1];
}

/**
 */
[$a, $b] = t();
echo $a;
echo $b;
