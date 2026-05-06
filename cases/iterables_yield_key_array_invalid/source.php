<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 *
 */
function gen(): Generator
{
    yield [1, 2] => 'x';
}
