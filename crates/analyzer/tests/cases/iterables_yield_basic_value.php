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

foreach (gen() as $v) {
    echo $v;
}
