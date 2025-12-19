<?php

function takeString(string $s): void
{
    takeString($s);
}

function testSimpleBreakNarrowing(null|string $value): string
{
    while (true) {
        if ($value === null) {
            $value = 'default';
            break;
        }
        break;
    }

    takeString($value);

    return $value;
}
