<?php

declare(strict_types=1);

/** @return Generator<int, string, mixed, int> */
function inner(): Generator
{
    yield 'a';

    return 1;
}

/** @return Generator<int, string, mixed, int> */
function outer(): Generator
{
    return yield from inner();
}

/** @return Generator<int, string, mixed, null> */
function from_array(): Generator
{
    return yield from ['a', 'b'];
}

/** @return Generator<int, string, mixed, string> */
function wrong_return(): Generator
{
    // @mago-expect analysis:invalid-return-statement
    return yield from inner();
}
