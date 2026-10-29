<?php

function f(object $a, object $b): string
{
    $x = '';
    $x .= $a->one() . $a->two() . $a->three() . $a->four() . $b->five();
    $longerName = 1;

    return $x;
}
