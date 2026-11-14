<?php

declare(strict_types=1);

namespace App;

function boot(): bool
{
    if (!defined('BOOT_FILE')) {
        return false;
    }

    return BOOT_FILE === '';
}

function namespaced(): bool
{
    if (!defined('App\NS_CONST')) {
        return false;
    }

    return NS_CONST === '';
}

function call(): void
{
    if (!function_exists('polyfill')) {
        return;
    }

    polyfill();
}

function unguarded(): bool
{
    // @mago-expect analysis:non-existent-constant
    return OTHER_CONST === '';
}
