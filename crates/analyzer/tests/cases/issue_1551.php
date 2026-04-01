<?php

declare(strict_types=1);

$parents = [];
while (1) {
    $pid = mt_rand(0, max: 10);
    if (!$pid) {
        break;
    }

    $parents[] = $pid;
}

$anchor = 77;
if (count($parents)) {
    $anchor = $parents[count($parents) - 1];
}

if ($anchor == 77) {
    echo '77';
}
