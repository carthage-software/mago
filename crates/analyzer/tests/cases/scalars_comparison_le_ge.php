<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

function example(int $a, int $b): bool {
    return $a <= $b && $a >= 0;
}

takesBool(example(5, 5));
takesBool(example(10, 10));
