<?php

declare(strict_types=1);

final class Box {}

/**
 * @return Generator<int, string>
 *
 */
function gen(): Generator
{
    yield new Box() => 'x';
}
