<?php

function read(int $left, string $right): string
{
    return $right;
}

$partial = read(right: ?, left: ?);
$again = $partial(left: 1, right: ?);
$again('ok');
$again(right: 'ok');
// @mago-expect analysis:invalid-named-argument,too-many-arguments
$again(right: 'ok', left: 2);
