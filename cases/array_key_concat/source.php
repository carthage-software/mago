<?php

/**
 * @param array-key $key
 */
function test_array_key_concat(int|string $key): void
{
    $result = 'Key: ' . $key;
}

function test_foreach_key(): void
{
    $array = ['a' => 1, 'b' => 2];

    foreach ($array as $key => $value) {
        $message = 'Processing key: ' . $key;
    }
}
