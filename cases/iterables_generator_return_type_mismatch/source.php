<?php

declare(strict_types=1);

/**
 * @return Generator<int, string, mixed, string>
 *
 */
function gen(): Generator
{
    yield 'a';
    return 42;
}
