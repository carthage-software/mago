<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

function example(int $a): float {
    return takesFloat($a / 2.5);
}

example(10);
