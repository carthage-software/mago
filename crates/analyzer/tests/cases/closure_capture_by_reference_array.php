<?php

declare(strict_types=1);

$seen = [];
$isDuplicate = static function (string $key) use (&$seen): bool {
    if (isset($seen[$key])) {
        return true;
    }

    $seen[$key] = true;
    return false;
};

$isDuplicate('a');
$isDuplicate('a');
