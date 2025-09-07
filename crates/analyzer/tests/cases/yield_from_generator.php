<?php

/**
 * @return Generator<int, string>
 *
 * @mago-expect analysis:yield-from-invalid-key-type
 */
function generator(): Generator
{
    yield from get_string_string_iterable();
}

/**
 * @return iterable<string, string>
 */
function get_string_string_iterable(): iterable
{
    return [
        'key1' => 'value1',
        'key2' => 'value2',
    ];
}

function i_take_string(string $_string): void
{
}

foreach (generator() as $key => $value) {
    i_take_string($key); // @mago-expect analysis:invalid-argument
    i_take_string($value);
}
