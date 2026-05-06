<?php

declare(strict_types=1);

/** @return array<int, string> */
function t(): array
{
    return [0 => 'a', 1 => 'b'];
}

/**
 */
[$a, $b] = t();
echo $a;
echo $b;
