<?php

declare(strict_types=1);

$c = '';
while (mt_rand() < 5) {
    $c .= 'x';
}

if ($c) {
    echo 'this might be reached';
} else {
    echo 'this happens when loop never runs';
}
