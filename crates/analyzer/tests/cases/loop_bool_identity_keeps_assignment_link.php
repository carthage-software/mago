<?php

declare(strict_types=1);

final class U
{
    public int $id = 1;
}

function viaWhile(?U $v): int
{
    $c = $v !== null;
    while ($c === true) {
        return $v->id;
    }

    return 0;
}

function viaFor(?U $v): int
{
    $c = $v !== null;
    for (; $c !== false;) {
        return $v->id;
    }

    return 0;
}

function viaYoda(?U $v): int
{
    $c = $v !== null;
    while (true === $c) {
        return $v->id;
    }

    return 0;
}
