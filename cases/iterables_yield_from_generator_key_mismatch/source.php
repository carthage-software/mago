<?php

declare(strict_types=1);

/**
 * @return Generator<string, string>
 */
function inner(): Generator
{
    yield 'a' => 'x';
}

/**
 * @return Generator<int, string>
 *
 */
function outer(): Generator
{
    yield from inner();
}
