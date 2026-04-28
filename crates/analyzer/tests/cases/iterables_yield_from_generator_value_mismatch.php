<?php

declare(strict_types=1);

/**
 * @return Generator<int, int>
 */
function inner(): Generator
{
    yield 1;
    yield 2;
}

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-invalid-value-type
 */
function outer(): Generator
{
    yield from inner();
}
