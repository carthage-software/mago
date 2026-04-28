<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield 'x';
}

function take_bool(bool $_b): void
{
}

$g = gen();
take_bool($g->valid());
