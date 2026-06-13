<?php

// Multi-line && chain -- should preserve
$result =
    $foo
    && $bar
    && $baz;

// Multi-line || chain -- should preserve
$check =
    $one
    || $two;

// Single-line -- must NOT be forced to break
$inline = $a && $b && $c;

// Mixed: inline subexpression inside breaking outer
$mixed =
    $foo
    && ($bar || $baz);
