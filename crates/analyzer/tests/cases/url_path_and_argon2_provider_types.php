<?php

declare(strict_types=1);

function url_path_may_be_empty(string $url): bool
{
    $path = parse_url($url, PHP_URL_PATH);

    return !is_string($path) || $path === '';
}

function url_parts_path_may_be_empty(string $url): bool
{
    $parts = parse_url($url);

    return is_array($parts) && isset($parts['path']) && $parts['path'] === '';
}

function argon2_uses_libsodium(): bool
{
    return defined('PASSWORD_ARGON2_PROVIDER') && PASSWORD_ARGON2_PROVIDER === 'sodium';
}
