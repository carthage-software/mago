<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

function example(mixed $x): int {
    if (is_int($x)) {
        return takesInt($x);
    }
    return 0;
}
