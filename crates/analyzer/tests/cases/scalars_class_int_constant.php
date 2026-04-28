<?php

declare(strict_types=1);

final class Limits {
    public const MAX = 100;
}

/** @param int<100, 100> $n */
function takes(int $n): int { return $n; }

takes(Limits::MAX);
