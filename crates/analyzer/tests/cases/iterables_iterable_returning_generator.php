<?php

declare(strict_types=1);

/**
 * @return iterable<int, string>
 */
function get_iter(): iterable
{
    yield 'a';
    yield 'b';
}
