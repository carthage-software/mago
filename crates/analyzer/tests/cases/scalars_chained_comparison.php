<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

function example(int $x): bool {
    return $x > 0 && $x < 100;
}

takesBool(example(50));
