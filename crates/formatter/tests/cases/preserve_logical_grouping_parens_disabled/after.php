<?php

if ($var1 > 200 && $var2 < 1 || $var1 <= 200 && $var2 < 3) {
    run();
}

$a = $x && $y || $z;
$b = $x || $y && $z;

$c = $a && $b || $c && $d || $e && $f;

$d = ($a || $b) && $c;

$e = ($a + $b) * $c;
