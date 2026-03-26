<?php declare(strict_types=1);

function test1(string $foo): void
{
    $parts = explode(',', $foo);
    if (count($parts) !== 2 || strtolower($parts[0]) !== 'bar') {
        // Do something
    }
}

function test2(string $foo): void
{
    if (strlen($foo) > 10 || ctype_digit($foo)) {
        // Do something
    }
}
