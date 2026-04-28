<?php

declare(strict_types=1);

/** @param int ...$args */
function variadicIntBJ(int ...$args): int
{
    return array_sum($args);
}

echo variadicIntBJ(1, 2, 3);
