<?php

declare(strict_types=1);

$n = 0;
while ($row = mt_rand(0, 3)) {
    ++$n;
}

if ($n) {
    echo 'some';
}
