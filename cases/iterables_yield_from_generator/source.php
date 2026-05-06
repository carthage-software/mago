<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 */
function inner(): Generator
{
    yield 'x';
    yield 'y';
}

/**
 * @return Generator<int, string>
 */
function outer(): Generator
{
    yield from inner();
}

foreach (outer() as $v) {
    echo $v;
}
