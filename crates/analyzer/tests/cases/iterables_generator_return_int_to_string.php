<?php

declare(strict_types=1);

/**
 * @return Generator<int, int, mixed, string>
 *
 * @mago-expect analysis:invalid-return-statement
 */
function gen(): Generator
{
    yield 1;
    return 42;
}
