<?php

declare(strict_types=1);

/**
 * @return Generator<int, int>
 *
 * @mago-expect analysis:yield-from-non-iterable
 */
function gen(): Generator
{
    yield from true;
}
