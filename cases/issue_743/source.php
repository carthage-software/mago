<?php

function stringify(mixed $value): string
{
    if (is_object($value) && method_exists($value, '__toString')) {
        return (string) $value;
    }

    return 'hm, idk';
}

function stringify_uppercase(mixed $value): string
{
    if (is_object($value) && method_exists($value, '__TOSTRING')) {
        return (string) $value;
    }

    return 'hm, idk';
}
