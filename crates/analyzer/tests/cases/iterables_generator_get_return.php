<?php

declare(strict_types=1);

/**
 * @return Generator<int, string, mixed, int>
 */
function gen(): Generator
{
    yield 'a';
    return 42;
}

function take_int(int $_n): void
{
}

$g = gen();
foreach ($g as $_) {
}
take_int($g->getReturn());
