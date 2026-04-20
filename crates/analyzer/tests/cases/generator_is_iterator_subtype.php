<?php

declare(strict_types=1);

class Thing {}

/**
 * @return Generator<int, Thing>
 */
function make_generator(): Generator
{
    yield new Thing();
}

/**
 * @param Iterator<mixed, Thing> $it
 */
function accept_iterator(Iterator $it): void
{
    foreach ($it as $_) {}
}

/**
 * @param iterable<mixed, Thing> $source
 */
function driver(iterable $source): void
{
    $iter = $source instanceof Iterator
        ? $source
        : make_generator();

    accept_iterator($iter);
}
