<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

if (PHP_INT_SIZE === 8) {
    $msg = '64-bit';
} else {
    $msg = '32-bit';
}
takesString($msg);
