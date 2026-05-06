<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield 'a';
    yield 'b';
}

$g = gen();
$g->rewind();
$g->next();
