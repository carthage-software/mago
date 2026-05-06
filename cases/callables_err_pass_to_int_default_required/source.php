<?php

declare(strict_types=1);

function callables_default_required(int $a, int $b = 10): int
{
    return $a + $b;
}

callables_default_required();
