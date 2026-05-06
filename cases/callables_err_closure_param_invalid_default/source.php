<?php

declare(strict_types=1);

$cb = function (int $n = 'oops'): int {
    return $n;
};

echo $cb(1);
