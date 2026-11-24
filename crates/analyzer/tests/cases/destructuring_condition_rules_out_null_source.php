<?php

declare(strict_types=1);

/** @return list{int, int}|null */
function destructuring_pair(int $c): ?array {
    return $c ? [$c, $c] : null;
}

/** @return array{x: int}|null */
function destructuring_point(int $c): ?array {
    return $c ? ['x' => $c] : null;
}

/** @return list{?int, int}|null */
function destructuring_nullable_pair(int $c): ?array {
    return $c ? [null, $c] : null;
}

function destructuring_conditions(): void {
    if ([$a, $b] = destructuring_pair(1)) {
        echo $a + $b;
    }

    if (['x' => $x] = destructuring_point(1)) {
        echo $x + 1;
    }

    while ([$c, $d] = destructuring_pair(rand(0, 1))) {
        echo $c + $d;
    }

    if (list($e, $g) = destructuring_pair(1)) {
        echo $e + $g;
    }

    if ([$h, $i] = destructuring_nullable_pair(1)) {
        echo $h + $i;
    }

    if ([, $j] = destructuring_pair(1)) {
        echo $j + 1;
    }
}
