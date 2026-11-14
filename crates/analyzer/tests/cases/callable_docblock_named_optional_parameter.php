<?php

declare(strict_types=1);

/**
 * @param callable(int $x, string $y=): void $c
 */
function run(callable $c): void
{
    $c(1);
    $c(1, 'y');
}
