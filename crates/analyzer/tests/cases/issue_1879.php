<?php

declare(strict_types=1);

/**
 * @param non-empty-lowercase-string $str
 */
function foo(string $str): void
{
    echo $str;
}

/** @psalm-assert-if-true lowercase-string $s */
function my_lowercase_asserter(string $s): bool
{
    return ctype_lower($s);
}

$str = (string) file_get_contents('a.txt');

if ($str !== '' && $str === strtolower($str)) {
    foo($str);
}

if ($str !== '' && ctype_lower($str)) {
    foo($str);
}

if ($str !== '' && my_lowercase_asserter($str)) {
    foo($str);
}
