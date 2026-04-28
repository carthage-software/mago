<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:invalid-yield-value-type
 */
function gen(): Generator
{
    yield null;
}
