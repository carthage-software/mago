<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield 'x';
}

function take_string_or_null(null|string $_s): void
{
}

$g = gen();
take_string_or_null($g->current());
