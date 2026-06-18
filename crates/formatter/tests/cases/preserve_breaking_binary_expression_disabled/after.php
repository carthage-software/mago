<?php

// Multi-line && chain that fits on one line -- should be collapsed
$result = $foo && $bar && $baz;

// Multi-line || chain that fits on one line -- should be collapsed
$check = $one || $two;

// Single-line -- stays on one line
$inline = $a && $b && $c;
