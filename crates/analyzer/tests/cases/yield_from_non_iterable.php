<?php

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-non-iterable
 */
function generator(): Generator
{
    yield from 42;
}
