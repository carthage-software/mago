<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield from ['a', 'b', 'c'];
}
