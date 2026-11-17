<?php

declare(strict_types=1);

function share_dns(): CurlSharePersistentHandle
{
    return curl_share_init_persistent([CURL_LOCK_DATA_DNS, CURL_LOCK_DATA_CONNECT]);
}

function share_nothing(): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    curl_share_init_persistent([]);
}
