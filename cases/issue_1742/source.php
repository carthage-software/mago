<?php

declare(strict_types=1);

interface I
{
    public function f(): bool;
}

function f(I $i): void
{
    $ok = false;

    try {
        $ok = $i->f();

        $ok = true;
    } finally {
        if (!$ok) {
            echo 'oops';
        }
    }
}
