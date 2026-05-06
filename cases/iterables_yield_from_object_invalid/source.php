<?php

declare(strict_types=1);

final class Plain {}

/**
 * @return Generator<int, int>
 *
 */
function gen(): Generator
{
    yield from new Plain();
}
