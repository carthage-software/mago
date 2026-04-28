<?php

declare(strict_types=1);

/**
 * @return Generator<int, string, mixed, string>
 *
 * @mago-expect analysis:invalid-return-statement
 */
function gen(): Generator
{
    yield 'a';
    return 42;
}
