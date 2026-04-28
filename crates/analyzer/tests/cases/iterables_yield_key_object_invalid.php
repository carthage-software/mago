<?php

declare(strict_types=1);

final class Box
{
}

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:invalid-yield-key-type
 */
function gen(): Generator
{
    yield new Box() => 'x';
}
