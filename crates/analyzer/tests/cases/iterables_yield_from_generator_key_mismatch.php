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
 * @mago-expect analysis:yield-from-invalid-key-type
 */
function outer(): Generator
{
    yield from inner();
}
