<?php

declare(strict_types=1);

/** @return "\n"|"\r\n" */
function eol(bool $windows): string
{
    return $windows ? "\r\n" : "\n";
}

/** @return '\\' */
function sep(): string
{
    return '\\';
}

/** @return 'it\'s' */
function apostrophe(): string
{
    return 'it\'s';
}

/** @param 'App\\Foo'|"App\\Bar" $class */
function cls(string $class): string
{
    return $class;
}

cls('App\\Foo');
cls('App\Foo');

/** @param array{'a\'b': int, "c\"d": int} $a */
function shape(array $a): int
{
    return $a["a'b"] + $a['c"d'];
}

/** @return "\t" */
function tab(): string
{
    // @mago-expect analysis:invalid-return-statement
    return "\\t";
}
