<?php

function get_string(): string
{
    return 'string';
}

function get_float(): float
{
    return 1.23;
}

function take_int(int $a): void
{
    echo $a;
}

function take_string(string $a): void
{
    echo $a;
}

function take_float(float $a): void
{
    echo $a;
}

const A = 123; // ok - inferred as int

const B = get_string(); // @mago-expect analysis:non-documented-constant

/** @var float */
const C = get_float(); // ok - documented

define('A2', 456); // ok - inferred as int
define('B2', get_string()); // err, but not reported - non-documented-constant
/** @var float */
define('C2', get_float()); // ok - documented

take_int(A); // ok
take_string(B); // @mago-expect analysis:mixed-argument
take_float(C); // ok

take_int(A2); // ok
take_string(B2); // @mago-expect analysis:mixed-argument
take_float(C2); // ok
