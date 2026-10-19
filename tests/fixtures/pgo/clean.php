<?php

declare(strict_types=1);

function pgoAdd(int $left, int $right): int
{
    return $left + $right;
}

echo pgoAdd(1, 2);
