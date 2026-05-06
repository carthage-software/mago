<?php

declare(strict_types=1);

/** @return array<negative-int, string> */
function t(): array
{
    return [-1 => 'a'];
}

/**
 */
[$a, $b] = t();
echo $a;
echo $b;
