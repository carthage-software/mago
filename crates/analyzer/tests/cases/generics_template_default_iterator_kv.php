<?php

declare(strict_types=1);

/**
 * @return Iterator<int, string>
 */
function gen_make_iter(): Iterator
{
    yield 0 => 'a';
    yield 1 => 'b';
}

foreach (gen_make_iter() as $k => $v) {
    echo $k . '-' . $v;
}
