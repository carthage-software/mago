<?php

declare(strict_types=1);

$work = [];
$target = 10;
for ($i = 0; $i < $target; ++$i) {
    $work[$i] = $i;
}

for ($x = 0; $x < ($target * 2); ++$x) {
    if ($x) {
        echo $work[$x - 1];
    } else {
        echo $work[$x]; // this is wrong! key 0 DOES exit.
    }
}
