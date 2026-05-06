<?php

declare(strict_types=1);

/**
 * @return Generator<int, int, mixed, string>
 *
 */
function gen(): Generator
{
    yield 1;
    return 42;
}
