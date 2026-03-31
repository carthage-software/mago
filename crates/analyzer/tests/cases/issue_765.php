<?php

declare(strict_types=1);

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return array{'fragment'?: non-empty-string, 'host'?: non-empty-string, 'pass'?: non-empty-string, 'path'?: non-empty-string, 'port'?: int<0, 65535>, 'query'?: non-empty-string, 'scheme'?: non-empty-string, 'user'?: non-empty-string}
 */
function getFullResult(string $url): array
{
    $result = parse_url($url);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return non-empty-string|null
 */
function getScheme(string $url): ?string
{
    $result = parse_url($url, PHP_URL_SCHEME);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return non-empty-string|null
 */
function getHost(string $url): ?string
{
    $result = parse_url($url, PHP_URL_HOST);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return int<0, 65535>|null
 */
function getPort(string $url): ?int
{
    $result = parse_url($url, PHP_URL_PORT);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return non-empty-string|null
 */
function getUser(string $url): ?string
{
    $result = parse_url($url, PHP_URL_USER);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return non-empty-string|null
 */
function getPass(string $url): ?string
{
    $result = parse_url($url, PHP_URL_PASS);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return non-empty-string|null
 */
function getPath(string $url): ?string
{
    $result = parse_url($url, PHP_URL_PATH);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return non-empty-string|null
 */
function getQuery(string $url): ?string
{
    $result = parse_url($url, PHP_URL_QUERY);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return non-empty-string|null
 */
function getFragment(string $url): ?string
{
    $result = parse_url($url, PHP_URL_FRAGMENT);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return array{'fragment'?: non-empty-string, 'host'?: non-empty-string, 'pass'?: non-empty-string, 'path'?: non-empty-string, 'port'?: int<0, 65535>, 'query'?: non-empty-string, 'scheme'?: non-empty-string, 'user'?: non-empty-string}
 */
function getFullResultWithMinusOne(string $url): array
{
    $result = parse_url($url, -1);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}
