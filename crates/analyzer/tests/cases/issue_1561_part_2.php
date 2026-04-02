<?php

declare(strict_types=1);

function ipi_find_handler(string $mod, string $action): ?string
{
    $fn = $mod . '_ipi_handler_' . $action;

    if (function_exists($fn)) {
        return $fn;
    }

    if (function_exists($fn)) {
        return $fn;
    }

    return null;
}
