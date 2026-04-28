<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-invalid-value-type
 */
function gen(): Generator
{
    yield from [1, 2, 3];
}
