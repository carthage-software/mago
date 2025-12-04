<?php

function test_float_concat(): void
{
    $time = 1.5;

    $result = 'Time: ' . $time . 'ms';
    $result2 = 'Value: ' . (3.14);
    $result3 = 'Int: ' . (42) . ', Float: ' . (1.5);
}

function test_with_return(): string
{
    $elapsed = microtime(true);
    return 'Elapsed: ' . $elapsed;
}
