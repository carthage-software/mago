<?php

declare(strict_types=1);

function anonymous_bind(LDAP\Connection $ldap): bool
{
    return ldap_bind($ldap);
}

function connect(): LDAP\Connection|false
{
    return ldap_connect();
}

function sign(string $data, OpenSSLAsymmetricKey $key): bool
{
    $signature = '';

    return openssl_sign($data, $signature, $key, OPENSSL_ALGO_SHA256, 1);
}

function position(string $haystack, string $needle): int|false
{
    return grapheme_strpos($haystack, $needle, 0, 'en_US');
}

function files(string $directory): RecursiveDirectoryIterator
{
    return new RecursiveDirectoryIterator(directory: $directory);
}

/** @param CachingIterator<int, string, Iterator<int, string>> $iterator */
function flags(CachingIterator $iterator): int
{
    return $iterator->getFlags();
}

/** @param LimitIterator<int, string, Iterator<int, string>> $iterator */
function position_in(LimitIterator $iterator): int
{
    return $iterator->getPosition();
}

function no_arguments(): string
{
    // @mago-expect analysis:too-few-arguments
    return implode();
}
