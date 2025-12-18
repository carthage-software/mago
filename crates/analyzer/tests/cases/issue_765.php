<?php

declare(strict_types=1);

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return array{'fragment'?: non-empty-string, 'host'?: non-empty-string, 'pass'?: non-empty-string, 'path': string, 'port'?: int<0, 65535>, 'query'?: non-empty-string, 'scheme'?: non-empty-string, 'user'?: non-empty-string}
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
 * @return non-empty-string|null
 */
function getScheme(string $url): null|string
{
    return parse_url($url, PHP_URL_SCHEME);
}

/**
 * @param string $url
 *
 * @return non-empty-string|null
 */
function getHost(string $url): null|string
{
    return parse_url($url, PHP_URL_HOST);
}

/**
 * @param string $url
 *
 * @return int<0, 65535>|null
 */
function getPort(string $url): null|int
{
    return parse_url($url, PHP_URL_PORT);
}

/**
 * @param string $url
 *
 * @return non-empty-string|null
 */
function getUser(string $url): null|string
{
    return parse_url($url, PHP_URL_USER);
}

/**
 * @param string $url
 *
 * @return non-empty-string|null
 */
function getPass(string $url): null|string
{
    return parse_url($url, PHP_URL_PASS);
}

/**
 * @param string $url
 *
 * @return string|null
 */
function getPath(string $url): null|string
{
    return parse_url($url, PHP_URL_PATH);
}

/**
 * @param string $url
 *
 * @return non-empty-string|null
 */
function getQuery(string $url): null|string
{
    return parse_url($url, PHP_URL_QUERY);
}

/**
 * @param string $url
 *
 * @return non-empty-string|null
 */
function getFragment(string $url): null|string
{
    return parse_url($url, PHP_URL_FRAGMENT);
}

/**
 * @param string $url
 *
 * @throws RuntimeException
 *
 * @return array{'fragment'?: non-empty-string, 'host'?: non-empty-string, 'pass'?: non-empty-string, 'path': string, 'port'?: int<0, 65535>, 'query'?: non-empty-string, 'scheme'?: non-empty-string, 'user'?: non-empty-string}
 */
function getFullResultWithMinusOne(string $url): array
{
    $result = parse_url($url, -1);

    if ($result === false) {
        throw new RuntimeException('Invalid URL');
    }

    return $result;
}

/**
 * Both return null|non-empty-string
 *
 * @param string $url
 * @param int<0, 1> $component (PHP_URL_SCHEME or PHP_URL_HOST)
 *
 * @return non-empty-string|null
 */
function getSchemeOrHost(string $url, int $component): null|string
{
    return parse_url($url, $component);
}

/**
 * @param string $url
 * @param 0|2 $component (PHP_URL_SCHEME or PHP_URL_PORT)
 *
 * @return int<0, 65535>|non-empty-string|null
 */
function getSchemeOrPort(string $url, int $component): int|string|null
{
    return parse_url($url, $component);
}
