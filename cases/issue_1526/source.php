<?php

declare(strict_types=1);

/** @return array{bool,int,int,int} */
function helper(): array
{
    return [true, mt_rand(0, max: 255), mt_rand(0, max: 255), mt_rand(0, max: 255)];
}

$rs = 0;
$s = 0;
$target = 9;
for ($x = 0; $x < $target; ++$x) {
    for ($y = 0; $y < $target; ++$y) {
        [$ok, $r, $g, $b] = helper();
        if ($ok) {
            $rs += $r;
            $s += $r + $g + $b;
        }
    }
}

$rf = $rs / max($s, 1);
