<?php

declare(strict_types=1);

/**
 * @return Generator<int, string, int, void>
 */
function gen(): Generator
{
    $sent = yield 'first';
    echo $sent;
}

$g = gen();
$g->current();
/** @mago-expect analysis:invalid-argument */
$g->send('not an int');
