<?php

declare(strict_types=1);

function callables_one_required(int $n): int
{
    return $n;
}

callables_one_required(1, 2);
