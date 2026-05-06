<?php

declare(strict_types=1);

/** @return array<string, int> */
function t(): array
{
    return ['name' => 1, 'age' => 2];
}

['name' => $a, 'age' => $b] = t();
echo $a;
echo $b;
