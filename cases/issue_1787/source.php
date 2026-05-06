<?php

declare(strict_types=1);

namespace App;

function say_hello(): void
{
    echo 'Hello';
}

function g(int $n): string
{
    $ok = match ($n) {
        0 => true,
        default => namespace\say_hello(),
    };

    if ($ok) {
        return 'a';
    }

    return 'b';
}
