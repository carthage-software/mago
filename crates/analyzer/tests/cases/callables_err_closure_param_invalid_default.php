<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-parameter-default-value */
$cb = function (int $n = 'oops'): int {
    return $n;
};

echo $cb(1);
