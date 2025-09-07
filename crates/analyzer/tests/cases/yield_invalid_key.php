<?php

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:invalid-yield-key-type
 */
function generator(): Generator
{
    yield 'key' => 'value';
}
