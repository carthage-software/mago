<?php

declare(strict_types=1);

if (isset($_SESSION)) {
    $value = $_SESSION['key'] ?? null;
    if (is_string($value)) {
        echo 'session key: ' . $value;
    } else {
        echo 'session key is not a string';
    }
}
