<?php

declare(strict_types=1);

$square = static function (int $n): int {
    return $n * $n;
};

echo $square(5);
