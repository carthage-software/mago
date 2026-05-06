<?php

declare(strict_types=1);

/**
 * @param iterable<string, string> $src
 *
 * @return Generator<int, string>
 *
 */
function relay(iterable $src): Generator
{
    yield from $src;
}
