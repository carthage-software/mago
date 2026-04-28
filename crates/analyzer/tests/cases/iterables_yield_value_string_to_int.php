<?php

declare(strict_types=1);

/**
 * @return Generator<int, int>
 *
 * @mago-expect analysis:invalid-yield-value-type
 */
function gen(): Generator
{
    yield 0 => 'oops';
}
