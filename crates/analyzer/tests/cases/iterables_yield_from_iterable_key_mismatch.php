<?php

declare(strict_types=1);

/**
 * @param iterable<string, string> $src
 *
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-invalid-key-type
 */
function relay(iterable $src): Generator
{
    yield from $src;
}
