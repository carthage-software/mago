<?php

declare(strict_types=1);

function takesInt(int $r): int { return $r; }

function compare(int $a, int $b): int {
    return $a <=> $b;
}

takesInt(compare(1, 2));
