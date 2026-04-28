<?php

declare(strict_types=1);

/**
 * @return Generator<string, int>
 */
function gen(): Generator
{
    yield 'a' => 1;
    yield 'b' => 2;
}

/**
 * @param list<int> $_l
 */
function take_list(array $_l): void
{
}

take_list(iterator_to_array(gen(), false));
