<?php

declare(strict_types=1);

final class Def
{
    public int $n = 0;
}

function f(?Def $d): int
{
    $ok = !is_null($d);
    if (!$ok) {
        return 0;
    }

    return $d->n;
}

function g(?Def $d): int
{
    if ($missing = !($d instanceof Def)) {
        return 0;
    }

    return $d->n;
}
