<?php

declare(strict_types=1);

$fact = function (int $n) use (&$fact): int {
    if ($n <= 1) {
        return 1;
    }
    /** @var Closure(int): int $fact */
    return $n * $fact($n - 1);
};

echo $fact(5);
