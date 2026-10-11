<?php

function f(): string
{
    return $a
        . "x" . $b
        . "y" . $c;
}

function g(): array
{
    return h(extra: $a + [
        'k' => 1,
    ]);
}

function minimal(): string
{
    return $a
        . "x" . $b;
}

function parenthesized(): string
{
    return ($a
        . "x" . $b);
}

function fullyBroken(): string
{
    return $a
        . "x"
        . $b;
}

h($a + [
    'k' => 1,
]);

h(extra: $a
    + [
        'k' => 1,
    ]);

$x = $a
    . "x" . $b;

foo($a
    . "x");

function inline(): string
{
    return $a . "x" . $b;
}

h($a + ['k' => 1]);
