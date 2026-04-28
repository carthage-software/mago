<?php

declare(strict_types=1);

/**
 * @return Generator<int, int|string>
 */
function gen(): Generator
{
    yield 1;
    yield 'two';
    yield 3;
}
