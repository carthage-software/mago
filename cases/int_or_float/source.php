<?php

function example(int|float|string $x): void
{
    echo "The value is: $x\n";
}

function main(int $value, int $other): void
{
    $a = $value / $other;

    example($a);
    example($value);
    example($other);
    example('Hello, World!');
}
