<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:invalid-yield-key-type
 */
function gen(): Generator
{
    yield 1.5 => 'x';
}
