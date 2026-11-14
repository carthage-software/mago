<?php

declare(strict_types=1);

function openssl3(): bool
{
    if (OPENSSL_VERSION_TEXT === 'OpenSSL 3.6.1 27 Jan 2026') {
        return true;
    }

    return false;
}

function oniguruma(): bool
{
    if (MB_ONIGURUMA_VERSION === '6.9.10') {
        return true;
    }

    return false;
}

function icu(): bool
{
    if (INTL_ICU_VERSION === '77.1') {
        return true;
    }

    return false;
}

function sodium(): bool
{
    if (SODIUM_LIBRARY_VERSION === '1.0.20') {
        return true;
    }

    return false;
}

function libzip(): bool
{
    if (ZipArchive::LIBZIP_VERSION === '1.11.4') {
        return true;
    }

    return false;
}


function new_iconv(): bool
{
    return version_compare(ICONV_VERSION, '2.0', '>=');
}

function new_unicode(): bool
{
    return version_compare(IntlChar::UNICODE_VERSION, '15.0', '>=');
}

function new_libxml(): bool
{
    return version_compare(LIBXML_LOADED_VERSION, '21300', '>=');
}
