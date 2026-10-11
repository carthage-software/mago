<?php

function read(int $value, int $other): int
{
    return $value + $other;
}

$partial = read(other: 2, value: ?);
$partial(1);
$partial(value: 1);
// @mago-expect analysis:invalid-named-argument,too-many-arguments
$partial(value: 1, other: 2);
