<?php

declare(strict_types=1);

/** @return array<string, int> */
function t(): array
{
    return ['a' => 1, 'b' => 2];
}

/**
 */
[$a, $b] = t();
echo $a;
echo $b;
