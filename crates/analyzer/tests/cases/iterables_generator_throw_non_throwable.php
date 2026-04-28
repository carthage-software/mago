<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield 'a';
}

$g = gen();
$g->current();
/** @mago-expect analysis:invalid-argument */
$g->throw('not throwable');
