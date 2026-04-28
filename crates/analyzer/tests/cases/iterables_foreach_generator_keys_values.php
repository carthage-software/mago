<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

function take_int(int $_n): void
{
}

/**
 * @return Generator<string, int>
 */
function gen(): Generator
{
    yield 'a' => 1;
    yield 'b' => 2;
}

foreach (gen() as $k => $v) {
    take_string($k);
    take_int($v);
}
