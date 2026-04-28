<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

function example(): int {
    $sum = 0;
    for ($i = 0; $i < 10; $i++) {
        $sum += $i;
    }
    return takesInt($sum);
}
