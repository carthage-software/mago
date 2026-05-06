<?php

declare(strict_types=1);

/** @return array{0: int, 1: int, foo: int} */
function t(): array
{
    return [0 => 0, 1 => 1, 'foo' => 2];
}

[$a, $b] = t();
echo $a;
echo $b;
