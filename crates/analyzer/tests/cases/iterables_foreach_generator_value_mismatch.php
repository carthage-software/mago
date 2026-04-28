<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield 'a';
}

foreach (gen() as $v) {
    /** @mago-expect analysis:invalid-argument */
    take_int($v);
}
