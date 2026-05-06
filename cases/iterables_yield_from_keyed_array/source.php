<?php

declare(strict_types=1);

/**
 * @return Generator<string, int>
 */
function gen(): Generator
{
    yield from ['a' => 1, 'b' => 2];
}

foreach (gen() as $k => $v) {
    echo $k . ':' . $v;
}
