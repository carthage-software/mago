<?php

declare(strict_types=1);

function hasEmptyQuery(string $url): bool
{
    $query = parse_url($url, PHP_URL_QUERY);

    return $query !== null && $query !== false && $query === '';
}

function hasEmptyFragment(string $url): bool
{
    $parts = parse_url($url);

    return $parts !== false && isset($parts['fragment']) && $parts['fragment'] === '';
}

function hasEmptyUserInfo(string $url): bool
{
    $user = parse_url($url, PHP_URL_USER);
    $pass = parse_url($url, PHP_URL_PASS);

    return $user === '' || $pass === '';
}
