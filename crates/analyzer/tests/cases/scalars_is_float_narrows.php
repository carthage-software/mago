<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

function example(int|float $x): void {
    if (is_float($x)) {
        takesFloat($x);
    }
}
