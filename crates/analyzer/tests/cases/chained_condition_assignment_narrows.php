<?php

declare(strict_types=1);

final class Def
{
    public int $n = 0;
}

function f(?Def $d): int
{
    if ($isSimple = $isRoot = null === $d) {
        return 0;
    }

    return $d->n;
}
