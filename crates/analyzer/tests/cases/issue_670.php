<?php

declare(strict_types=1);

if (isset($_SESSION)) {
    /** @mago-expect analysis:mixed-assignment */
    $value = $_SESSION['key'] ?? null;
    if (is_string($value)) {
        echo 'session key: ' . $value;
    } else {
        echo 'session key is not a string';
    }
}
