<?php

declare(strict_types=1);

function test_isset_on_openssl_details(): void
{
    /** @var array{bits?: int, key?: string, type?: int}|false $details */
    $details = false;
    if ($details === false || !isset($details['bits']) || $details['bits'] < 1024) {
        return;
    }
}
