<?php

declare(strict_types=1);

$work = [];
$target = 2;
for ($i = 0; $i < $target; ++$i) {
    $work[$i] = [];
    for ($j = 0; $j < $target; ++$j) {
        $work[$i][$j] = $i + ($i * $j);
    }
}
for ($x = 0; $x < $target; ++$x) {
    for ($y = 0; $y < $target; ++$y) {
        $v = 0;
        if ($y) {
            $v += $work[$x][$y - 1];
        } else {
            /** @mago-expect analysis:possibly-undefined-array-index */
            $v += $work[$x][$y];
        }
    }
}
