<?php

declare(strict_types=1);

$target = 9;
$div = intval($target / 2);
for ($x = 0; $x < $target; ++$x) {
    for ($y = 0; $y < $target; ++$y) {
        $q = 4;
        if ($x < $div) { // needed somehow.
            continue;
        }

        if ($x > $div) {
            if ($y < $div) { // also needed.
            }
        }
    }
}
