<?php

declare(strict_types=1);

$consume = function (string $s): string {
    return $s;
};

/** @mago-expect analysis:null-argument */
$consume(null);
