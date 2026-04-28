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

function take_string(string $_s): void
{
}

$g = gen();
foreach ($g as $_) {
}
/** @mago-expect analysis:invalid-argument */
take_string($g->getReturn());
