<?php

final readonly class Str
{
    /**
     * Convert the given string to upper-case.
     *
     * @param  string  $value
     * @return ($value is '' ? '' : non-empty-string&uppercase-string)
     */
    public static function upper(string $value): string
    {
        return mb_strtoupper($value, 'UTF-8');
    }
}

/**
 * @return non-empty-uppercase-string
 */
function foo(): string
{
    return Str::upper('foo');
}

/**
 * @return ''
 */
function bar(): string
{
    return Str::upper('');
}

/**
 * @return uppercase-string
 */
function baz(string $x): string
{
    return Str::upper($x);
}
