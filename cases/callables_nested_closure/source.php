<?php

declare(strict_types=1);

$outer = function (int $a): Closure {
    return function (int $b) use ($a): int {
        return $a + $b;
    };
};

$inner = $outer(10);
echo $inner(5);
