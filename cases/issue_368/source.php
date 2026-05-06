<?php

$fibonacci = function (int $n) use (&$fibonacci): int {
    if ($n <= 1) {
        return $n;
    }

    return $fibonacci($n - 1) + $fibonacci($n - 2);
};

echo $fibonacci(10);

$factorial = function (int $n) use (&$factorial): int {
    if ($n <= 1) {
        return 1;
    }

    return $n * $factorial($n - 1, 0);
};

echo $factorial(5);

$gcd = function (int $a, int $b) use (&$gcd): int {
    if ($b === 0) {
        return $a;
    }

    return $gcd($b);
};

echo $gcd(48, 18);
