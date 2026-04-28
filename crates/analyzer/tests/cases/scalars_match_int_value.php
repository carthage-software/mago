<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

function example(int $x): string {
    return match (true) {
        $x < 0 => 'negative',
        $x === 0 => 'zero',
        default => 'positive',
    };
}

takesString(example(5));
