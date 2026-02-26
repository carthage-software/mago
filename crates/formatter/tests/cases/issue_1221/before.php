<?php

function x(){
    $x = mt_rand() > 0.5
    ? // This is a multi-
    // line comment explaining the first
    // value
    5 :
     // This is a multi-
    // line comment explaining the second
    // value
    42;
}

$a = $b
    ? // line comment explaining the first
    // This is a multi-
    // line comment
    5
    : 1;

$c = $d
    ? 5
    : // trailing comment on colon
    // another comment
    1;

$e = $f
    ? /* block comment */ 5
    : 1;

$g = $h
    ? // single trailing comment
    5
    : 1;
