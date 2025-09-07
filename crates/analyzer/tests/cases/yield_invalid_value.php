<?php

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:invalid-yield-value-type
 */
function generator(): Generator
{
    yield 1 => 42;
}
