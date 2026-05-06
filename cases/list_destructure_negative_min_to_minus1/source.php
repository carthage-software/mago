<?php

declare(strict_types=1);

/** @return array<int<min, -1>, string> */
function t(): array
{
    return [-1 => 'a'];
}

/**
 */
[$a, $b] = t();
echo $a;
echo $b;
