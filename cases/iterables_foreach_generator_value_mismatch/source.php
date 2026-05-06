<?php

declare(strict_types=1);

function take_int(int $_n): void {}

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield 'a';
}

foreach (gen() as $v) {
    take_int($v);
}
