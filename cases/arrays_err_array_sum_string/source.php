<?php

declare(strict_types=1);

function bad(string $s): int|float
{
    return array_sum($s);
}
