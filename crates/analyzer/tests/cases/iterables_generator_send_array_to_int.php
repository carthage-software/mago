<?php

declare(strict_types=1);

/**
 * @return Generator<int, int, int, void>
 */
function gen(): Generator
{
    $sent = yield 1;
    echo $sent;
}

$g = gen();
$g->current();
/** @mago-expect analysis:invalid-argument */
$g->send([1, 2, 3]);
