<?php

declare(strict_types=1);

function callables_array_default(int $n = []): int
{
    return $n;
}

echo callables_array_default(1);
