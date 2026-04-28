<?php

declare(strict_types=1);

/** @param positive-int $b */
function divider(int $a, int $b): int|float {
    return $a / $b;
}

divider(10, 5);
