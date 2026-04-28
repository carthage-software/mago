<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-invalid-key-type
 */
function gen(): Generator
{
    yield from ['a' => 'x', 'b' => 'y'];
}
