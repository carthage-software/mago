<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

function caller(int $x): int {
    return takesInt($x);
}
