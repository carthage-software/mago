<?php

declare(strict_types=1);

$parents = [];
while (1) {
    $pid = mt_rand(0, max: 42);
    if (!$pid) {
        break;
    }
    $parents[] = $pid;
}

if (count($parents)) {
    echo 'do something';
}
