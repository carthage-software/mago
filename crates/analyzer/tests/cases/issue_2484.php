<?php

declare(strict_types=1);

function host(string $url): string
{
    $parts = parse_url($url);
    if (!is_array($parts) || empty($parts['host'])) {
        return 'bad';
    }

    return $parts['host'];
}

function scheme(string $url): string
{
    $parts = parse_url($url);
    if (!is_array($parts) || empty($parts['scheme'])) {
        return 'bad';
    }

    return $parts['scheme'];
}

/** @param array{host?: non-empty-string} $parts */
function optionalHost(array $parts): string
{
    if (empty($parts['host'])) {
        return 'bad';
    }

    return $parts['host'];
}

/** @param array{host?: non-empty-string}|false $parts */
function optionalHostOrFalse(array|false $parts): string
{
    if (!is_array($parts) || empty($parts['host'])) {
        return 'bad';
    }

    return $parts['host'];
}

/** @param array{host?: truthy-string} $parts */
function optionalTruthyHost(array $parts, bool $skip): string
{
    if ($skip || empty($parts['host'])) {
        return 'bad';
    }

    return $parts['host'];
}

/** @param array{host: non-empty-string} $parts */
function nonEmptyHost(array $parts, bool $skip): string
{
    if (!$skip && !empty($parts['host'])) {
        return $parts['host'];
    }

    return 'bad';
}

/**
 * @param array{host: non-empty-string} $parts
 * @return '0'|null
 */
function zeroHost(array $parts): ?string
{
    if (empty($parts['host'])) {
        return $parts['host'];
    }

    return null;
}

/**
 * @param array{host: truthy-string} $parts
 * @return false
 */
function requiredTruthyHost(array $parts): bool
{
    return empty($parts['host']);
}

echo host('mailto:a@b');
echo scheme('//example.com');
echo optionalHost([]);
