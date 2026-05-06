<?php

class A {}

class B {}

function true_or_false(): bool
{
    return true_or_false();
}

function treat_a(A $a): A
{
    return $a;
}

/**
 */
function example(): A
{
    $a = new A();
    $b = new B();
    $a = true_or_false() ? $a : $b;

    /** @var A $a */
    $a = true_or_false() ? treat_a($a) : $b;

    return $a;
}
