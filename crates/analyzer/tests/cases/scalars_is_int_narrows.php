<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

function example(int|string $x): void {
    if (is_int($x)) {
        takesInt($x);
    }
}
