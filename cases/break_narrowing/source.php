<?php

function takeString(string $s): void
{
    takeString($s);
}

function takeInt(int $i): void
{
    takeInt($i);
}

function testSimpleBreakNarrowing(?string $value): string
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

function testBreakAfterNullCheck(?int $value): ?int
{
    while (true) {
        if ($value !== null) {
            break;
        }
        $value = 0;
        break;
    }

    takeInt($value);

    return $value;
}
