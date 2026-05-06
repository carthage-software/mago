<?php

declare(strict_types=1);

function callables_normal(int $n): int
{
    return $n + 1;
}

echo callables_normal(5);
