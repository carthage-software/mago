<?php

declare(strict_types=1);

/**
 * @return Generator<string, string>
 *
 * @mago-expect analysis:invalid-yield-value-type
 */
function gen(): Generator
{
    yield 'a' => 1;
}
