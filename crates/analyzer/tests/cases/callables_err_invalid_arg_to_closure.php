<?php

declare(strict_types=1);

$consume = function (int $n): int {
    return $n;
};

/** @mago-expect analysis:invalid-argument */
$consume('hello');
