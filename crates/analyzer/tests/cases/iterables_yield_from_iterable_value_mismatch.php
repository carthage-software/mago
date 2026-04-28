<?php

declare(strict_types=1);

/**
 * @param iterable<int, int> $src
 *
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-invalid-value-type
 */
function relay(iterable $src): Generator
{
    yield from $src;
}
