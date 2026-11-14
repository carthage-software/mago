<?php

declare(strict_types=1);

/** @param 'after'|'before' $expected */
function same(string $expected): void
{
    echo $expected;
}

function a(?int $c): void
{
    $x = 'before';
    $z = $c ?? ($x = 'after');
    same($x);
    echo $z;
}

function h(?int $c): int
{
    $z = $c ?? ($y = 5);
    echo $z;

    // @mago-expect analysis:possibly-undefined-variable
    return $y;
}
