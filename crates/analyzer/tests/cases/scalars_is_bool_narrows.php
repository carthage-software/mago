<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

function example(bool|int $x): void {
    if (is_bool($x)) {
        takesBool($x);
    }
}
