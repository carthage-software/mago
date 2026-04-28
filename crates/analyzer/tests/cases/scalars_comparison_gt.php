<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

function example(int $a, int $b): bool {
    return $a > $b;
}

takesBool(example(10, 5));
