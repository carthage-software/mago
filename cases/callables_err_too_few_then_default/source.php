<?php

declare(strict_types=1);

function callables_a_required_b_default(int $a, int $b = 0): int
{
    return $a + $b;
}

callables_a_required_b_default();
